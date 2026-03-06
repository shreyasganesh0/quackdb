# Lesson 05: RLE Compression

## What You're Building
Run-Length Encoding compresses sequences where the same value repeats consecutively.
Instead of storing `[A, A, A, B, B]`, you store `[(A, 3), (B, 2)]`. Databases use RLE
on sorted columns where long runs of identical values are common -- for example, a
"country" column sorted by region. You will also build a skip index that enables O(1)
random access into the compressed data.

## Rust Concepts You'll Need
- [Structs and Impl](../concepts/structs_and_impl.md) -- Run<T> and RleEncoded<T> are generic structs
- [Traits and Derive](../concepts/traits_and_derive.md) -- trait bounds Clone + PartialEq on encode, Clone on decode

## Key Patterns

### Generic Functions with Trait Bounds
The `encode` function works for any type that can be compared (`PartialEq`) and
cloned (`Clone`). Rust's trait bounds enforce this at compile time.

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
position `i`, jump to `skip_index[i / N]` and scan forward from there.

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

## Step-by-Step Implementation Order
1. Start with `encode()` -- iterate the slice, track current value and run count; push a Run when the value changes or the slice ends
2. Build the skip index inside `encode()` -- choose a skip_interval (e.g., 128), walk the runs tracking cumulative position, record which run covers each interval boundary
3. Implement `decode()` -- iterate runs, repeat each value by its count
4. Implement `get_at_index()` -- use the skip index to find the starting run, then scan forward to find the run that contains the target index
5. Implement `encode_bytes()` and `decode_bytes()` -- simpler byte-pair format; handle edge cases like empty input
6. Implement `compression_ratio()` -- original_len divided by number of runs (consider what "size" means for the encoded form)
7. Watch out for: empty input (return empty results), single-element input, and the skip index off-by-one at boundaries

## Reading the Tests
- **`test_rle_random_access`** creates 5 runs of 100 elements each, then checks `get_at_index` at positions 0, 99, 100, 250, and 499. Position 100 is the first element of the second run -- this is the boundary case your skip index must handle correctly.
- **`test_rle_all_same`** encodes 1000 copies of 42 and expects exactly 1 run with count=1000. The compression ratio test expects > 10x, which tells you the ratio formula is `original_len / runs.len()` or similar.
- **`test_rle_roundtrip_strings`** proves that encode/decode works for `&str` slices, confirming your generic bounds are sufficient for non-numeric types.
