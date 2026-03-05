//! Lesson 17: Hash Join Tests

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::execution::hash_join::*;

fn make_build_side() -> DataChunk {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Varchar]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Varchar("alice".into())]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Varchar("bob".into())]);
    chunk.append_row(&[ScalarValue::Int32(3), ScalarValue::Varchar("charlie".into())]);
    chunk
}

fn make_probe_side() -> DataChunk {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(100)]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Int64(200)]);
    chunk.append_row(&[ScalarValue::Int32(4), ScalarValue::Int64(400)]); // no match
    chunk
}

#[test]
fn test_hash_join_inner() {
    let build = make_build_side();
    let probe = make_probe_side();

    let mut ht = JoinHashTable::new(vec![0], vec![LogicalType::Int32, LogicalType::Varchar]);
    ht.build(build).unwrap();

    let result = ht.probe(&probe, &[0], JoinType::Inner).unwrap();
    assert_eq!(result.count(), 2); // matches on id=1 and id=2
}

#[test]
fn test_hash_join_left() {
    let build = make_build_side();
    let probe = make_probe_side();

    let mut ht = JoinHashTable::new(vec![0], vec![LogicalType::Int32, LogicalType::Varchar]);
    ht.build(build).unwrap();

    let result = ht.probe(&probe, &[0], JoinType::Left).unwrap();
    assert_eq!(result.count(), 3); // all probe rows kept, id=4 has NULLs for build columns
}

#[test]
fn test_hash_join_right() {
    let build = make_build_side();
    let probe = make_probe_side();

    let mut ht = JoinHashTable::new(vec![0], vec![LogicalType::Int32, LogicalType::Varchar]);
    ht.build(build).unwrap();

    let result = ht.probe(&probe, &[0], JoinType::Right).unwrap();
    // All build rows kept, id=3 has NULLs for probe columns
    assert_eq!(result.count(), 3);
}

#[test]
fn test_hash_join_full() {
    let build = make_build_side();
    let probe = make_probe_side();

    let mut ht = JoinHashTable::new(vec![0], vec![LogicalType::Int32, LogicalType::Varchar]);
    ht.build(build).unwrap();

    let result = ht.probe(&probe, &[0], JoinType::Full).unwrap();
    // 2 matches + 1 unmatched build (id=3) + 1 unmatched probe (id=4) = 4
    assert_eq!(result.count(), 4);
}

#[test]
fn test_hash_join_semi() {
    let build = make_build_side();
    let probe = make_probe_side();

    let mut ht = JoinHashTable::new(vec![0], vec![LogicalType::Int32, LogicalType::Varchar]);
    ht.build(build).unwrap();

    let result = ht.probe(&probe, &[0], JoinType::Semi).unwrap();
    // Only probe rows that have a match: id=1, id=2
    assert_eq!(result.count(), 2);
    // Semi join only returns probe columns
    assert_eq!(result.column_count(), 2); // probe has 2 columns
}

#[test]
fn test_hash_join_anti() {
    let build = make_build_side();
    let probe = make_probe_side();

    let mut ht = JoinHashTable::new(vec![0], vec![LogicalType::Int32, LogicalType::Varchar]);
    ht.build(build).unwrap();

    let result = ht.probe(&probe, &[0], JoinType::Anti).unwrap();
    // Only probe rows with NO match: id=4
    assert_eq!(result.count(), 1);
}

#[test]
fn test_hash_join_multi_key() {
    let mut build = DataChunk::new(&[LogicalType::Int32, LogicalType::Int32, LogicalType::Varchar]);
    build.append_row(&[ScalarValue::Int32(1), ScalarValue::Int32(10), ScalarValue::Varchar("a".into())]);
    build.append_row(&[ScalarValue::Int32(1), ScalarValue::Int32(20), ScalarValue::Varchar("b".into())]);

    let mut probe = DataChunk::new(&[LogicalType::Int32, LogicalType::Int32, LogicalType::Float64]);
    probe.append_row(&[ScalarValue::Int32(1), ScalarValue::Int32(10), ScalarValue::Float64(1.0)]);
    probe.append_row(&[ScalarValue::Int32(1), ScalarValue::Int32(30), ScalarValue::Float64(3.0)]);

    let mut ht = JoinHashTable::new(vec![0, 1], vec![LogicalType::Int32, LogicalType::Int32, LogicalType::Varchar]);
    ht.build(build).unwrap();

    let result = ht.probe(&probe, &[0, 1], JoinType::Inner).unwrap();
    assert_eq!(result.count(), 1); // only (1,10) matches
}

#[test]
fn test_hash_join_duplicates() {
    let mut build = DataChunk::new(&[LogicalType::Int32, LogicalType::Varchar]);
    build.append_row(&[ScalarValue::Int32(1), ScalarValue::Varchar("a".into())]);
    build.append_row(&[ScalarValue::Int32(1), ScalarValue::Varchar("b".into())]);

    let mut probe = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    probe.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(100)]);

    let mut ht = JoinHashTable::new(vec![0], vec![LogicalType::Int32, LogicalType::Varchar]);
    ht.build(build).unwrap();

    let result = ht.probe(&probe, &[0], JoinType::Inner).unwrap();
    assert_eq!(result.count(), 2); // one probe row matches two build rows
}

#[test]
fn test_hash_join_empty_build() {
    let build = DataChunk::new(&[LogicalType::Int32, LogicalType::Varchar]);
    let probe = make_probe_side();

    let mut ht = JoinHashTable::new(vec![0], vec![LogicalType::Int32, LogicalType::Varchar]);
    ht.build(build).unwrap();

    let result = ht.probe(&probe, &[0], JoinType::Inner).unwrap();
    assert_eq!(result.count(), 0);
}

#[test]
fn test_hash_join_empty_probe() {
    let build = make_build_side();
    let probe = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);

    let mut ht = JoinHashTable::new(vec![0], vec![LogicalType::Int32, LogicalType::Varchar]);
    ht.build(build).unwrap();

    let result = ht.probe(&probe, &[0], JoinType::Inner).unwrap();
    assert_eq!(result.count(), 0);
}

#[test]
fn test_hash_join_build_row_count() {
    let build = make_build_side();
    let mut ht = JoinHashTable::new(vec![0], vec![LogicalType::Int32, LogicalType::Varchar]);
    ht.build(build).unwrap();
    assert_eq!(ht.build_row_count(), 3);
}
