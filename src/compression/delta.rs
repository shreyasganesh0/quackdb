//! # Lesson 07: Compression — Delta Encoding (File 2 of 2)
//!
//! This file implements delta encoding and frame-of-reference (FOR) encoding.
//! Delta encoding stores differences between consecutive values instead of the
//! values themselves. FOR encoding stores values as unsigned offsets from the
//! column minimum. Both produce small integers ideal for bitpacking.
//!
//! It works together with:
//! - `bitpack.rs` — low-level bitpacking routines that this file's combined
//!   `delta_bitpack_encode`/`decode` functions depend on.
//!
//! **Implementation order**: Implement `bitpack.rs` first, then this file.
//! The standalone `encode`/`decode` and `frame_of_reference_*` functions here
//! are self-contained, but `delta_bitpack_encode`/`decode` call into `bitpack.rs`.
//!
//! Key Rust concepts: iterator `windows()` for pairwise access, `i64`/`u64`
//! conversions, and combining encodings (delta + bitpack).

/// Delta-encoded data: a base value plus a sequence of deltas.
#[derive(Debug, Clone)]
pub struct DeltaEncoded {
    /// The first value (base).
    pub base: i64,
    /// Deltas from each value to the next (length = original_count - 1).
    pub deltas: Vec<i64>,
}

/// Encode a slice of `i64` values using delta encoding.
///
/// Stores the first value as `base`, then each subsequent value as the
/// difference from its predecessor.
// Hint: `deltas[i] = data[i+1] - data[i]`.
pub fn encode(data: &[i64]) -> DeltaEncoded {
    todo!()
}

/// Decode delta-encoded data back to `i64` values.
// Hint: start with `base`, then cumulatively add each delta.
pub fn decode(encoded: &DeltaEncoded) -> Vec<i64> {
    todo!()
}

/// Frame-of-reference encoding: store values as unsigned offsets from the minimum.
///
/// Returns `(min_value, offsets)` where `offsets[i] = data[i] - min_value`.
// Hint: find the min, then subtract it from every value and cast to u64.
pub fn frame_of_reference_encode(data: &[i64]) -> (i64, Vec<u64>) {
    todo!()
}

/// Decode frame-of-reference encoded data back to `i64` values.
// Hint: add `min_val` back to each offset.
pub fn frame_of_reference_decode(min_val: i64, offsets: &[u64]) -> Vec<i64> {
    todo!()
}

/// Combined delta + bitpack encoding for optimal compression of sorted data.
///
/// First delta-encodes, then bitpacks the deltas for maximum compression.
/// The output is a self-contained byte buffer with a small header.
// Hint: delta-encode first, convert deltas to unsigned (zigzag or offset),
// determine bit width, then bitpack. Prepend a header with base, count, and bit_width.
pub fn delta_bitpack_encode(data: &[i64]) -> Vec<u8> {
    todo!()
}

/// Decode combined delta + bitpack encoded data.
// Hint: read the header to recover base, count, and bit_width,
// then unpack and reverse the delta encoding.
pub fn delta_bitpack_decode(encoded: &[u8], count: usize) -> Vec<i64> {
    todo!()
}
