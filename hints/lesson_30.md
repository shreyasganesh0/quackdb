# Lesson 30: Window Functions

## What You're Building
A window function evaluation engine that computes ranking, offset, and aggregate functions over partitions of sorted data. Unlike regular aggregates that collapse rows, window functions attach a computed value to each row based on a "window" of neighboring rows. This powers SQL constructs like ROW_NUMBER(), RANK(), running sums, and LAG/LEAD lookups.

## Concept Recap
Building on Lessons 11-13 (Aggregation and Sorting): Window functions reuse the same aggregation logic (SUM, COUNT, MIN, MAX) but apply it per-row over a sliding frame rather than collapsing groups. The sort keys from the Sort operator determine the order within each partition. You already know how to group data by key columns -- window functions add the twist of computing per-row results within each group.

## Rust Concepts You'll Need
- [Trait Objects](../concepts/trait_objects.md) -- the WindowFunction trait is returned as `Box<dyn WindowFunction>` from a factory function, enabling runtime dispatch over different function types
- [Closures](../concepts/closures.md) -- useful for frame-bound calculations and partition grouping logic
- [Collections](../concepts/collections.md) -- heavy use of Vec for buffering input, grouping by partition keys, and computing per-row results

## Key Patterns

### Partition-Then-Evaluate
Buffer all input, group rows by partition keys, then evaluate the function within each partition independently. This is like grading exams by class section -- you sort papers into piles by section, then rank students within each pile separately.

```rust
// Analogy: grading students per classroom (NOT the QuackDB solution)
use std::collections::HashMap;

struct Student { classroom: String, score: f64 }

fn rank_within_classroom(students: &[Student]) -> Vec<usize> {
    let mut by_class: HashMap<&str, Vec<(usize, f64)>> = HashMap::new();
    for (i, s) in students.iter().enumerate() {
        by_class.entry(&s.classroom).or_default().push((i, s.score));
    }

    let mut ranks = vec![0usize; students.len()];
    for (_class, mut group) in by_class {
        group.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for (rank, (orig_idx, _)) in group.iter().enumerate() {
            ranks[*orig_idx] = rank + 1;
        }
    }
    ranks
}
```

### Frame-Bounded Aggregation
For running sums and counts, resolve frame bounds (e.g., UnboundedPreceding to CurrentRow) into concrete index ranges, then aggregate within that range. Think of it like a telescope with adjustable zoom -- the frame bounds determine how wide the lens is for each row's calculation.

```rust
// Analogy: moving average over sensor readings (NOT the QuackDB solution)
fn running_sum(values: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(values.len());
    let mut acc = 0.0;
    for &v in values {
        acc += v;
        result.push(acc); // frame: unbounded preceding to current row
    }
    result
}

fn sliding_window_sum(values: &[f64], window_size: usize) -> Vec<f64> {
    values.windows(window_size).map(|w| w.iter().sum()).collect()
}
```

### Rank vs. Dense Rank Tie Handling
RANK and DENSE_RANK differ only in how they handle ties. RANK leaves gaps (1, 1, 3, 4), while DENSE_RANK does not (1, 1, 2, 3). Both compare the current row's sort key to the previous row's -- if they match, it is a tie. The difference is whether the next non-tied rank skips ahead or increments by one.

```rust
// Analogy: race results with ties (NOT the QuackDB solution)
fn race_rank(times: &[f64]) -> Vec<usize> {
    let mut ranks = vec![1usize; times.len()];
    for i in 1..times.len() {
        if times[i] == times[i - 1] {
            ranks[i] = ranks[i - 1]; // tie: same rank
        } else {
            ranks[i] = i + 1; // RANK skips: position-based
            // For DENSE_RANK, you would do: ranks[i] = ranks[i-1] + 1
        }
    }
    ranks
}
```

## Common Mistakes
- **Forgetting that window functions are blocking operators.** Unlike filter or projection, a window function must see ALL input rows before it can produce any output. Buffer everything in `execute()`, then do the real work in `finalize()`.
- **Returning results in partition order instead of original row order.** After partitioning and sorting within partitions, the results must be mapped back to the original row positions. Keep track of original indices.
- **Off-by-one errors in frame bound resolution.** "UnboundedPreceding to CurrentRow" means indices [0, current_row] inclusive. "CurrentRow to CurrentRow" means just the single row. Check your range calculations carefully.

## Where to Start
Start with `RowNumber` — it is the simplest window function (just counting). Then implement `Rank` and `DenseRank` (compare adjacent rows). The aggregate windows (Sum, Avg) use the same frame-resolution logic, so implement one and the rest follow the pattern.

## Step-by-Step Implementation Order
1. Start with `create_window_function()` -- match on WindowFunctionType and return the appropriate struct boxed as `Box<dyn WindowFunction>`; each function type needs a struct implementing the trait.
2. Implement RowNumber's `evaluate()` -- simply assign 1, 2, 3... based on position in the ordered partition.
3. Implement Rank and DenseRank -- compare adjacent rows in sort order; Rank skips numbers on ties, DenseRank does not.
4. Implement Lag/Lead -- look back/forward by offset in the ordered indices, returning a default value (or Null) at boundaries.
5. Implement aggregate windows (Sum, Avg, Count, Min, Max) -- resolve frame bounds to start/end indices per row, aggregate within that range.
6. Implement `WindowOperator::new()` -- compute output_types by appending window result types to input types.
7. Implement `WindowOperator::execute()` -- buffer input chunks (return NeedMoreInput); do the real work in `finalize()`.
8. Implement `WindowOperator::finalize()` -- combine buffered input, partition by partition_by columns, sort within partitions, evaluate each WindowDef, append result columns.
9. Watch out for: window functions are blocking operators -- they must see all input before producing output.

## Reading the Tests
- **`test_row_number`** creates a window with no partitioning but ordered by column 1. It evaluates on 5 rows and asserts each result is Int64 values 1 through 5 in sequence. This confirms the evaluate function receives ordered indices and produces sequential numbering. It is the simplest window function to validate.
- **`test_rank`** creates 4 rows with values [10, 10, 20, 30] and evaluates Rank. With two tied values at 10, the expected ranks are 1, 1, 3, 4 (gap after the tie). This validates that your tie detection compares sort key values and that RANK uses position-based numbering after ties.
- **`test_dense_rank`** uses the same data as test_rank but expects 1, 1, 2, 3 (no gaps). This confirms the difference between Rank and DenseRank -- DenseRank increments by 1 after a tie group.
- **`test_running_sum`** uses a frame of UnboundedPreceding to CurrentRow on values [10, 20, 30]. The expected running sums are 10, 30, 60. This validates that your aggregate window functions correctly interpret frame bounds and accumulate within the frame.
- **`test_lag_lead`** evaluates Lag and Lead on values [1, 2, 3] and checks that both produce 3 results. Lag should return Null (or default) for the first row; Lead should return Null (or default) for the last row. This tests boundary handling for offset functions.
- **`test_window_count`** evaluates a running COUNT with UnboundedPreceding to CurrentRow on 3 rows, expecting results 1, 2, 3. This validates that COUNT increments correctly per frame expansion.
- **`test_window_operator_pipeline`** integrates the window operator into a full Pipeline with an InMemorySource. It uses ROW_NUMBER partitioned by column 0 (department) and ordered by column 2 (salary descending). It expects all 5 input rows to appear in the output with an additional window column. This is the end-to-end integration test.
