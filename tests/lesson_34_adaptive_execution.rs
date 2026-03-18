//! # Lesson 34: Adaptive Query Execution — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Bloom filter basics (`test_bloom_filter_basic`)
//! 2. Bloom filter — empty filter (`test_bloom_filter_empty`)
//! 3. Runtime statistics defaults (`test_runtime_statistics`)
//! 4. Edge cases (single-element bloom filter, false positive rate)
//! 5. Bloom filter false positive rate (`test_bloom_filter_false_positive_rate`)
//! 6. Adaptive parallelism (`test_adaptive_parallelism`, `test_adaptive_parallelism_scale_down`)
//! 7. Adaptive join operator (`test_adaptive_join_operator`)
//! 8. Bloom filter runtime pushdown (`test_bloom_filter_runtime_pushdown`)

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::execution::adaptive::*;
use quackdb::execution::pipeline::*;

// ── 1. Bloom filter basics ──────────────────────────────────────────

#[test]
fn test_bloom_filter_basic() {
    let mut bf = BloomFilter::new(1000, 0.01);
    bf.insert(b"hello");
    bf.insert(b"world");

    assert!(bf.might_contain(b"hello"), "bloom filter must never produce false negatives for inserted elements");
    assert!(bf.might_contain(b"world"), "bloom filter must never produce false negatives for inserted elements");
}

// ── 2. Bloom filter — empty ─────────────────────────────────────────

#[test]
fn test_bloom_filter_empty() {
    let bf = BloomFilter::new(100, 0.01);
    assert!(!bf.might_contain(b"anything"), "an empty bloom filter should never report that it might contain an element");
}

// ── 3. Runtime statistics defaults ──────────────────────────────────

#[test]
fn test_runtime_statistics() {
    let stats = RuntimeStatistics::default();
    assert_eq!(stats.rows_processed, 0, "RuntimeStatistics::default() must zero-initialize all counters so they accumulate from a clean baseline");
    assert_eq!(stats.bytes_processed, 0);
}

// ── 4. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_bloom_filter_single_element() {
    // Edge case: bloom filter with exactly one element
    let mut bf = BloomFilter::new(100, 0.01);
    bf.insert(b"only");
    assert!(bf.might_contain(b"only"), "the single inserted element must always be found");
}

#[test]
fn test_adaptive_parallelism_bounds() {
    // Edge case: min and max workers equal
    let mut ap = AdaptiveParallelism::new(4, 4);
    assert_eq!(ap.current_workers(), 4, "when min == max, workers should be fixed at that value");

    let stats = RuntimeStatistics {
        rows_processed: 1000000,
        bytes_processed: 1000000,
        execution_time_us: 1000,
        actual_cardinality: 1000000,
    };
    let workers = ap.adjust(&stats);
    assert_eq!(workers, 4, "when min == max, adjust should always return the fixed worker count");
}

// ── 5. Bloom filter false positive rate ─────────────────────────────

#[test]
fn test_bloom_filter_false_positive_rate() {
    let mut bf = BloomFilter::new(1000, 0.01);
    for i in 0..1000u32 {
        bf.insert(&i.to_le_bytes());
    }

    // Verify no false negatives for inserted elements
    for i in 0..1000u32 {
        assert!(bf.might_contain(&i.to_le_bytes()));
    }

    // Measure false positive rate on non-inserted elements
    let mut false_positives = 0;
    for i in 1000..2000u32 {
        if bf.might_contain(&i.to_le_bytes()) {
            false_positives += 1;
        }
    }
    let fp_rate = false_positives as f64 / 1000.0;
    assert!(fp_rate < 0.05, "bloom filter false positive rate should stay under 5% for a correctly sized filter, got {:.2}%", fp_rate * 100.0);
}

// ── 6. Adaptive parallelism ─────────────────────────────────────────

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
    assert!(workers >= 1 && workers <= 8, "adaptive parallelism must stay within the configured min/max worker bounds");
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
    assert!(workers <= 4, "small workloads should scale down parallelism to avoid thread overhead exceeding the work itself");
}

// ── 7. Adaptive join operator ───────────────────────────────────────

#[test]
fn test_adaptive_join_operator() {
    let mut op = AdaptiveJoinOperator::new(
        vec![LogicalType::Int32, LogicalType::Int64],
        10000,
    );

    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(100)]);

    let _ = op.execute(&chunk);
    // The adaptive join operator should accept input without panicking, even for small chunks below the bloom filter threshold
}

// ── 7b. Bloom filter edge cases ──────────────────────────────────────

#[test]
fn test_bloom_filter_large_values() {
    // Edge case: bloom filter with large byte values
    let mut bf = BloomFilter::new(100, 0.01);
    let large_key = vec![0xFFu8; 256];
    bf.insert(&large_key);
    assert!(bf.might_contain(&large_key), "bloom filter must handle large keys without panicking or producing false negatives");
}

#[test]
fn test_runtime_statistics_fields() {
    // Edge case: verify all fields of RuntimeStatistics can be set and read
    let stats = RuntimeStatistics {
        rows_processed: 42,
        bytes_processed: 1024,
        execution_time_us: 500,
        actual_cardinality: 42,
    };
    assert_eq!(stats.rows_processed, 42, "RuntimeStatistics fields must be readable after construction");
    assert_eq!(stats.execution_time_us, 500);
}

#[test]
fn test_adaptive_parallelism_multiple_adjustments() {
    // Edge case: multiple successive adjustments should converge
    let mut ap = AdaptiveParallelism::new(1, 8);
    let high_stats = RuntimeStatistics {
        rows_processed: 1000000,
        bytes_processed: 1000000,
        execution_time_us: 1000,
        actual_cardinality: 1000000,
    };
    // Call adjust multiple times to test stability
    let w1 = ap.adjust(&high_stats);
    let w2 = ap.adjust(&high_stats);
    assert!(w1 >= 1 && w1 <= 8, "first adjustment must stay in bounds");
    assert!(w2 >= 1 && w2 <= 8, "repeated adjustments must stay in bounds");
}

// ── 8. Bloom filter runtime pushdown ────────────────────────────────

#[test]
fn test_bloom_filter_runtime_pushdown() {
    // Simulates using a bloom filter built from the build side to filter probe rows at runtime
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
    assert!(filtered >= 100, "all 100 inserted values must pass the bloom filter (no false negatives allowed)");
    assert!(filtered < 200, "bloom filter should reject some of the 100 non-inserted values, reducing probe work at runtime");
}
