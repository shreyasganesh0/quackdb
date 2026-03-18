//! # Lesson 12: Columnar File Reader — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Basic reader open and metadata (`test_reader_open`)
//! 2. Full scan (`test_reader_scan_all`)
//! 3. Edge cases (empty file)
//! 4. Predicate operations (`test_predicate_ops`)
//! 5. Projection pushdown (`test_reader_projection`)
//! 6. Write-read roundtrip (`test_reader_write_read_roundtrip`)
//! 7. Row group pruning — integration (`test_reader_row_group_pruning`)

use quackdb::storage::columnar_file::*;
use quackdb::storage::reader::*;
use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use std::io::Cursor;

/// Helper: write a test file with the given (i32, i64) rows and return the raw bytes.
fn write_test_file(rows: &[(i32, i64)]) -> Vec<u8> {
    let schema = vec![
        ("id".to_string(), LogicalType::Int32),
        ("value".to_string(), LogicalType::Int64),
    ];
    let mut buf = Vec::new();

    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    for &(id, val) in rows {
        chunk.append_row(&[ScalarValue::Int32(id), ScalarValue::Int64(val)]);
    }

    let mut writer = ColumnarFileWriter::new(Cursor::new(&mut buf), schema).unwrap();
    writer.write_chunk(&chunk).unwrap();
    writer.finish().unwrap();
    buf
}

// ── 1. Basic reader open and metadata ───────────────────────────────

#[test]
fn test_reader_open() {
    let buf = write_test_file(&[(1, 100), (2, 200), (3, 300)]);
    let reader = ColumnarFileReader::open(Cursor::new(&buf)).unwrap();

    assert_eq!(reader.total_rows(), 3, "reader must parse total_rows from the file footer");
    assert_eq!(reader.schema().len(), 2, "schema with two columns must be recovered from footer metadata");
    assert_eq!(reader.row_group_count(), 1, "single write_chunk call produces exactly one row group");
}

// ── 2. Full scan ────────────────────────────────────────────────────

#[test]
fn test_reader_scan_all() {
    let buf = write_test_file(&[(1, 100), (2, 200), (3, 300)]);
    let mut reader = ColumnarFileReader::open(Cursor::new(&buf)).unwrap();

    let chunks = reader.scan(None, &[]).unwrap();
    let total_rows: usize = chunks.iter().map(|c| c.count()).sum();
    assert_eq!(total_rows, 3, "full scan with no predicates must return all rows");
}

// ── 3. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_reader_empty_file() {
    let schema = vec![("x".to_string(), LogicalType::Int32)];
    let mut buf = Vec::new();
    let writer = ColumnarFileWriter::new(Cursor::new(&mut buf), schema).unwrap();
    writer.finish().unwrap();

    let mut reader = ColumnarFileReader::open(Cursor::new(&buf)).unwrap();
    assert_eq!(reader.total_rows(), 0, "empty file must report zero total rows");
    let chunks = reader.scan(None, &[]).unwrap();
    assert!(chunks.is_empty() || chunks.iter().all(|c| c.count() == 0), "scanning an empty file must yield no data");
}

#[test]
fn test_reader_single_row() {
    // Edge case: file with exactly one row
    let buf = write_test_file(&[(42, 999)]);
    let mut reader = ColumnarFileReader::open(Cursor::new(&buf)).unwrap();
    assert_eq!(reader.total_rows(), 1, "single-row file must report total_rows = 1");

    let chunks = reader.scan(None, &[]).unwrap();
    let total: usize = chunks.iter().map(|c| c.count()).sum();
    assert_eq!(total, 1, "scanning a single-row file must return exactly one row");
}

// ── 4. Predicate operations ────────────────────────────────────────

