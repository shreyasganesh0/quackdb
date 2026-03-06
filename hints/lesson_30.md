# Lesson 30: Window Functions

## What You're Building
A window function evaluation engine that computes ranking, offset, and aggregate functions over partitions of sorted data. Unlike regular aggregates that collapse rows, window functions attach a computed value to each row based on a "window" of neighboring rows. This powers SQL constructs like ROW_NUMBER(), RANK(), running sums, and LAG/LEAD lookups.

## Rust Concepts You'll Need
- [Trait Objects](../concepts/trait_objects.md) -- the WindowFunction trait is returned as `Box<dyn WindowFunction>` from a factory function, enabling runtime dispatch over different function types
- [Closures](../concepts/closures.md) -- useful for frame-bound calculations and partition grouping logic
- [Collections](../concepts/collections.md) -- heavy use of Vec for buffering input, grouping by partition keys, and computing per-row results

## Key Patterns

### Partition-Then-Evaluate
Buffer all input, group rows by partition keys, then evaluate the function within each partition independently.

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
For running sums and counts, resolve frame bounds (e.g., UnboundedPreceding to CurrentRow) into concrete index ranges, then aggregate within that range.

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

## Step-by-Step Implementation Order
1. Start with `create_window_function()` -- match on WindowFunctionType and return the appropriate struct boxed as `Box<dyn WindowFunction>`; each function type needs a struct implementing the trait
2. Implement RowNumber's `evaluate()` -- simply assign 1, 2, 3... based on position in the ordered partition
3. Implement Rank and DenseRank -- compare adjacent rows in sort order; Rank skips numbers on ties, DenseRank does not
4. Implement Lag/Lead -- look back/forward by offset in the ordered indices, returning a default value at boundaries
5. Implement aggregate windows (Sum, Avg, Count, Min, Max) -- resolve frame bounds to start/end indices per row, aggregate within that range
6. Implement `WindowOperator::new()` -- compute output_types by appending window result types to input types
7. Implement `WindowOperator::execute()` -- buffer input chunks (return NeedMoreInput); do the real work in `finalize()`
8. Implement `WindowOperator::finalize()` -- combine buffered input, partition by partition_by columns, sort within partitions, evaluate each WindowDef, append result columns
9. Watch out for: window functions are blocking operators -- they must see all input before producing output

## Reading the Tests
- **`test_row_number`** creates a window with no partitioning but ordered by column 1. It evaluates on 5 rows and asserts each result is Int64 values 1 through 5 in sequence. This confirms the evaluate function receives ordered indices and produces sequential numbering.
- **`test_running_sum`** uses a frame of UnboundedPreceding to CurrentRow on values [10, 20, 30]. The expected running sums are 10, 30, 60. This validates that your aggregate window functions correctly interpret frame bounds and accumulate within the frame.
