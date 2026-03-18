//! # Lesson 11: Columnar File Writer — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Magic bytes and format identification (`test_magic_bytes`)
//! 2. Single-column write (`test_columnar_write_single_column`)
//! 3. Multi-column write (`test_columnar_write_multi_column`)
//! 4. Edge cases (empty file, stats with nulls)
//! 5. Column stats (`test_column_stats_update`, `test_column_stats_merge`)
//! 6. Multiple row groups (`test_columnar_write_multiple_row_groups`)
//! 7. DataChunk-based write (`test_columnar_write_chunk`)
//! 8. Footer serialization (`test_footer_serialize_roundtrip`)

use quackdb::storage::columnar_file::*;
use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use std::io::Cursor;

// ── 1. Magic bytes ──────────────────────────────────────────────────

#[test]
fn test_magic_bytes() {
    assert_eq!(MAGIC, b"QUAK", "magic bytes uniquely identify the file format and prevent misinterpretation");
}

// ── 2. Single-column write ──────────────────────────────────────────

#[test]
fn test_columnar_write_single_column() {
    let schema = vec![("id".to_string(), LogicalType::Int32)];
    let mut buf = Vec::new();
    let mut writer = ColumnarFileWriter::new(Cursor::new(&mut buf), schema).unwrap();

    writer.begin_row_group().unwrap();
    writer.write_column(0, &[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0], 3, ColumnStats::new()).unwrap();
    writer.end_row_group(3).unwrap();
    let _ = writer.finish().unwrap();

    // Check magic bytes at start
    assert_eq!(&buf[..4], MAGIC, "file must start with magic bytes for format identification");
}

// ── 3. Multi-column write ───────────────────────────────────────────

#[test]
fn test_columnar_write_multi_column() {
    let schema = vec![
        ("id".to_string(), LogicalType::Int32),
        ("value".to_string(), LogicalType::Float64),
    ];
    let mut buf = Vec::new();
    let mut writer = ColumnarFileWriter::new(Cursor::new(&mut buf), schema).unwrap();

    writer.begin_row_group().unwrap();
    writer.write_column(0, &[1, 0, 0, 0, 2, 0, 0, 0], 2, ColumnStats::new()).unwrap();
    writer.write_column(1, &[0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 64], 2, ColumnStats::new()).unwrap();
    writer.end_row_group(2).unwrap();
    writer.finish().unwrap();

    assert!(!buf.is_empty(), "multi-column write must produce output containing header, data, and footer");
}

// ── 4. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_columnar_write_empty_file() {
    // Edge case: writer with schema but no row groups written
    let schema = vec![("x".to_string(), LogicalType::Int32)];
    let mut buf = Vec::new();
    let writer = ColumnarFileWriter::new(Cursor::new(&mut buf), schema).unwrap();
    writer.finish().unwrap();
    assert!(!buf.is_empty(), "even an empty file must contain magic bytes and a footer");
}

#[test]
fn test_columnar_write_with_stats() {
    let schema = vec![("id".to_string(), LogicalType::Int32)];
    let mut buf = Vec::new();
    let mut writer = ColumnarFileWriter::new(Cursor::new(&mut buf), schema).unwrap();

    let mut stats = ColumnStats::new();
    stats.null_count = 0;
    stats.min_value = Some(vec![1, 0, 0, 0]);
    stats.max_value = Some(vec![10, 0, 0, 0]);

    writer.begin_row_group().unwrap();
    writer.write_column(0, &[1, 0, 0, 0], 1, stats).unwrap();
    writer.end_row_group(1).unwrap();
    writer.finish().unwrap();
}

// ── 5. Column stats ────────────────────────────────────────────────

#[test]
fn test_column_stats_update() {
    let mut stats = ColumnStats::new();
    stats.update(&[5, 0, 0, 0], false);
    stats.update(&[10, 0, 0, 0], false);
    stats.update(&[1, 0, 0, 0], false);
    stats.update(&[], true); // null

    assert_eq!(stats.null_count, 1, "stats must accurately track null values for query optimization");
}

