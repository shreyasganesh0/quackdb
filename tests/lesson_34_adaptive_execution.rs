//! Lesson 34: Adaptive Query Execution Tests

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::execution::adaptive::*;
use quackdb::execution::pipeline::*;

#[test]
fn test_bloom_filter_basic() {
    let mut bf = BloomFilter::new(1000, 0.01);
    bf.insert(b"hello");
    bf.insert(b"world");

    assert!(bf.might_contain(b"hello"));
    assert!(bf.might_contain(b"world"));
}

#[test]
fn test_bloom_filter_false_positive_rate() {
    let mut bf = BloomFilter::new(1000, 0.01);
    for i in 0..1000u32 {
        bf.insert(&i.to_le_bytes());
    }

    for i in 0..1000u32 {
        assert!(bf.might_contain(&i.to_le_bytes()));
    }

    let mut false_positives = 0;
    for i in 1000..2000u32 {
        if bf.might_contain(&i.to_le_bytes()) {
            false_positives += 1;
        }
    }
    let fp_rate = false_positives as f64 / 1000.0;
    assert!(fp_rate < 0.05, "False positive rate {} too high", fp_rate);
}

#[test]
fn test_bloom_filter_empty() {
    let bf = BloomFilter::new(100, 0.01);
    assert!(!bf.might_contain(b"anything"));
}

#[test]
fn test_runtime_statistics() {
    let stats = RuntimeStatistics::default();
    assert_eq!(stats.rows_processed, 0);
    assert_eq!(stats.bytes_processed, 0);
}

#[test]
fn test_adaptive_join_operator() {
    let mut op = AdaptiveJoinOperator::new(
        vec![LogicalType::Int32, LogicalType::Int64],
        10000,
    );

    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(100)]);

    let _ = op.execute(&chunk);
}

#[test]
fn test_adaptive_parallelism() {
    let mut ap = AdaptiveParallelism::new(1, 8);
    assert!(ap.current_workers() >= 1);

    let stats = RuntimeStatistics {
        rows_processed: 1000000,
        bytes_processed: 1000000,
        execution_time_us: 1000,
        actual_cardinality: 1000000,
    };

    let workers = ap.adjust(&stats);
    assert!(workers >= 1 && workers <= 8);
}

#[test]
fn test_adaptive_parallelism_scale_down() {
    let mut ap = AdaptiveParallelism::new(1, 8);

    let stats = RuntimeStatistics {
        rows_processed: 10,
        bytes_processed: 100,
        execution_time_us: 100,
        actual_cardinality: 10,
    };

    let workers = ap.adjust(&stats);
    assert!(workers <= 4);
}

#[test]
fn test_bloom_filter_runtime_pushdown() {
    let mut bf = BloomFilter::new(100, 0.01);
    for i in 0..100i32 {
        bf.insert(&i.to_le_bytes());
    }

    let mut filtered = 0;
    for i in 0..200i32 {
        if bf.might_contain(&i.to_le_bytes()) {
            filtered += 1;
        }
    }
    assert!(filtered >= 100);
    assert!(filtered < 200);
}
