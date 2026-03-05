//! Lesson 07: Bitpacking
//!
//! Pack integers using the minimum number of bits required.

/// Determine the minimum number of bits needed to represent a value.
pub fn bits_required(max_value: u64) -> u8 {
    todo!()
}

/// Pack a slice of u32 values using `bit_width` bits per value.
pub fn pack(values: &[u32], bit_width: u8) -> Vec<u8> {
    todo!()
}

/// Unpack bitpacked data back to u32 values.
pub fn unpack(packed: &[u8], bit_width: u8, count: usize) -> Vec<u32> {
    todo!()
}

/// Pack a slice of u64 values using `bit_width` bits per value.
pub fn pack_u64(values: &[u64], bit_width: u8) -> Vec<u8> {
    todo!()
}

/// Unpack bitpacked data back to u64 values.
pub fn unpack_u64(packed: &[u8], bit_width: u8, count: usize) -> Vec<u64> {
    todo!()
}

/// Calculate compression ratio for bitpacking.
pub fn compression_ratio(original_bits: u8, packed_bits: u8) -> f64 {
    todo!()
}
