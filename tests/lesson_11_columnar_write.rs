//! Lesson 11: Columnar File Writer Tests

use quackdb::storage::columnar_file::*;
use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use std::io::Cursor;

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

#[test]
fn test_magic_bytes() {
    assert_eq!(MAGIC, b"QUAK", "magic bytes uniquely identify the file format and prevent misinterpretation");
}
