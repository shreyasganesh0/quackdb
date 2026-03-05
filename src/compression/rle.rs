//! Lesson 05: Run-Length Encoding
//!
//! Encode/decode typed data using run-length encoding.

/// A single run: value repeated `count` times.
#[derive(Debug, Clone, PartialEq)]
pub struct Run<T> {
    pub value: T,
    pub count: u32,
}

/// RLE-encoded data with skip index for random access.
#[derive(Debug, Clone)]
pub struct RleEncoded<T> {
    pub runs: Vec<Run<T>>,
    /// Skip index: maps every N-th element to a run index for O(1) random access.
    pub skip_index: Vec<u32>,
    pub skip_interval: usize,
    pub total_count: usize,
}

/// Encode a slice of values using run-length encoding.
pub fn encode<T: Clone + PartialEq>(data: &[T]) -> RleEncoded<T> {
    todo!()
}

/// Decode RLE-encoded data back to a flat vector.
pub fn decode<T: Clone>(encoded: &RleEncoded<T>) -> Vec<T> {
    todo!()
}

/// Get the value at a specific index using the skip index for fast access.
pub fn get_at_index<T: Clone>(encoded: &RleEncoded<T>, index: usize) -> T {
    todo!()
}

/// Encode raw bytes using RLE (byte-level).
pub fn encode_bytes(data: &[u8]) -> Vec<u8> {
    todo!()
}

/// Decode RLE-encoded bytes.
pub fn decode_bytes(encoded: &[u8]) -> Vec<u8> {
    todo!()
}

/// Calculate the compression ratio (original_size / compressed_size).
pub fn compression_ratio<T>(original_len: usize, encoded: &RleEncoded<T>) -> f64 {
    todo!()
}
