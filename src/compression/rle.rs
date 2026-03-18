//! Lesson 05: Run-Length Encoding
//!
//! Compress data by replacing consecutive repeated values with (value, count)
//! pairs. Particularly effective for sorted columns or columns with long runs
//! of identical values (e.g., region codes, boolean flags).
//!
//! Key Rust concepts: generics with `Clone + PartialEq` bounds, iterator
//! patterns, and building a skip index for efficient random access.

/// A single run: `value` repeated `count` times.
#[derive(Debug, Clone, PartialEq)]
pub struct Run<T> {
    pub value: T,
    pub count: u32,
}

/// RLE-encoded data with a skip index for random access.
///
/// The `skip_index` maps every `skip_interval`-th logical position to a run
/// index, enabling O(1) lookup instead of scanning all runs.
#[derive(Debug, Clone)]
pub struct RleEncoded<T> {
    pub runs: Vec<Run<T>>,
    /// Skip index: maps every N-th element to a run index for O(1) random access.
    pub skip_index: Vec<u32>,
    pub skip_interval: usize,
    pub total_count: usize,
}

/// Encode a slice of values using run-length encoding.
///
/// Consecutive equal values are collapsed into a single `Run`.
// Hint: iterate through `data`, tracking the current value and count.
// When the value changes, push the completed Run and start a new one.
// Build the skip index after all runs are collected.
pub fn encode<T: Clone + PartialEq>(data: &[T]) -> RleEncoded<T> {
    todo!()
}

/// Decode RLE-encoded data back to a flat vector.
// Hint: iterate over runs, pushing `run.value` repeated `run.count` times.
pub fn decode<T: Clone>(encoded: &RleEncoded<T>) -> Vec<T> {
    todo!()
}

/// Get the value at a specific logical index using the skip index.
///
/// Uses the skip index for O(1) lookup of the approximate run, then
/// scans forward to find the exact run containing `index`.
pub fn get_at_index<T: Clone>(encoded: &RleEncoded<T>, index: usize) -> T {
    todo!()
}

/// Encode raw bytes using byte-level RLE.
///
/// Output format: pairs of `[count, value]` bytes. Runs longer than 255
/// must be split into multiple pairs.
pub fn encode_bytes(data: &[u8]) -> Vec<u8> {
    todo!()
}

/// Decode byte-level RLE-encoded data.
// Hint: read pairs of (count, value) and repeat each value count times.
pub fn decode_bytes(encoded: &[u8]) -> Vec<u8> {
    todo!()
}

/// Calculate the compression ratio (original_size / compressed_size).
///
/// A ratio > 1.0 means the data got smaller; < 1.0 means it grew.
pub fn compression_ratio<T>(original_len: usize, encoded: &RleEncoded<T>) -> f64 {
    todo!()
}
