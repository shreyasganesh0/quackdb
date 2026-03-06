# Lesson 18: Sort-Merge Join

## What You're Building
A merge join operator that combines two pre-sorted streams by walking through them in tandem. RowComparator performs multi-column comparison respecting sort direction and null ordering. KeyNormalizer converts row values into byte-comparable format so that byte-level comparison (`<` on `Vec<u8>`) preserves the logical ordering. MergeJoinOperator buffers sorted left and right inputs, then performs the merge using two cursors. This join is efficient when both inputs are already sorted, avoiding the cost of building a hash table.

## Rust Concepts You'll Need
- [Iterators](../concepts/iterators.md) -- the merge algorithm conceptually zips two sorted sequences, advancing whichever side is smaller
- [Closures](../concepts/closures.md) -- comparison logic can be encapsulated in closures passed to sort routines
- [Enums and Matching](../concepts/enums_and_matching.md) -- SortDirection and NullOrder enums control comparison behavior; Ordering enum from std::cmp drives the merge logic

## Key Patterns

### Implementing Ordering-Based Comparison
Multi-key comparison chains: compare the first key, and only if equal, proceed to the next. Direction (ascending vs descending) flips the result. Null ordering decides whether NULLs sort before or after real values.

```rust
// Analogy: comparing tournament players by (rank, then name) (NOT the QuackDB solution)
use std::cmp::Ordering;

struct Player { rank: u32, name: String }

fn compare_players(a: &Player, b: &Player, rank_desc: bool) -> Ordering {
    let rank_ord = a.rank.cmp(&b.rank);
    let rank_ord = if rank_desc { rank_ord.reverse() } else { rank_ord };
    match rank_ord {
        Ordering::Equal => a.name.cmp(&b.name),
        other => other,
    }
}

// Chain with .then_with() for cleaner multi-key comparisons:
fn compare_chained(a: &Player, b: &Player) -> Ordering {
    a.rank.cmp(&b.rank)
        .then_with(|| a.name.cmp(&b.name))
}
```

### Merge Algorithm with Two Pointers
Walk two sorted sequences with independent cursors. Compare the current elements. If they match, emit the pair and handle duplicates. If one is smaller, advance that cursor.

```rust
// Analogy: merging two sorted guest lists (NOT the QuackDB solution)
fn merge_guest_lists(left: &[u32], right: &[u32]) -> Vec<(u32, u32)> {
    let (mut i, mut j) = (0, 0);
    let mut matches = Vec::new();
    while i < left.len() && j < right.len() {
        match left[i].cmp(&right[j]) {
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
            Ordering::Equal => {
                // Handle duplicates: find the range of equal values on each side
                let li = i;
                while i < left.len() && left[i] == left[li] { i += 1; }
                let lj = j;
                while j < right.len() && right[j] == right[lj] { j += 1; }
                // Cross-product of the two ranges
                for a in li..i {
                    for b in lj..j {
                        matches.push((left[a], right[b]));
                    }
                }
            }
        }
    }
    matches
}
```

## Step-by-Step Implementation Order
1. Start with `RowComparator::compare_within()` -- iterate over sort_keys, extract values from the chunk at the two row indices, compare them; apply direction reversal for Descending; handle NULLs according to NullOrder; chain with Ordering::then()
2. Implement `RowComparator::compare()` -- same logic but reads from two different chunks
3. Implement `KeyNormalizer::normalize()` -- for each key column, serialize the value into bytes such that byte ordering matches logical ordering; flip bits for descending; use a sentinel byte to distinguish NULLs
4. Implement `MergeJoinOperator::new()` -- store join type, keys, types, and initialize empty buffers and positions
5. Implement `MergeJoinOperator::merge()` for Inner join -- use two pointers walking through the buffered chunks, compare current rows, advance the smaller side, emit matches; handle duplicate keys by tracking ranges
6. Extend merge to Left join -- emit left rows with NULL right columns when the left key has no match
7. Watch out for the duplicate-key cross product: when both sides have multiple rows with the same key, you must emit all combinations

## Reading the Tests
- **`test_row_comparator`** compares rows within a single chunk with two sort keys (col 0 ASC, col 1 ASC). It asserts `(1,200) > (1,100)` and `(1,200) < (2,50)`. This shows that the first key takes priority, and the second key breaks ties.
- **`test_merge_join_duplicates`** has left=[1,1,2] and right=[1,1]. It expects 4 result rows (the 2x2 cross product for key=1). This confirms your merge must correctly handle the cartesian product of duplicate-key ranges.
