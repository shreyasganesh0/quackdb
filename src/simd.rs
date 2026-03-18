//! Lesson 35: SIMD-Style Vectorization
//!
//! Implements tight, branch-free loops that the compiler can auto-vectorize
//! into SIMD instructions (SSE/AVX on x86, NEON on ARM). The key patterns:
//! operate on contiguous slices, avoid branches inside loops, and use simple
//! arithmetic that maps 1:1 to SIMD ops. Optional: use `#[target_feature]`
//! and `std::arch` intrinsics for explicit SIMD.

/// Add two `i32` slices element-wise, writing results to `out`.
///
/// All three slices must have the same length. This loop is designed for
/// auto-vectorization: no branches, no dependencies between iterations.
pub fn vectorized_add_i32(a: &[i32], b: &[i32], out: &mut [i32]) {
    for i in 0..a.len() {
        out[i] = a[i] + b[i];
    }
}

/// Add two `f64` slices element-wise.
///
/// Same auto-vectorization pattern as `vectorized_add_i32`.
pub fn vectorized_add_f64(a: &[f64], b: &[f64], out: &mut [f64]) {
    for i in 0..a.len() {
        out[i] = a[i] + b[i];
    }
}

/// Selective filter: write the indices of values greater than `threshold`
/// into `indices` and return the count.
///
/// Uses a branchless pattern: always compute the index, but only advance
/// the write pointer when the condition is true.
pub fn vectorized_filter_gt_i32(values: &[i32], threshold: i32, indices: &mut [u32]) -> usize {
    let mut count = 0;
    for i in 0..values.len() {
        indices[count] = i as u32;
        count += (values[i] > threshold) as usize;
    }
    count
}

/// Selective filter for `f64` values greater than `threshold`.
pub fn vectorized_filter_gt_f64(values: &[f64], threshold: f64, indices: &mut [u32]) -> usize {
    let mut count = 0;
    for i in 0..values.len() {
        indices[count] = i as u32;
        count += (values[i] > threshold) as usize;
    }
    count
}

/// Compute FNV-1a-style hashes for a slice of `i64` values.
///
/// Each value is hashed independently into `out[i]`. FNV-1a:
/// `hash = offset_basis; for each byte: hash ^= byte; hash *= prime;`
pub fn vectorized_hash_i64(values: &[i64], out: &mut [u64]) {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    for i in 0..values.len() {
        let bytes = values[i].to_le_bytes();
        let mut hash = FNV_OFFSET;
        for &b in &bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        out[i] = hash;
    }
}

/// Branchless minimum of two `i32` slices.
///
/// Avoids `if` inside the loop so the compiler can vectorize with `vmin`.
pub fn branchless_min_i32(a: &[i32], b: &[i32], out: &mut [i32]) {
    for i in 0..a.len() {
        out[i] = a[i].min(b[i]);
    }
}

/// Branchless maximum of two `i32` slices.
pub fn branchless_max_i32(a: &[i32], b: &[i32], out: &mut [i32]) {
    for i in 0..a.len() {
        out[i] = a[i].max(b[i]);
    }
}

/// Null-aware vectorized addition: if either input is null (validity bit = 0),
/// the output is 0 and the output validity bit is cleared.
///
/// Validity bitmasks are packed as `u64` words (bit N corresponds to element N).
pub fn vectorized_add_nullable_i32(
    a: &[i32],
    b: &[i32],
    validity_a: &[u64],
    validity_b: &[u64],
    out: &mut [i32],
    validity_out: &mut [u64],
) {
    // Combine validity bitmasks with bitwise AND
    for w in 0..validity_a.len() {
        validity_out[w] = validity_a[w] & validity_b[w];
    }
    // Branchless addition: multiply by validity bit
    for i in 0..a.len() {
        let word = i / 64;
        let bit = i % 64;
        let valid = ((validity_out[word] >> bit) & 1) as i32;
        out[i] = (a[i] + b[i]) * valid;
    }
}

/// Allocate a byte buffer aligned to `alignment` bytes for SIMD operations.
///
/// Many SIMD instructions require or perform better with aligned loads.
pub fn aligned_alloc(size: usize, alignment: usize) -> Vec<u8> {
    // Hint: use `std::alloc::Layout::from_size_align` and `std::alloc::alloc`
    // to get an aligned pointer, then wrap it in a Vec via `from_raw_parts`.
    // Alternatively, over-allocate and manually align the start offset.
    todo!()
}

/// Sum all elements of an `i32` slice using an auto-vectorizable tight loop.
///
/// Returns `i64` to avoid overflow on large inputs.
pub fn vectorized_sum_i32(values: &[i32]) -> i64 {
    values.iter().map(|&v| v as i64).sum()
}

/// Sum all elements of an `f64` slice.
pub fn vectorized_sum_f64(values: &[f64]) -> f64 {
    values.iter().sum()
}