#[test]
fn test_predicate_ops() {
    let stats = ColumnStats {
        null_count: 0,
        min_value: Some(10i32.to_le_bytes().to_vec()),
        max_value: Some(100i32.to_le_bytes().to_vec()),
        distinct_count: None,
    };

    // GT 200 should prune (max is 100)
    let pred = ScanPredicate {
        column_index: 0,
        op: PredicateOp::Gt,
        value: ScalarValue::Int32(200),
    };
    assert!(pred.can_prune(&stats, &LogicalType::Int32), "GT 200 must prune: max is 100, so no row can satisfy > 200");

    // GT 50 should NOT prune (max is 100, could have values > 50)
    let pred = ScanPredicate {
        column_index: 0,
        op: PredicateOp::Gt,
        value: ScalarValue::Int32(50),
    };
    assert!(!pred.can_prune(&stats, &LogicalType::Int32), "GT 50 must NOT prune: values up to 100 exist");

    // LT 5 should prune (min is 10)
    let pred = ScanPredicate {
        column_index: 0,
        op: PredicateOp::Lt,
        value: ScalarValue::Int32(5),
    };
    assert!(pred.can_prune(&stats, &LogicalType::Int32), "LT 5 must prune: min is 10, so no row can be < 5");
}

#[test]
fn test_predicate_equal_boundary() {
    // Edge case: predicate value equals exactly the min or max
    let stats = ColumnStats {
        null_count: 0,
        min_value: Some(10i32.to_le_bytes().to_vec()),
        max_value: Some(100i32.to_le_bytes().to_vec()),
        distinct_count: None,
    };

    // GT 100 should prune (max is exactly 100, nothing can be > 100)
    let pred = ScanPredicate {
        column_index: 0,
        op: PredicateOp::Gt,
        value: ScalarValue::Int32(100),
    };
    assert!(pred.can_prune(&stats, &LogicalType::Int32), "GT max value must prune since no row can exceed the max");
}

// ── 5. Projection pushdown ─────────────────────────────────────────

#[test]
fn test_reader_projection() {
    let buf = write_test_file(&[(1, 100), (2, 200)]);
    let mut reader = ColumnarFileReader::open(Cursor::new(&buf)).unwrap();

    // Only read column 0 (id)
    let chunks = reader.scan(Some(&[0]), &[]).unwrap();
    assert!(!chunks.is_empty());
    let chunk = &chunks[0];
    assert_eq!(chunk.column_count(), 1, "projection pushdown must return only the requested columns");
}

// ── 6. Write-read roundtrip ────────────────────────────────────────

#[test]
fn test_reader_write_read_roundtrip() {
    let original = vec![(10, 1000i64), (20, 2000), (30, 3000)];
    let buf = write_test_file(&original);
    let mut reader = ColumnarFileReader::open(Cursor::new(&buf)).unwrap();

    let chunks = reader.scan(None, &[]).unwrap();
    assert_eq!(chunks.len(), 1, "one row group should produce one chunk");
    let chunk = &chunks[0];
    assert_eq!(chunk.count(), 3, "round-tripped chunk must contain all original rows");

    for (i, &(expected_id, expected_val)) in original.iter().enumerate() {
        assert_eq!(chunk.column(0).get_value(i), ScalarValue::Int32(expected_id));
        assert_eq!(chunk.column(1).get_value(i), ScalarValue::Int64(expected_val));
    }
}

// ── 7. Row group pruning — integration ──────────────────────────────

#[test]
fn test_reader_row_group_pruning() {
    // Create a file with multiple row groups that can be pruned
    let schema = vec![("id".to_string(), LogicalType::Int32)];
    let mut buf = Vec::new();

    let mut writer = ColumnarFileWriter::new(Cursor::new(&mut buf), schema).unwrap();

    // Row group 1: ids 1-100
    let mut chunk1 = DataChunk::new(&[LogicalType::Int32]);
    for i in 1..=100 {
        chunk1.append_row(&[ScalarValue::Int32(i)]);
    }
    writer.write_chunk(&chunk1).unwrap();

    // Row group 2: ids 101-200
    let mut chunk2 = DataChunk::new(&[LogicalType::Int32]);
    for i in 101..=200 {
        chunk2.append_row(&[ScalarValue::Int32(i)]);
    }
    writer.write_chunk(&chunk2).unwrap();

    writer.finish().unwrap();

    let mut reader = ColumnarFileReader::open(Cursor::new(&buf)).unwrap();

    // Predicate: id > 150 — should prune the first row group
    let predicates = vec![ScanPredicate {
        column_index: 0,
        op: PredicateOp::Gt,
        value: ScalarValue::Int32(150),
    }];

    let chunks = reader.scan(None, &predicates).unwrap();
    // Should only return data from row group 2
    let total: usize = chunks.iter().map(|c| c.count()).sum();
    assert!(total <= 100, "Pruning should skip first row group");
}
