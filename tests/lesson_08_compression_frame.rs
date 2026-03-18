//! # Lesson 08: Compression Framework — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Frame serialization roundtrip (`test_frame_serialize_roundtrip`)
//! 2. Data analysis — sorted, constant, low-cardinality (`test_analyzer_*`)
//! 3. Edge cases (empty input, single element)
//! 4. Algorithm selection (`test_pick_algorithm`)
//! 5. Auto-compress + decompress correctness (`test_auto_compress_*`)
//! 6. Integration — compress/decompress all algorithms

use quackdb::compression::CompressionAlgorithm;
use quackdb::compression::frame::{
    CompressionAnalyzer, CompressionFrame, CompressionStats,
    auto_compress, decompress,
};

// ── 1. Frame serialization roundtrip ────────────────────────────────

#[test]
fn test_frame_serialize_roundtrip() {
    let frame = CompressionFrame {
        header: quackdb::compression::frame::CompressionFrameHeader {
            algorithm: CompressionAlgorithm::Rle,
            count: 100,
            uncompressed_size: 800,
            compressed_size: 50,
        },
        data: vec![1, 2, 3, 4, 5],
    };

    let bytes = frame.to_bytes();
    let restored = CompressionFrame::from_bytes(&bytes).unwrap();
    assert_eq!(restored.header.algorithm, CompressionAlgorithm::Rle, "algorithm tag must survive serialization for correct decoding");
    assert_eq!(restored.header.count, 100);
    assert_eq!(restored.header.uncompressed_size, 800, "uncompressed size is needed for buffer allocation on read");
    assert_eq!(restored.header.compressed_size, 50);
    assert_eq!(restored.data, vec![1, 2, 3, 4, 5], "payload bytes must be identical after round-trip");
}

// ── 2. Data analysis ────────────────────────────────────────────────

#[test]
fn test_analyzer_constant_data() {
    let data = vec![42i64; 1000];
    let stats = CompressionAnalyzer::analyze_i64(&data);
    assert!(stats.is_constant, "repeated single value is constant");
    assert!(stats.is_sorted, "constant data is trivially sorted");
    assert!(stats.distinct_ratio < 0.01, "one distinct value in 1000 rows gives a near-zero ratio");
}

#[test]
fn test_analyzer_sorted_data() {
    let data: Vec<i64> = (0..1000).collect();
    let stats = CompressionAnalyzer::analyze_i64(&data);
    assert!(stats.is_sorted, "0..1000 is monotonically increasing");
    assert!(!stats.is_constant, "sorted is not the same as constant");
    assert!(stats.distinct_ratio > 0.9, "all-unique values should have distinct_ratio near 1.0");
}

#[test]
fn test_analyzer_low_cardinality() {
    let data: Vec<i64> = (0..1000).map(|i| i % 5).collect();
    let stats = CompressionAnalyzer::analyze_i64(&data);
    assert!(stats.distinct_ratio < 0.05, "5 distinct values in 1000 rows is low cardinality");
    assert!(!stats.is_sorted, "cycling modulo pattern is not sorted");
}

// ── 3. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_auto_compress_empty() {
    let data: Vec<i64> = vec![];
    let frame = auto_compress(&data);
    let decoded = decompress(&frame);
    assert!(decoded.is_empty(), "compressing empty input must round-trip to empty");
}

#[test]
fn test_auto_compress_single_element() {
    // Edge case: single-element input
    let data: Vec<i64> = vec![42];
    let frame = auto_compress(&data);
    let decoded = decompress(&frame);
    assert_eq!(decoded, data, "single-element input must survive compress/decompress roundtrip");
    assert_eq!(frame.header.count, 1, "frame header count must be 1 for single-element input");
}

// ── 4. Algorithm selection ──────────────────────────────────────────

#[test]
fn test_pick_algorithm() {
    let sorted_stats = CompressionStats {
        is_sorted: true,
        is_constant: false,
        distinct_ratio: 1.0,
        run_count: 1000,
        min_value: 0,
        max_value: 999,
    };
    let algo = CompressionAnalyzer::pick_algorithm(&sorted_stats);
    assert!(algo == CompressionAlgorithm::Delta || algo == CompressionAlgorithm::DeltaBitpack);

    let const_stats = CompressionStats {
        is_sorted: true,
        is_constant: true,
        distinct_ratio: 0.001,
        run_count: 1,
        min_value: 42,
        max_value: 42,
    };
    let algo = CompressionAnalyzer::pick_algorithm(&const_stats);
    assert_eq!(algo, CompressionAlgorithm::Rle, "constant data collapses to one run, making RLE optimal");
}

// ── 5. Auto-compress + decompress ───────────────────────────────────

#[test]
fn test_auto_compress_constant() {
    let data = vec![7i64; 1000];
    let frame = auto_compress(&data);
    let decoded = decompress(&frame);
    assert_eq!(decoded, data);
    // Should pick RLE for constant data
    assert_eq!(frame.header.algorithm, CompressionAlgorithm::Rle);
}

#[test]
fn test_auto_compress_sorted() {
    let data: Vec<i64> = (0..1000).collect();
    let frame = auto_compress(&data);
    let decoded = decompress(&frame);
    assert_eq!(decoded, data);
    // Should pick delta or delta_bitpack for sorted data
    assert!(
        frame.header.algorithm == CompressionAlgorithm::Delta
        || frame.header.algorithm == CompressionAlgorithm::DeltaBitpack,
        "Expected delta-based compression for sorted data, got {:?}",
        frame.header.algorithm
    );
}

#[test]
fn test_auto_compress_low_cardinality() {
    let data: Vec<i64> = (0..1000).map(|i| i % 3).collect();
    let frame = auto_compress(&data);
    let decoded = decompress(&frame);
    assert_eq!(decoded, data);
    // Should pick dictionary or RLE for low-cardinality
    assert!(
        frame.header.algorithm == CompressionAlgorithm::Dictionary
        || frame.header.algorithm == CompressionAlgorithm::Rle,
        "Expected dictionary/RLE for low-cardinality, got {:?}",
        frame.header.algorithm
    );
}

#[test]
fn test_auto_compress_random() {
    // Random-looking data with high cardinality
    let data: Vec<i64> = (0..1000).map(|i| ((i * 1337) % 100000) as i64).collect();
    let frame = auto_compress(&data);
    let decoded = decompress(&frame);
    assert_eq!(decoded, data);
    // For random data, bitpack or none should be used
}

// ── 6. Integration ──────────────────────────────────────────────────

#[test]
fn test_compress_decompress_all_algorithms() {
    let data: Vec<i64> = (0..100).collect();
    let frame = auto_compress(&data);
    let decoded = decompress(&frame);
    assert_eq!(decoded, data);
    assert_eq!(frame.header.count, 100, "frame header must record the logical row count for decoding");
}

#[test]
fn test_auto_compress_two_elements() {
    // Edge case: two elements — minimal non-trivial input
    let data: Vec<i64> = vec![10, 20];
    let frame = auto_compress(&data);
    let decoded = decompress(&frame);
    assert_eq!(decoded, data, "two-element input must survive compress/decompress roundtrip");
}
