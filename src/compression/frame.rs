//! Lesson 08: Compression Framework
//!
//! A self-describing compression frame format with automatic algorithm selection.
//! Each compressed block is wrapped in a frame that records which algorithm was
//! used, the value count, and the compressed/uncompressed sizes -- making the
//! data self-describing and decompressible without external metadata.
//!
//! Key Rust concepts: trait objects vs. enum dispatch, serialization/
//! deserialization of binary headers, and heuristic-based algorithm selection.

use super::CompressionAlgorithm;

/// Header for a compressed frame, making it self-describing.
///
/// Serialized as a fixed-size binary prefix before the compressed payload.
#[derive(Debug, Clone)]
pub struct CompressionFrameHeader {
    /// Which algorithm was used.
    pub algorithm: CompressionAlgorithm,
    /// Number of values in this frame.
    pub count: u32,
    /// Uncompressed size in bytes.
    pub uncompressed_size: u32,
    /// Compressed size in bytes (after header).
    pub compressed_size: u32,
}

/// A self-describing compressed frame (header + compressed payload).
#[derive(Debug, Clone)]
pub struct CompressionFrame {
    pub header: CompressionFrameHeader,
    pub data: Vec<u8>,
}

impl CompressionFrame {
    /// Serialize the frame (header + data) to bytes.
    // Hint: write the header fields in a fixed order (little-endian),
    // then append the compressed data bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    /// Deserialize a frame from bytes.
    // Hint: parse the fixed-size header first, then read `compressed_size`
    // bytes of payload.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        todo!()
    }
}

/// Statistics gathered by analyzing a data sample, used to pick the best
/// compression algorithm.
#[derive(Debug, Clone)]
pub struct CompressionStats {
    /// Whether the data is sorted in ascending order.
    pub is_sorted: bool,
    /// Whether all values are identical.
    pub is_constant: bool,
    /// Ratio of distinct values to total values (0.0 to 1.0).
    pub distinct_ratio: f64,
    /// Number of runs (for RLE analysis).
    pub run_count: usize,
    /// Minimum value in the sample.
    pub min_value: i64,
    /// Maximum value in the sample.
    pub max_value: i64,
}

/// Analyzes data samples to determine the best compression algorithm.
pub struct CompressionAnalyzer;

impl CompressionAnalyzer {
    /// Analyze `i64` data and compute compression statistics.
    // Hint: iterate once to compute min, max, distinct count, run count,
    // and sortedness in a single pass.
    pub fn analyze_i64(data: &[i64]) -> CompressionStats {
        todo!()
    }

    /// Pick the best algorithm based on computed statistics.
    ///
    /// Heuristic: constant data -> RLE, sorted data -> delta+bitpack,
    /// low cardinality -> dictionary, otherwise -> bitpacking.
    pub fn pick_algorithm(stats: &CompressionStats) -> CompressionAlgorithm {
        todo!()
    }
}

/// Automatically compress `i64` data using the best-fit algorithm.
///
/// Analyzes the data, picks an algorithm, compresses, and wraps the result
/// in a self-describing `CompressionFrame`.
pub fn auto_compress(data: &[i64]) -> CompressionFrame {
    todo!()
}

/// Decompress a frame back to `i64` values.
///
/// Reads the algorithm from the frame header and dispatches to the
/// appropriate decoder.
pub fn decompress(frame: &CompressionFrame) -> Vec<i64> {
    todo!()
}
