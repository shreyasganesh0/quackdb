//! Lesson 07: Delta Encoding
//!
//! Store differences between consecutive values for sorted/sequential data.

/// Delta-encoded data.
#[derive(Debug, Clone)]
pub struct DeltaEncoded {
    /// The first value (base).
    pub base: i64,
    /// Deltas from each value to the next.
    pub deltas: Vec<i64>,
}

/// Encode a slice of i64 values using delta encoding.
pub fn encode(data: &[i64]) -> DeltaEncoded {
    todo!()
}

/// Decode delta-encoded data back to i64 values.
pub fn decode(encoded: &DeltaEncoded) -> Vec<i64> {
    todo!()
}

/// Frame-of-reference encoding: store values relative to a minimum.
pub fn frame_of_reference_encode(data: &[i64]) -> (i64, Vec<u64>) {
    todo!()
}

/// Decode frame-of-reference encoded data.
pub fn frame_of_reference_decode(min_val: i64, offsets: &[u64]) -> Vec<i64> {
    todo!()
}

/// Combined delta + bitpack encoding for optimal compression of sorted data.
pub fn delta_bitpack_encode(data: &[i64]) -> Vec<u8> {
    todo!()
}

/// Decode combined delta + bitpack encoded data.
pub fn delta_bitpack_decode(encoded: &[u8], count: usize) -> Vec<i64> {
    todo!()
}
