# Lesson 18: Sort-Merge Join

## What You're Building
A merge join operator that combines two pre-sorted streams by walking through them in tandem. RowComparator performs multi-column comparison respecting sort direction and null ordering. KeyNormalizer converts row values into byte-comparable format so that byte-level comparison (`<` on `Vec<u8>`) preserves the logical ordering. MergeJoinOperator buffers sorted left and right inputs, then performs the merge using two cursors. This join is efficient when both inputs are already sorted, avoiding the cost of building a hash table.

## Concept Recap
Building on Lessons 14-17: You'll reuse `JoinType` from the hash join (Lesson 17) and `DataChunk`/`Vector` from the storage layer. The merge join operator integrates into the `Pipeline` framework from Lesson 14 as an alternative join strategy. While hash join excels with unsorted data, merge join shines when both inputs arrive pre-sorted.

## Rust Concepts You'll Need
- [Iterators](../concepts/iterators.md) -- the merge algorithm conceptually zips two sorted sequences, advancing whichever side is smaller
- [Closures](../concepts/closures.md) -- comparison logic can be encapsulated in closures passed to sort routines
- [Enums and Matching](../concepts/enums_and_matching.md) -- SortDirection and NullOrder enums control comparison behavior; Ordering enum from std::cmp drives the merge logic

## Key Patterns

### Implementing Ordering-Based Comparison
Multi-key comparison chains: compare the first key, and only if equal, proceed to the next. Think of it like sorting names in a phone book -- last name first, then first name to break ties, then middle initial if still tied.

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
Walk two sorted sequences with independent cursors. Compare the current elements. If they match, emit the pair and handle duplicates. If one is smaller, advance that cursor. Think of it like merging two sorted decks of cards -- you always play the smaller card from whichever deck has it.

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

### Byte-Normalized Keys for Type-Agnostic Comparison
Converting values to byte-comparable form lets you compare rows using simple byte ordering regardless of the original types. Think of it like converting all measurements to meters before comparing -- once everything is in the same format, comparison becomes trivial.

## Common Mistakes
- Not handling the duplicate-key cross product correctly. When both sides have N and M rows with the same key, you must emit N*M output rows. Forgetting to track the range boundaries leads to missed combinations.
- Flipping the comparison for descending order at the wrong point. The direction reversal should apply to the final Ordering, not to the raw values. Using `.reverse()` on the Ordering is the cleanest approach.
- Not distinguishing NullsFirst vs NullsLast. NULL handling in comparisons requires explicit checks before comparing values -- if either value is NULL, return an Ordering based on the null_order setting without comparing the actual values.

## Step-by-Step Implementation Order
1. Start with `RowComparator::compare_within()` -- iterate over sort_keys, extract values from the chunk at the two row indices, compare them; apply direction reversal for Descending; handle NULLs according to NullOrder; chain with Ordering::then()
2. Implement `RowComparator::compare()` -- same logic but reads from two different chunks
3. Implement `KeyNormalizer::normalize()` -- for each key column, serialize the value into bytes such that byte ordering matches logical ordering; flip bits for descending; use a sentinel byte to distinguish NULLs
4. Implement `MergeJoinOperator::new()` -- store join type, keys, types, and initialize empty buffers and positions
5. Implement `MergeJoinOperator::add_left()` and `add_right()` -- buffer the sorted input chunks
6. Implement `MergeJoinOperator::merge()` for Inner join -- use two pointers walking through the buffered chunks, compare current rows, advance the smaller side, emit matches; handle duplicate keys by tracking ranges
7. Extend merge to Left join -- emit left rows with NULL right columns when the left key has no match
8. Watch out for the duplicate-key cross product: when both sides have multiple rows with the same key, you must emit all combinations
9. Handle edge cases: empty inputs, all-matching keys, and no-matching keys

## Reading the Tests
- **`test_merge_join_inner`** merges left=[1,3,5] with right=[1,2,3,4]. Only keys 1 and 3 match, so `total == 2`. This validates the basic two-pointer merge that skips non-matching keys.
- **`test_merge_join_duplicates`** has left=[1,1,2] and right=[1,1]. It expects 4 result rows (the 2x2 cross product for key=1). This confirms your merge must correctly handle the cartesian product of duplicate-key ranges.
- **`test_row_comparator`** compares rows within a single chunk with two sort keys (col 0 ASC, col 1 ASC). It asserts `(1,200) > (1,100)` and `(1,200) < (2,50)`. This shows that the first key takes priority, and the second key breaks ties.
- **`test_row_comparator_descending`** compares 10 vs 20 with descending order and asserts `10 > 20` in that context. This confirms direction reversal flips the comparison result.
- **`test_row_comparator_nulls_first`** asserts that NULL is Less than 10 when NullsFirst is set. This verifies your NULL handling places NULLs before all values.
- **`test_row_comparator_nulls_last`** asserts that NULL is Greater than 10 when NullsLast is set. This is the opposite of NullsFirst.
- **`test_key_normalizer`** normalizes two rows and asserts `k1 < k2` using byte comparison. This confirms that your byte encoding preserves the logical sort order across types.
- **`test_merge_join_left_outer`** merges left=[1,3,5] with right=[1,2,3,4]. Expects 3 rows: key 1 matches, key 3 matches, key 5 has no match (right columns are NULL). This validates Left join's preservation of all left rows.
