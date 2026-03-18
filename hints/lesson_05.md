# Lesson 05: RLE Compression

## What You're Building
Run-Length Encoding compresses sequences where the same value repeats consecutively.
Instead of storing `[A, A, A, B, B]`, you store `[(A, 3), (B, 2)]`. Databases use RLE
on sorted columns where long runs of identical values are common -- for example, a
"country" column sorted by region. You will also build a skip index that enables O(1)
random access into the compressed data.

## Concept Recap
Building on Lesson 04: You built `DataChunk` and `Vector` which store columnar data in contiguous byte buffers. Now you will compress those column values using RLE. The compression works on slices of typed data (like `&[i32]`) that you would extract from a Vector's internal buffer.

## Rust Concepts You'll Need
- [Structs and Impl](../concepts/structs_and_impl.md) -- Run<T> and RleEncoded<T> are generic structs
- [Traits and Derive](../concepts/traits_and_derive.md) -- trait bounds Clone + PartialEq on encode, Clone on decode

## Key Patterns

### Generic Functions with Trait Bounds
The `encode` function works for any type that can be compared (`PartialEq`) and
cloned (`Clone`). Rust's trait bounds enforce this at compile time. Think of a
sorting machine that works on any item with a label -- it just needs to read the
label (PartialEq) and make copies (Clone).

```rust
// Analogy: grouping consecutive identical items in a shopping list
fn group_consecutive<T: Clone + PartialEq>(items: &[T]) -> Vec<(T, usize)> {
    let mut groups = Vec::new();
    if items.is_empty() { return groups; }
    let mut current = items[0].clone();
    let mut count = 1;
    for item in &items[1..] {
        if *item == current {
            count += 1;
        } else {
            groups.push((current, count));
            current = item.clone();
            count = 1;
        }
    }
    groups.push((current, count));
    groups
}
```

### Building a Skip Index for O(1) Access
A skip index records "at every N-th logical position, which run are we in?" To look up
position `i`, jump to `skip_index[i / N]` and scan forward from there. It works like a
book's index -- instead of scanning every page, jump to the nearest chapter heading.

```rust
// Analogy: a book's chapter index (NOT the QuackDB solution)
struct BookIndex {
    chapter_starts: Vec<usize>,  // page where each chapter begins
    page_interval: usize,        // every N pages we record the chapter
    page_to_chapter: Vec<usize>, // skip_index[page / N] = chapter_idx
}

impl BookIndex {
    fn chapter_at_page(&self, page: usize) -> usize {
        let hint = self.page_to_chapter[page / self.page_interval];
        let mut ch = hint;
        while ch + 1 < self.chapter_starts.len()
            && self.chapter_starts[ch + 1] <= page
        {
            ch += 1;
        }
        ch
    }
}
```

### Byte-Level RLE
`encode_bytes` and `decode_bytes` work on raw `&[u8]`. A common format is pairs of
`[count, value]` bytes. Watch out for runs longer than 255 -- you may need to split
them into multiple pairs or use a wider count encoding.

### Compression Ratio Calculation
The ratio tells you how effective compression is: `original_count / runs_count` or
a similar formula. A ratio of 1.0 means no compression; 10.0 means 10x smaller.
Think of it like a zip file -- the ratio tells you how much space you saved.

```rust
// Analogy: measuring how well a summary captures a book
fn summary_ratio(full_pages: usize, summary_pages: usize) -> f64 {
    full_pages as f64 / summary_pages as f64
}
```

## Step-by-Step Implementation Order
1. Start with `encode()` -- iterate the slice, track current value and run count; push a Run when the value changes or the slice ends; handle empty input by returning empty runs
2. Build the skip index inside `encode()` -- choose a skip_interval (e.g., 128), walk the runs tracking cumulative position, record which run covers each interval boundary
3. Set `total_count` on the RleEncoded struct to the input length
4. Implement `decode()` -- iterate runs, repeat each value by its count
5. Implement `get_at_index()` -- use the skip index to find the starting run, then scan forward to find the run that contains the target index
6. Implement `encode_bytes()` and `decode_bytes()` -- simpler byte-pair format; handle edge cases like empty input
7. Implement `compression_ratio()` -- original_len divided by number of runs (consider what "size" means for the encoded form)
8. Handle edge cases: empty input (return empty results), single-element input, and alternating values (worst case where every element is its own run)

## Common Mistakes
- **Off-by-one at run boundaries in `get_at_index`**: Index 99 is the last element of a 100-count run starting at 0, and index 100 is the first element of the next run. Make sure your cumulative position tracking uses `<` vs `<=` correctly.
- **Forgetting the final run in `encode`**: After the loop ends, you must push the last accumulated run. A common bug is to only push runs when the value changes, missing the final group entirely.
- **Not handling the worst case gracefully**: Alternating values like `[0, 1, 0, 1, ...]` produce one run per element. The roundtrip must still be lossless even when there is no actual compression benefit.

## Reading the Tests
- **`test_rle_all_same`** encodes 1000 copies of 42 and expects exactly 1 run with count=1000. The decoded output must equal the input. This is the best-case scenario for RLE.
- **`test_rle_alternating`** encodes alternating 0s and 1s (100 elements) and expects 100 runs -- one per element. This is the worst case. It confirms RLE must be lossless even when no compression occurs.
- **`test_rle_sorted`** creates 10 groups of 50 identical values and expects exactly 10 runs. This represents the typical use case: sorted columns with long runs.
- **`test_rle_single_element`** encodes a single-element vec and expects 1 run. Edge case for the smallest possible input.
- **`test_rle_empty`** encodes an empty vec and expects 0 runs and total_count of 0. Your code must handle empty slices without panicking.
- **`test_rle_random_access`** creates 5 runs of 100 elements each, then checks `get_at_index` at positions 0, 99, 100, 250, and 499. Position 100 is the first element of the second run -- this is the boundary case your skip index must handle correctly.
- **`test_rle_roundtrip_strings`** proves that encode/decode works for `&str` slices, confirming your generic bounds are sufficient for non-numeric types. It expects 3 runs for ["hello", "hello", "hello", "world", "world", "foo"].
- **`test_rle_compression_ratio`** encodes 10000 identical values and checks the ratio is > 10x. This tells you the formula produces large ratios when runs are very long.
- **`test_rle_bytes`** encodes 500 copies of byte 0xAA and checks the encoded form is smaller than the original. This tests your byte-level RLE format.
- **`test_rle_bytes_mixed`** encodes bytes 0-255 (all different) and verifies the roundtrip is lossless. Worst case for byte RLE.

## What Comes Next
RLE works well for sorted data with long runs. Lesson 06 introduces **dictionary
encoding**, which handles the opposite case: columns with many repeated values
scattered throughout (low cardinality but not sorted). Together, RLE and dictionary
encoding cover the two most common compression scenarios for string columns.