#[test]
fn test_column_stats_merge() {
    let mut s1 = ColumnStats::new();
    s1.null_count = 2;
    s1.min_value = Some(vec![1, 0]);
    s1.max_value = Some(vec![10, 0]);

    let mut s2 = ColumnStats::new();
    s2.null_count = 3;
    s2.min_value = Some(vec![0, 0]);
    s2.max_value = Some(vec![20, 0]);

    s1.merge(&s2);
    assert_eq!(s1.null_count, 5, "merging stats must sum null counts across row groups");
}

#[test]
fn test_column_stats_all_nulls() {
    // Edge case: all values are null — stats should track the null count correctly
    let mut stats = ColumnStats::new();
    for _ in 0..5 {
        stats.update(&[], true);
    }
    assert_eq!(stats.null_count, 5, "stats must count all nulls when every value is null");
}

// ── 6. Multiple row groups ──────────────────────────────────────────

#[test]
fn test_columnar_write_multiple_row_groups() {
    let schema = vec![("x".to_string(), LogicalType::Int64)];
    let mut buf = Vec::new();
    let mut writer = ColumnarFileWriter::new(Cursor::new(&mut buf), schema).unwrap();

    for _ in 0..3 {
        writer.begin_row_group().unwrap();
        writer.write_column(0, &[42, 0, 0, 0, 0, 0, 0, 0], 1, ColumnStats::new()).unwrap();
        writer.end_row_group(1).unwrap();
    }
    writer.finish().unwrap();
}

// ── 7. DataChunk-based write ────────────────────────────────────────

#[test]
fn test_columnar_write_chunk() {
    let schema = vec![
        ("a".to_string(), LogicalType::Int32),
        ("b".to_string(), LogicalType::Int64),
    ];
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(100)]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Int64(200)]);

    let mut buf = Vec::new();
    let mut writer = ColumnarFileWriter::new(Cursor::new(&mut buf), schema).unwrap();
    writer.write_chunk(&chunk).unwrap();
    writer.finish().unwrap();

    assert!(&buf[..4] == MAGIC, "DataChunk-based write must also produce a valid file header");
}

// ── 7b. DataChunk-based write with nulls ──────────────────────────

#[test]
fn test_columnar_write_chunk_single_row() {
    // Edge case: writing a chunk with exactly one row
    let schema = vec![("x".to_string(), LogicalType::Int32)];
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(99)]);

    let mut buf = Vec::new();
    let mut writer = ColumnarFileWriter::new(Cursor::new(&mut buf), schema).unwrap();
    writer.write_chunk(&chunk).unwrap();
    writer.finish().unwrap();

    assert!(&buf[..4] == MAGIC, "single-row chunk write must produce a valid file");
}

#[test]
fn test_column_stats_update_non_null_only() {
    // Edge case: stats with no nulls at all should have null_count == 0
    let mut stats = ColumnStats::new();
    stats.update(&[1, 0, 0, 0], false);
    stats.update(&[2, 0, 0, 0], false);
    stats.update(&[3, 0, 0, 0], false);
    assert_eq!(stats.null_count, 0, "stats with only non-null values must report null_count of 0");
}

// ── 8. Footer serialization ────────────────────────────────────────

#[test]
fn test_footer_serialize_roundtrip() {
    let footer = FileFooter {
        schema: vec![
            ("id".to_string(), LogicalType::Int32),
            ("name".to_string(), LogicalType::Varchar),
        ],
        row_groups: vec![RowGroupMeta {
            num_rows: 100,
            columns: vec![ColumnChunkMeta {
                column_index: 0,
                logical_type: LogicalType::Int32,
                offset: 4,
                size: 400,
                num_values: 100,
                stats: ColumnStats::new(),
                compression: 0,
            }],
        }],
        total_rows: 100,
    };

    let bytes = footer.to_bytes();
    let restored = FileFooter::from_bytes(&bytes).unwrap();
    assert_eq!(restored.schema.len(), 2, "footer must preserve the full schema for readers");
    assert_eq!(restored.total_rows, 100, "total_rows in footer enables row-count queries without scanning");
    assert_eq!(restored.row_groups.len(), 1, "row group metadata must survive footer serialization");
}
