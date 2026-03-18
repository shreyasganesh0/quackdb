//! # Lesson 07: Compression — Bitpacking (File 1 of 2)
//!
//! This file implements bitpacking: packing integers using the minimum number
//! of bits required to represent the largest value. For example, if the maximum
//! value in a column is 15, each value needs only 4 bits instead of the full 32.
//!
//! It works together with:
//! - `delta.rs` — delta and frame-of-reference encoding, which produces small
//!   integer deltas that are then bitpacked using functions from this file.
//!
//! **Start here**: Implement `bitpack.rs` first. The `delta.rs` file's combined
//! `delta_bitpack_encode`/`decode` functions call into the bitpacking routines
//! defined here, so having bitpacking working first makes delta easier to test.
//!
//! Key Rust concepts: bitwise operations (shift, mask, OR), working at the
//! bit level across byte boundaries, and `u64::leading_zeros()`.

/// Determine the minimum number of bits needed to represent `max_value`.
///
/// Returns 0 for `max_value == 0`, 1 for 1, 2 for 2-3, 3 for 4-7, etc.
// Hint: use `64 - max_value.leading_zeros()` (or handle 0 as a special case).
pub fn bits_required(max_value: u64) -> u8 {
    if max_value == 0 {
        return 0;
    }
    (64 - max_value.leading_zeros()) as u8
}

/// Pack a slice of `u32` values using `bit_width` bits per value.
///
/// Returns a byte vector containing the tightly-packed bits.
// Hint: maintain a bit offset into the output buffer. For each value,
// write `bit_width` bits starting at the current offset, handling the
// case where a value spans two bytes.
pub fn pack(values: &[u32], bit_width: u8) -> Vec<u8> {
    todo!()
}

/// Unpack bitpacked data back to `u32` values.
///
/// `count` is the number of values encoded in `packed`.
// Hint: reverse of `pack` -- read `bit_width` bits per value from the
// packed buffer using bit offset arithmetic.
pub fn unpack(packed: &[u8], bit_width: u8, count: usize) -> Vec<u32> {
    todo!()
}

/// Pack a slice of `u64` values using `bit_width` bits per value.
pub fn pack_u64(values: &[u64], bit_width: u8) -> Vec<u8> {
    todo!()
}

/// Unpack bitpacked data back to `u64` values.
pub fn unpack_u64(packed: &[u8], bit_width: u8, count: usize) -> Vec<u64> {
    todo!()
}

/// Calculate compression ratio for bitpacking.
///
/// Compares the original bit width per value to the packed bit width.
pub fn compression_ratio(original_bits: u8, packed_bits: u8) -> f64 {
    if packed_bits == 0 {
        return 0.0;
    }
    original_bits as f64 / packed_bits as f64
}
