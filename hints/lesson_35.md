# Lesson 35: SIMD Vectorization

## What You're Building
A set of tight, auto-vectorizable loop functions that process data in bulk over slices. Modern CPUs have SIMD (Single Instruction, Multiple Data) units that can perform the same operation on multiple values simultaneously. By writing simple loops without branches or complex control flow, you let the Rust compiler auto-vectorize your code into SIMD instructions. This lesson covers element-wise arithmetic, filtering, hashing, branchless min/max, null-aware operations with validity bitmasks, and aligned memory allocation.

## Rust Concepts You'll Need
- [Slices and Bytes](../concepts/slices_and_bytes.md) -- all functions operate on &[T] and &mut [T] slices; understanding slice indexing and length is essential
- [Unsafe Rust](../concepts/unsafe_rust.md) -- aligned_alloc requires unsafe for custom memory alignment; the rest can be safe Rust that the compiler auto-vectorizes
- [Bitwise Operations](../concepts/bitwise_ops.md) -- validity bitmasks use u64 arrays where each bit represents one row's null/valid status

## Key Patterns

### Tight Loops for Auto-Vectorization
The compiler can auto-vectorize simple loops that iterate over slices with no branches, no function calls, and predictable access patterns. Assert equal lengths to help the compiler reason about bounds.

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
Avoid if/else in inner loops. Instead, use arithmetic to select values. The compiler can turn these into conditional moves (cmov) which avoid branch misprediction.

```rust
// Analogy: branchless clamping of pixel values (NOT the QuackDB solution)
fn branchless_clamp(values: &[i32], lo: i32, hi: i32, out: &mut [i32]) {
    for i in 0..values.len() {
        // Using min/max which the compiler can turn into cmov instructions
        let v = values[i];
        let clamped = if v < lo { lo } else if v > hi { hi } else { v };
        out[i] = clamped;
        // Alternatively: out[i] = v.max(lo).min(hi);
    }
}
```

### Validity Bitmask Processing
Null values are tracked as bits in u64 arrays. Bit N in word N/64 at position N%64 indicates whether row N is valid. Combine validity masks with bitwise AND to propagate nulls.

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
SIMD instructions often require or benefit from memory aligned to 32 or 64 bytes. Use `std::alloc::Layout` with alignment to allocate raw memory, then wrap it safely.

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

## Step-by-Step Implementation Order
1. Start with `vectorized_add_i32()` and `vectorized_add_f64()` -- simple element-wise loops: `out[i] = a[i] + b[i]`; handle empty slices naturally (zero iterations)
2. Implement `vectorized_filter_gt_i32()` and `vectorized_filter_gt_f64()` -- iterate over values, when value > threshold, write the index to `indices[count]` and increment count; return count
3. Implement `vectorized_hash_i64()` -- use FNV-1a or a multiplicative hash: start with an offset basis, XOR in the value bytes, multiply by the FNV prime; write each hash to out
4. Implement `branchless_min_i32()` and `branchless_max_i32()` -- use `a[i].min(b[i])` and `a[i].max(b[i])` which the compiler will optimize to branchless instructions
5. Implement `vectorized_sum_i32()` and `vectorized_sum_f64()` -- accumulate in a loop; for i32 sum, use i64 accumulator to avoid overflow
6. Implement `vectorized_add_nullable_i32()` -- compute validity_out by ANDing validity_a and validity_b word-by-word; for each element, add values but zero the result if the bit is unset
7. Implement `aligned_alloc()` -- use `std::alloc::Layout::from_size_align` and `std::alloc::alloc_zeroed` in an unsafe block; construct a Vec from the raw pointer
8. Watch out for: aligned_alloc must use unsafe and handle the Layout correctly; the validity bitmask operates at word granularity (u64), so process 64 elements per word; ensure the sum functions use a wider accumulator type to prevent overflow

## Reading the Tests
- **`test_vectorized_add_nullable`** uses validity_a = 0b1011 (index 2 is null) and validity_b = 0b1110 (index 0 is null). It checks that out[1] == 22 and out[3] == 44, which are the positions where both inputs are valid. This verifies your bitmask AND logic and the null-aware addition.
- **`test_aligned_alloc`** allocates 256 bytes with 64-byte alignment and asserts the pointer address modulo 64 equals 0. This confirms your unsafe allocation produces correctly aligned memory.
