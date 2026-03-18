# Lesson 19: External Sort

## What You're Building
A sort engine with three components. MinHeap is a generic binary heap parameterized by a boxed comparison function, used for k-way merging. ExternalSortOperator sorts data that may exceed memory by splitting it into sorted runs, then merging them. TopNOperator optimizes ORDER BY ... LIMIT N by keeping only the N smallest rows. Real databases rely on sort for ORDER BY, GROUP BY, merge joins, and duplicate elimination.

## Concept Recap
Building on Lessons 14 and 18: You'll use `RowComparator`, `SortKey`, `SortDirection`, and `NullOrder` from Lesson 18 for comparison logic. The sort operator integrates into the `Pipeline` framework from Lesson 14 as a pipeline-breaking operator. The `DataChunk` structure carries data through sort and merge phases. This is the final piece of the execution engine before you move to the SQL frontend.

## Rust Concepts You'll Need
- [Generics](../concepts/generics.md) -- MinHeap<T> works with any element type
- [Closures](../concepts/closures.md) -- the heap comparator is `Box<dyn Fn(&T, &T) -> Ordering>`, a boxed closure for runtime-polymorphic comparison
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- Box<dyn Fn> needed because closures are unsized trait objects
- [Collections](../concepts/collections.md) -- Vec backs both the heap and sorted runs

## Key Patterns

### Heap with Custom Comparator (Box<dyn Fn>)
A heap that accepts any ordering logic at runtime via a boxed closure, called during sift-up and sift-down. Think of it like a priority queue at a hospital -- the triage nurse (comparator) decides who goes first, and the rule can change depending on context (emergency room vs scheduled appointments).

```rust
// Analogy: a priority task queue (NOT the QuackDB solution)
use std::cmp::Ordering;

struct PriorityQueue<T> {
    items: Vec<T>,
    compare: Box<dyn Fn(&T, &T) -> Ordering>,
}

impl<T> PriorityQueue<T> {
    fn new(cmp: impl Fn(&T, &T) -> Ordering + 'static) -> Self {
        Self { items: Vec::new(), compare: Box::new(cmp) }
    }
    fn sift_up(&mut self, mut idx: usize) {
        while idx > 0 {
            let parent = (idx - 1) / 2;
            if (self.compare)(&self.items[idx], &self.items[parent]) == Ordering::Less {
                self.items.swap(idx, parent);
                idx = parent;
            } else { break; }
        }
    }
}
```

### External Sort with Sorted Runs and Merge
When data exceeds the memory budget, sort each batch independently (a "run"), then merge all runs using a min-heap tracking which run each element came from. Think of it like sorting a huge pile of mail -- you sort manageable stacks on your desk (runs), then merge the sorted stacks by always picking the next letter from whichever stack has the smallest address.

```rust
// Analogy: merging sorted sublists (NOT the QuackDB solution)
fn k_way_merge_lists(runs: &[Vec<i32>]) -> Vec<i32> {
    let mut cursors = vec![0usize; runs.len()];
    let mut result = Vec::new();
    loop {
        let best = (0..runs.len())
            .filter(|&r| cursors[r] < runs[r].len())
            .min_by_key(|&r| runs[r][cursors[r]]);
        match best {
            Some(r) => { result.push(runs[r][cursors[r]]); cursors[r] += 1; }
            None => break,
        }
    }
    result
}
```

### Sorting by Permutation Index
Rather than moving entire rows around during sort, create a vector of row indices `[0, 1, 2, ...]`, sort the indices using the comparator, then rearrange the chunk according to the sorted indices. Think of it like sorting library books by writing their shelf positions on cards, sorting the cards, then reshuffling the books in one pass.

## Common Mistakes
- Getting the sift_down boundaries wrong. When popping from a min-heap, you swap the first and last elements, remove the last, then sift down from index 0. The sift-down loop must check both children and stop when neither child is smaller than the current element.
- Forgetting the `'static` lifetime bound on the heap comparator closure. Because the closure is boxed, it must own all its data -- it cannot borrow references with shorter lifetimes.
- Returning unsorted data from TopN. The TopN operator must both select the N smallest rows AND return them in sorted order.

## Step-by-Step Implementation Order
1. Start with `MinHeap::new()` -- store an empty Vec and Box the comparison closure
2. Implement `push()` -- append to Vec, sift up from the last index
3. Implement `pop()` -- swap first and last, remove last, sift down from index 0; return None if empty
4. Implement `peek()`, `len()`, `is_empty()` helper methods
5. Implement `ExternalSortOperator::sort_chunk()` -- create a permutation of row indices, sort it using RowComparator from lesson 18, rearrange the chunk according to the sorted permutation
6. Implement `execute()` -- buffer incoming chunks; when `current_run_size` exceeds `memory_budget`, sort and flush the current run
7. Implement `finalize()` -- flush remaining data as a run, then call `k_way_merge`
8. Implement `k_way_merge()` -- use a MinHeap of (run_index, chunk_index, row_index) tuples; pop smallest, emit it, advance that run's cursor
9. Implement `TopNOperator` -- buffer all input, sort, return only the first N rows

## Reading the Tests
- **`test_min_heap`** pushes [5, 2, 8, 1, 4] and pops [1, 2, 4, 5, 8]. This confirms your sift_up and sift_down produce correct min-heap ordering. Every pop must return the smallest remaining element.
- **`test_min_heap_single`** pushes one element, peeks at it, pops it, and checks `is_empty()`. This validates your heap with the simplest possible case and tests the peek/is_empty helpers.
- **`test_sort_in_memory`** sorts 5 rows by column 0 ascending and checks the output order [1, 2, 4, 5, 8]. It also asserts the row count is preserved. This is the basic correctness test for `sort_chunk`.
- **`test_sort_descending`** sorts the same data in descending order and checks 8 is first, 1 is last. This confirms your sort respects the `SortDirection::Descending` setting.
- **`test_sort_multi_column`** sorts by two columns (col 0 ASC, col 1 ASC). Rows with col0=1 are ordered by col1: [10, 20, 30]. This validates multi-key sorting where the second key breaks ties.
- **`test_sort_with_nulls`** sorts `[3, NULL, 1]` with NullsLast. Expects order [1, 3, NULL]. This confirms NULL values are placed at the end per the NullOrder setting.
- **`test_k_way_merge`** creates three sorted runs ([1,4,7], [2,5,8], [3,6,9]) and expects merged output [1..9]. This verifies your merge correctly interleaves multiple runs using the min-heap.
- **`test_external_sort_pipeline`** creates a sort operator with a 1MB budget and runs it through the pipeline. This integration test confirms the operator works within the pipeline framework.
- **`test_top_n`** expects the 3 smallest values [1, 2, 4] from 5 unsorted rows, confirming TopN returns N smallest in sorted order.
- **`test_sort_strings`** sorts Varchar values ["charlie", "alice", "bob"] and expects alphabetical order. This confirms your sort handles string types, not just integers.

## What Comes Next
You've built a complete vectorized execution engine: expressions, pipelines, scans,
filters, joins, aggregations, and sorting. Part V adds the **SQL frontend** -- the
layer that turns human-readable queries into the operator pipelines you just built.
Lesson 20 starts with lexing SQL text into tokens. By Lesson 24, you'll connect the
parser through the planner to your execution operators, achieving end-to-end SQL
query processing.
