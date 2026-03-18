# Lesson 19: External Sort

## What You're Building
A sort engine with three components. MinHeap is a generic binary heap parameterized by a boxed comparison function, used for k-way merging. ExternalSortOperator sorts data that may exceed memory by splitting it into sorted runs, then merging them. TopNOperator optimizes ORDER BY ... LIMIT N by keeping only the N smallest rows. Real databases rely on sort for ORDER BY, GROUP BY, merge joins, and duplicate elimination.

## Rust Concepts You'll Need
- [Generics](../concepts/generics.md) -- MinHeap<T> works with any element type
- [Closures](../concepts/closures.md) -- the heap comparator is `Box<dyn Fn(&T, &T) -> Ordering>`, a boxed closure for runtime-polymorphic comparison
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- Box<dyn Fn> needed because closures are unsized trait objects
- [Collections](../concepts/collections.md) -- Vec backs both the heap and sorted runs

## Key Patterns

### Heap with Custom Comparator (Box<dyn Fn>)
A heap that accepts any ordering logic at runtime via a boxed closure, called during sift-up and sift-down.

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
When data exceeds the memory budget, sort each batch independently (a "run"), then merge all runs using a min-heap tracking which run each element came from.

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

## Step-by-Step Implementation Order
1. Start with `MinHeap::new()` -- store an empty Vec and Box the comparison closure
2. Implement `push()` -- append to Vec, sift up from the last index
3. Implement `pop()` -- swap first and last, remove last, sift down from index 0; return None if empty
4. Implement `ExternalSortOperator::sort_chunk()` -- create a permutation of row indices, sort it using RowComparator from lesson 18, rearrange the chunk
5. Implement `execute()` -- buffer incoming chunks; when `current_run_size` exceeds `memory_budget`, sort and flush the current run
6. Implement `finalize()` -- flush remaining data as a run, then call `k_way_merge`
7. Implement `k_way_merge()` -- use a MinHeap of (run_index, chunk_index, row_index) tuples; pop smallest, emit it, advance that run's cursor
8. Implement `TopNOperator` -- buffer all input, sort, return only the first N rows
9. Watch out: the heap comparator closure must be `'static` because it is boxed

## Reading the Tests
- **`test_min_heap`** pushes [5, 2, 8, 1, 4] and pops [1, 2, 4, 5, 8]. This confirms your sift_up and sift_down produce correct min-heap ordering.
- **`test_k_way_merge`** creates three sorted runs ([1,4,7], [2,5,8], [3,6,9]) and expects merged output [1..9]. This verifies your merge correctly interleaves multiple runs.
- **`test_top_n`** expects the 3 smallest values [1, 2, 4] from 5 unsorted rows, confirming TopN returns N smallest in sorted order.

## What Comes Next
You've built a complete vectorized execution engine: expressions, pipelines, scans,
filters, joins, aggregations, and sorting. Part V adds the **SQL frontend** — the
layer that turns human-readable queries into the operator pipelines you just built.
Lesson 20 starts with lexing SQL text into tokens. By Lesson 24, you'll connect the
parser through the planner to your execution operators, achieving end-to-end SQL
query processing.
