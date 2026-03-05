//! Lesson 08: Compression Framework Tests

use quackdb::compression::CompressionAlgorithm;
use quackdb::compression::frame::{
    CompressionAnalyzer, CompressionFrame, CompressionStats,
    auto_compress, decompress,
};

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
    assert_eq!(restored.header.algorithm, CompressionAlgorithm::Rle);
    assert_eq!(restored.header.count, 100);
    assert_eq!(restored.header.uncompressed_size, 800);
    assert_eq!(restored.header.compressed_size, 50);
    assert_eq!(restored.data, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_analyzer_sorted_data() {
    let data: Vec<i64> = (0..1000).collect();
    let stats = CompressionAnalyzer::analyze_i64(&data);
    assert!(stats.is_sorted);
    assert!(!stats.is_constant);
    assert!(stats.distinct_ratio > 0.9);
}

#[test]
fn test_analyzer_constant_data() {
    let data = vec![42i64; 1000];
    let stats = CompressionAnalyzer::analyze_i64(&data);
    assert!(stats.is_constant);
    assert!(stats.is_sorted);
    assert!(stats.distinct_ratio < 0.01);
}

#[test]
fn test_analyzer_low_cardinality() {
    let data: Vec<i64> = (0..1000).map(|i| i % 5).collect();
    let stats = CompressionAnalyzer::analyze_i64(&data);
    assert!(stats.distinct_ratio < 0.05);
    assert!(!stats.is_sorted);
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
fn test_auto_compress_constant() {
    let data = vec![7i64; 1000];
    let frame = auto_compress(&data);
    let decoded = decompress(&frame);
    assert_eq!(decoded, data);
    // Should pick RLE for constant data
    assert_eq!(frame.header.algorithm, CompressionAlgorithm::Rle);
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
    assert_eq!(algo, CompressionAlgorithm::Rle);
}

#[test]
fn test_auto_compress_empty() {
    let data: Vec<i64> = vec![];
    let frame = auto_compress(&data);
    let decoded = decompress(&frame);
    assert!(decoded.is_empty());
}

#[test]
fn test_compress_decompress_all_algorithms() {
    let data: Vec<i64> = (0..100).collect();
    let frame = auto_compress(&data);
    let decoded = decompress(&frame);
    assert_eq!(decoded, data);
    assert_eq!(frame.header.count, 100);
}
