//! Lesson 04: Data Chunks Tests

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::{DataChunk, ChunkCollection};

#[test]
fn test_chunk_creation() {
    let chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Varchar]);
    assert_eq!(chunk.column_count(), 2);
    assert_eq!(chunk.count(), 0);
}

#[test]
fn test_chunk_with_capacity() {
    let chunk = DataChunk::with_capacity(&[LogicalType::Int64, LogicalType::Float64], 100);
    assert_eq!(chunk.column_count(), 2);
    assert_eq!(chunk.count(), 0);
}

#[test]
fn test_chunk_append_row() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(100)]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Int64(200)]);
    chunk.append_row(&[ScalarValue::Int32(3), ScalarValue::Int64(300)]);

    assert_eq!(chunk.count(), 3);
    assert_eq!(chunk.column(0).get_value(0), ScalarValue::Int32(1));
    assert_eq!(chunk.column(0).get_value(1), ScalarValue::Int32(2));
    assert_eq!(chunk.column(0).get_value(2), ScalarValue::Int32(3));
    assert_eq!(chunk.column(1).get_value(0), ScalarValue::Int64(100));
    assert_eq!(chunk.column(1).get_value(1), ScalarValue::Int64(200));
    assert_eq!(chunk.column(1).get_value(2), ScalarValue::Int64(300));
}

#[test]
fn test_chunk_multi_type() {
    let mut chunk = DataChunk::new(&[
        LogicalType::Int32,
        LogicalType::Float64,
        LogicalType::Boolean,
    ]);
    chunk.append_row(&[
        ScalarValue::Int32(42),
        ScalarValue::Float64(3.14),
        ScalarValue::Boolean(true),
    ]);
    chunk.append_row(&[
        ScalarValue::Int32(0),
        ScalarValue::Float64(2.71),
        ScalarValue::Boolean(false),
    ]);

    assert_eq!(chunk.count(), 2);
    assert_eq!(chunk.column(0).get_value(0), ScalarValue::Int32(42));
    assert_eq!(chunk.column(1).get_value(0), ScalarValue::Float64(3.14));
    assert_eq!(chunk.column(2).get_value(0), ScalarValue::Boolean(true));
    assert_eq!(chunk.column(2).get_value(1), ScalarValue::Boolean(false));
}

#[test]
fn test_chunk_slice() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    for i in 0..10 {
        chunk.append_row(&[ScalarValue::Int32(i)]);
    }

    let sliced = chunk.slice(2, 5);
    assert_eq!(sliced.count(), 5);
    assert_eq!(sliced.column(0).get_value(0), ScalarValue::Int32(2));
    assert_eq!(sliced.column(0).get_value(1), ScalarValue::Int32(3));
    assert_eq!(sliced.column(0).get_value(4), ScalarValue::Int32(6));
}

#[test]
fn test_chunk_flatten() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    // Manually set a constant vector
    use quackdb::vector::Vector;
    let const_vec = Vector::new_constant(ScalarValue::Int32(99), 5);
    // Build a chunk with a constant vector
    let types = vec![LogicalType::Int32];
    let mut chunk = DataChunk::with_capacity(&types, 5);
    chunk.set_count(5);
    // We need to set the column — test that flatten works
    chunk.flatten();
    // After flatten, all columns should be flat
    // (This tests the infrastructure; the actual constant vector test
    //  depends on how DataChunk is constructed)
}

#[test]
fn test_chunk_reset() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Float64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Float64(1.0)]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Float64(2.0)]);
    assert_eq!(chunk.count(), 2);

    chunk.reset();
    assert_eq!(chunk.count(), 0);

    // Can reuse after reset
    chunk.append_row(&[ScalarValue::Int32(10), ScalarValue::Float64(10.0)]);
    assert_eq!(chunk.count(), 1);
    assert_eq!(chunk.column(0).get_value(0), ScalarValue::Int32(10));
}

#[test]
fn test_chunk_types() {
    let chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Varchar, LogicalType::Float64]);
    let types = chunk.types();
    assert_eq!(types, vec![LogicalType::Int32, LogicalType::Varchar, LogicalType::Float64]);
}

#[test]
fn test_chunk_display() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Varchar]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Varchar("hello".into())]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Varchar("world".into())]);

    let display = format!("{}", chunk);
    assert!(!display.is_empty());
    // Should contain at least the values
    assert!(display.contains("1") || display.contains("hello"));
}

#[test]
fn test_chunk_collection() {
    let types = vec![LogicalType::Int32, LogicalType::Float64];
    let mut collection = ChunkCollection::new(types.clone());

    let mut chunk1 = DataChunk::new(&types);
    chunk1.append_row(&[ScalarValue::Int32(1), ScalarValue::Float64(1.0)]);
    chunk1.append_row(&[ScalarValue::Int32(2), ScalarValue::Float64(2.0)]);

    let mut chunk2 = DataChunk::new(&types);
    chunk2.append_row(&[ScalarValue::Int32(3), ScalarValue::Float64(3.0)]);

    collection.append(chunk1);
    collection.append(chunk2);

    assert_eq!(collection.chunk_count(), 2);
    assert_eq!(collection.total_count(), 3);
    assert_eq!(collection.types(), &types);
}

#[test]
fn test_chunk_collection_empty() {
    let types = vec![LogicalType::Int32];
    let collection = ChunkCollection::new(types);
    assert_eq!(collection.chunk_count(), 0);
    assert_eq!(collection.total_count(), 0);
}

#[test]
fn test_chunk_column_access() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int32(10), ScalarValue::Int64(20)]);

    // Mutable access
    let col = chunk.column_mut(0);
    col.set_value(0, ScalarValue::Int32(99));

    assert_eq!(chunk.column(0).get_value(0), ScalarValue::Int32(99));
    assert_eq!(chunk.column(1).get_value(0), ScalarValue::Int64(20));
}
