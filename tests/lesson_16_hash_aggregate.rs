//! Lesson 16: Hash Aggregation Tests

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::execution::pipeline::*;
use quackdb::execution::expression::*;
use quackdb::execution::hash_aggregate::*;

fn make_agg_data() -> DataChunk {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    // group=1, value=10
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(10)]);
    // group=2, value=20
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Int64(20)]);
    // group=1, value=30
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(30)]);
    // group=2, value=40
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Int64(40)]);
    // group=1, value=50
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(50)]);
    chunk
}

#[test]
fn test_aggregate_sum() {
    let chunk = make_agg_data();
    let mut ht = AggregateHashTable::new(
        vec![LogicalType::Int32],
        vec![AggregateType::Sum],
        vec![LogicalType::Int64],
    );
    ht.add_chunk(&[0], &[1], &chunk).unwrap();

    let results = ht.finalize().unwrap();
    let total_rows: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total_rows, 2); // 2 groups

    // Group 1: sum = 10+30+50 = 90
    // Group 2: sum = 20+40 = 60
    // Order may vary, so collect all
    let mut group_sums: Vec<(i32, i64)> = Vec::new();
    for c in &results {
        for i in 0..c.count() {
            if let (ScalarValue::Int32(g), ScalarValue::Int64(s)) =
                (c.column(0).get_value(i), c.column(1).get_value(i))
            {
                group_sums.push((g, s));
            }
        }
    }
    group_sums.sort_by_key(|&(g, _)| g);
    assert_eq!(group_sums, vec![(1, 90), (2, 60)]);
}

#[test]
fn test_aggregate_count() {
    let chunk = make_agg_data();
    let mut ht = AggregateHashTable::new(
        vec![LogicalType::Int32],
        vec![AggregateType::Count],
        vec![LogicalType::Int64],
    );
    ht.add_chunk(&[0], &[1], &chunk).unwrap();

    let results = ht.finalize().unwrap();
    let mut group_counts: Vec<(i32, i64)> = Vec::new();
    for c in &results {
        for i in 0..c.count() {
            if let ScalarValue::Int32(g) = c.column(0).get_value(i) {
                // Count result could be Int64 or UInt64
                let count = match c.column(1).get_value(i) {
                    ScalarValue::Int64(v) => v,
                    ScalarValue::UInt64(v) => v as i64,
                    other => panic!("Unexpected count type: {:?}", other),
                };
                group_counts.push((g, count));
            }
        }
    }
    group_counts.sort_by_key(|&(g, _)| g);
    assert_eq!(group_counts, vec![(1, 3), (2, 2)]);
}

#[test]
fn test_aggregate_min_max() {
    let chunk = make_agg_data();
    let mut ht = AggregateHashTable::new(
        vec![LogicalType::Int32],
        vec![AggregateType::Min, AggregateType::Max],
        vec![LogicalType::Int64, LogicalType::Int64],
    );
    ht.add_chunk(&[0], &[1, 1], &chunk).unwrap();

    let results = ht.finalize().unwrap();
    let mut group_minmax: Vec<(i32, i64, i64)> = Vec::new();
    for c in &results {
        for i in 0..c.count() {
            if let (ScalarValue::Int32(g), ScalarValue::Int64(min_v), ScalarValue::Int64(max_v)) =
                (c.column(0).get_value(i), c.column(1).get_value(i), c.column(2).get_value(i))
            {
                group_minmax.push((g, min_v, max_v));
            }
        }
    }
    group_minmax.sort_by_key(|&(g, _, _)| g);
    assert_eq!(group_minmax, vec![(1, 10, 50), (2, 20, 40)]);
}

#[test]
fn test_aggregate_avg() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Float64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Float64(10.0)]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Float64(20.0)]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Float64(30.0)]);

    let mut ht = AggregateHashTable::new(
        vec![LogicalType::Int32],
        vec![AggregateType::Avg],
        vec![LogicalType::Float64],
    );
    ht.add_chunk(&[0], &[1], &chunk).unwrap();

    let results = ht.finalize().unwrap();
    for c in &results {
        for i in 0..c.count() {
            if let ScalarValue::Float64(avg) = c.column(1).get_value(i) {
                assert!((avg - 20.0).abs() < 0.001, "Expected avg=20.0, got {}", avg);
            }
        }
    }
}

#[test]
fn test_aggregate_global() {
    // No GROUP BY — global aggregation
    let mut chunk = DataChunk::new(&[LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int64(10)]);
    chunk.append_row(&[ScalarValue::Int64(20)]);
    chunk.append_row(&[ScalarValue::Int64(30)]);

    let mut ht = AggregateHashTable::new(
        vec![],  // no group by
        vec![AggregateType::Sum, AggregateType::Count],
        vec![LogicalType::Int64, LogicalType::Int64],
    );
    ht.add_chunk(&[], &[0, 0], &chunk).unwrap();

    let results = ht.finalize().unwrap();
    assert_eq!(ht.group_count(), 1); // one global group
}

#[test]
fn test_aggregate_empty() {
    let chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);

    let mut ht = AggregateHashTable::new(
        vec![LogicalType::Int32],
        vec![AggregateType::Sum],
        vec![LogicalType::Int64],
    );
    ht.add_chunk(&[0], &[1], &chunk).unwrap();

    let results = ht.finalize().unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 0);
}

#[test]
fn test_aggregate_hash_table_resize() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    for i in 0..1000 {
        chunk.append_row(&[ScalarValue::Int32(i), ScalarValue::Int64(i as i64 * 10)]);
    }

    let mut ht = AggregateHashTable::new(
        vec![LogicalType::Int32],
        vec![AggregateType::Sum],
        vec![LogicalType::Int64],
    );
    ht.add_chunk(&[0], &[1], &chunk).unwrap();

    assert_eq!(ht.group_count(), 1000);
}

#[test]
fn test_aggregate_with_nulls() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(10)]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Null(LogicalType::Int64)]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(30)]);

    let mut ht = AggregateHashTable::new(
        vec![LogicalType::Int32],
        vec![AggregateType::Sum],
        vec![LogicalType::Int64],
    );
    ht.add_chunk(&[0], &[1], &chunk).unwrap();

    let results = ht.finalize().unwrap();
    // SUM should skip nulls: 10 + 30 = 40
    for c in &results {
        for i in 0..c.count() {
            if let ScalarValue::Int64(s) = c.column(1).get_value(i) {
                assert_eq!(s, 40);
            }
        }
    }
}
