# Lesson 35: SIMD Vectorization

## What You're Building
A set of tight, auto-vectorizable loop functions that process data in bulk over slices. Modern CPUs have SIMD (Single Instruction, Multiple Data) units that can perform the same operation on multiple values simultaneously. By writing simple loops without branches or complex control flow, you let the Rust compiler auto-vectorize your code into SIMD instructions. This lesson covers element-wise arithmetic, filtering, hashing, branchless min/max, null-aware operations with validity bitmasks, and aligned memory allocation.

## Concept Recap
Building on the entire QuackDB journey: This lesson optimizes the innermost loops of everything you have built. The vectorized_add functions accelerate the expression evaluation from Lessons 7-8. The vectorized_filter accelerates the selection vectors from Lesson 5. The validity bitmasks connect to the null handling throughout your type system. The hash function accelerates the hash joins and hash aggregation from Lessons 11-12. This is the final performance layer that makes your analytical database competitive with production systems.

## Rust Concepts You'll Need
- [Slices and Bytes](../concepts/slices_and_bytes.md) -- all functions operate on &[T] and &mut [T] slices; understanding slice indexing and length is essential
- [Unsafe Rust](../concepts/unsafe_rust.md) -- aligned_alloc requires unsafe for custom memory alignment; the rest can be safe Rust that the compiler auto-vectorizes
- [Bitwise Operations](../concepts/bitwise_ops.md) -- validity bitmasks use u64 arrays where each bit represents one row's null/valid status

## Key Patterns

### Tight Loops for Auto-Vectorization
The compiler can auto-vectorize simple loops that iterate over slices with no branches, no function calls, and predictable access patterns. Assert equal lengths to help the compiler reason about bounds. Think of it like an assembly line -- if every station does the exact same operation on each item with no special cases, you can process multiple items simultaneously.

```rust
// Analogy: bulk temperature conversion (NOT the QuackDB solution)
fn celsius_to_fahrenheit(celsius: &[f64], fahrenheit: &mut [f64]) {
    assert_eq!(celsius.len(), fahrenheit.len());
    for i in 0..celsius.len() {
        fahrenheit[i] = celsius[i] * 1.8 + 32.0;
    }
}
```

### Branchless Operations
Avoid if/else in inner loops. Instead, use arithmetic or standard library functions like `.min()` and `.max()` to select values. The compiler can turn these into conditional moves (cmov) which avoid branch misprediction. This is like a sorting machine that uses physical rails to guide items to the right bin rather than a human making decisions at each item.

```rust
// Analogy: branchless clamping of pixel values (NOT the QuackDB solution)
fn branchless_clamp(values: &[i32], lo: i32, hi: i32, out: &mut [i32]) {
    for i in 0..values.len() {
        // Using min/max which the compiler can turn into cmov instructions
        out[i] = values[i].max(lo).min(hi);
    }
}
```

### Validity Bitmask Processing
Null values are tracked as bits in u64 arrays. Bit N in word N/64 at position N%64 indicates whether row N is valid. Combine validity masks with bitwise AND to propagate nulls. This is like overlaying two transparencies -- only where both have a mark does the result show through.

```rust
// Analogy: combining two sensor validity flags (NOT the QuackDB solution)
fn combine_validity(mask_a: &[u64], mask_b: &[u64], out: &mut [u64]) {
    for i in 0..mask_a.len() {
        out[i] = mask_a[i] & mask_b[i]; // valid only if both are valid
    }
}

fn is_valid(mask: &[u64], row: usize) -> bool {
    let word = row / 64;
    let bit = row % 64;
    (mask[word] & (1u64 << bit)) != 0
}
```

### Aligned Memory Allocation
SIMD instructions often require or benefit from memory aligned to 32 or 64 bytes. Use `std::alloc::Layout` with alignment to allocate raw memory, then wrap it safely. This is like parking a truck -- SIMD needs data at specific "parking spots" (memory addresses) that are multiples of the alignment size.

```rust
// Analogy: allocating an aligned audio buffer (NOT the QuackDB solution)
fn alloc_aligned_buffer(size: usize, align: usize) -> Vec<u8> {
    let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
    unsafe {
        let ptr = std::alloc::alloc_zeroed(layout);
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        Vec::from_raw_parts(ptr, size, size)
    }
}
```

## Common Mistakes
- **Adding branches inside tight loops.** An `if` statement inside a loop over data prevents auto-vectorization. Use `.min()`, `.max()`, or arithmetic tricks instead. For filtering, write indices to an output array rather than conditionally skipping.
- **Forgetting assert_eq! on slice lengths.** Without the assertion, the compiler cannot prove the slices have the same length and must insert bounds checks on every access, preventing vectorization.
- **Using i32 accumulator for i32 sum.** Summing many i32 values can overflow. Use an i64 accumulator to hold the running total, then return i64. The tests check correctness for sums that fit in i64 but might overflow i32.

## Step-by-Step Implementation Order
1. Start with `vectorized_add_i32()` and `vectorized_add_f64()` -- simple element-wise loops: `out[i] = a[i] + b[i]`; assert equal lengths; handle empty slices naturally (zero iterations).
2. Implement `vectorized_filter_gt_i32()` and `vectorized_filter_gt_f64()` -- iterate over values, when value > threshold, write the index to `indices[count]` and increment count; return count.
3. Implement `vectorized_hash_i64()` -- use FNV-1a or a multiplicative hash: start with an offset basis, XOR in the value bytes, multiply by the FNV prime; write each hash to out.
4. Implement `branchless_min_i32()` and `branchless_max_i32()` -- use `a[i].min(b[i])` and `a[i].max(b[i])` which the compiler will optimize to branchless instructions.
5. Implement `vectorized_sum_i32()` and `vectorized_sum_f64()` -- accumulate in a loop; for i32 sum, use i64 accumulator to avoid overflow.
6. Implement `vectorized_add_nullable_i32()` -- compute validity_out by ANDing validity_a and validity_b word-by-word; for each element, add values (the result at invalid positions does not matter as long as validity is correct, but zeroing is clean).
7. Implement `aligned_alloc()` -- use `std::alloc::Layout::from_size_align` and `std::alloc::alloc_zeroed` in an unsafe block; construct a Vec from the raw pointer.
8. Watch out for: aligned_alloc must use unsafe and handle the Layout correctly; the validity bitmask operates at word granularity (u64), so process 64 elements per word; ensure the sum functions use a wider accumulator type to prevent overflow.

## Rust Sidebar: Unsafe Alloc + Layout
If you hit `Layout::from_size_align: invalid parameters` or `misaligned pointer dereference`, here's what's happening: `std::alloc::alloc_zeroed` requires a `Layout` with non-zero size and a power-of-two alignment. Passing `align=0` or `size=0` panics. The returned pointer must be used carefully -- it is raw, unmanaged memory.
The fix: `let layout = std::alloc::Layout::from_size_align(size, align).unwrap();` then `unsafe { let ptr = std::alloc::alloc_zeroed(layout); Vec::from_raw_parts(ptr, size, size) }`. The `Vec::from_raw_parts` takes ownership of the allocation. Critical: `Vec` will call `dealloc` with the *default* layout on drop, so only use this for aligned buffers where you control the lifecycle.

## Reading the Tests
- **`test_vectorized_add_i32`** adds [1,2,3,4,5] + [10,20,30,40,50] and expects [11,22,33,44,55]. This is the simplest test -- get element-wise addition working first. It verifies basic correctness with no edge cases.
- **`test_vectorized_add_f64`** does the same for floating point: [1.0,2.0,3.0] + [0.5,0.5,0.5] = [1.5,2.5,3.5]. This tests that your f64 version handles floating-point addition correctly.
- **`test_vectorized_filter_gt_i32`** filters [1,5,3,8,2,9,4] for values > 4 and expects 3 indices pointing to values 5, 8, and 9. This tests that your filter correctly writes matching indices and returns the right count.
- **`test_vectorized_filter_gt_f64`** filters [1.0,5.0,3.0,8.0] for values > 4.0 and expects count 2. This covers the floating-point filter variant.
- **`test_vectorized_hash_i64`** hashes [1,2,3,4,5] and checks that adjacent values produce distinct hashes and that hashing the same input twice yields identical results. This tests both correctness (determinism) and quality (distinct outputs).
- **`test_branchless_min_i32`** computes element-wise min of [3,1,4,1,5] and [2,7,1,8,2], expecting [2,1,1,1,2]. **`test_branchless_max_i32`** does the same for max, expecting [3,7,4,8,5]. These tests validate that your min/max implementations compare correctly at every position.
- **`test_vectorized_add_nullable`** uses validity_a = 0b1011 (index 2 is null) and validity_b = 0b1110 (index 0 is null). It checks that out[1] == 22 and out[3] == 44, which are the positions where both inputs are valid. This verifies your bitmask AND logic and the null-aware addition.
- **`test_vectorized_sum_i32`** sums [1..10] and expects 55. **`test_vectorized_sum_f64`** sums [1.0..5.0] and expects 15.0. **`test_vectorized_sum_empty`** sums an empty slice and expects 0. These three tests cover the normal case, floating-point variant, and the identity-element edge case.
- **`test_aligned_alloc`** allocates 256 bytes with 64-byte alignment and asserts the pointer address modulo 64 equals 0. This confirms your unsafe allocation produces correctly aligned memory.
- **`test_vectorized_large_batch`** runs vectorized_add on 10000 elements and verifies every element. This stress-tests correctness at scale, catching any off-by-one errors that might not appear in small inputs.
- **`test_vectorized_empty`** runs vectorized_add on empty slices and expects an empty result. This tests the zero-iteration edge case.
