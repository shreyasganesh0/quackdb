//! # Lesson 18: Sort-Merge Join — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Row comparator basics (`test_row_comparator`)
//! 2. Comparator — descending and nulls (`test_row_comparator_descending`, `test_row_comparator_nulls_*`)
//! 3. Key normalization (`test_key_normalizer`)
//! 4. Edge cases (empty sides, single-row inputs)
//! 5. Inner merge join (`test_merge_join_inner`)
//! 6. Duplicate handling (`test_merge_join_duplicates`)
//! 7. Left outer merge join (`test_merge_join_left_outer`)

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::execution::sort_merge_join::*;
use quackdb::execution::hash_join::JoinType;
use std::cmp::Ordering;

fn make_sorted_left() -> DataChunk {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Varchar]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Varchar("alice".into())]);
    chunk.append_row(&[ScalarValue::Int32(3), ScalarValue::Varchar("charlie".into())]);
    chunk.append_row(&[ScalarValue::Int32(5), ScalarValue::Varchar("eve".into())]);
    chunk
}

fn make_sorted_right() -> DataChunk {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(100)]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Int64(200)]);
    chunk.append_row(&[ScalarValue::Int32(3), ScalarValue::Int64(300)]);
    chunk.append_row(&[ScalarValue::Int32(4), ScalarValue::Int64(400)]);
    chunk
}

// ── 1. Row comparator basics ────────────────────────────────────────

#[test]
fn test_row_comparator() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(200)]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(100)]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Int64(50)]);

    let cmp = RowComparator::new(vec![
        SortKey { column_index: 0, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast },
        SortKey { column_index: 1, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast },
    ]);

    assert_eq!(cmp.compare_within(&chunk, 0, 1), Ordering::Greater, "when first keys tie, comparator falls through to second key: 200 > 100");
    assert_eq!(cmp.compare_within(&chunk, 0, 2), Ordering::Less, "first key takes priority: 1 < 2 regardless of second key values");
    assert_eq!(cmp.compare_within(&chunk, 1, 0), Ordering::Less);    // (1,100) < (1,200)
}

// ── 2. Comparator — descending and nulls ────────────────────────────

#[test]
fn test_row_comparator_descending() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(10)]);
    chunk.append_row(&[ScalarValue::Int32(20)]);

    let cmp = RowComparator::new(vec![
        SortKey { column_index: 0, direction: SortDirection::Descending, null_order: NullOrder::NullsLast },
    ]);

    assert_eq!(cmp.compare_within(&chunk, 0, 1), Ordering::Greater, "descending order flips the comparison: 10 is 'greater' because it comes first in DESC");
}

#[test]
fn test_row_comparator_nulls_first() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(10)]);
    chunk.append_row(&[ScalarValue::Null(LogicalType::Int32)]);

    let cmp = RowComparator::new(vec![
        SortKey { column_index: 0, direction: SortDirection::Ascending, null_order: NullOrder::NullsFirst },
    ]);

    assert_eq!(cmp.compare_within(&chunk, 1, 0), Ordering::Less, "NULLS FIRST places NULL before all non-null values in sort order");
}

#[test]
fn test_row_comparator_nulls_last() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(10)]);
    chunk.append_row(&[ScalarValue::Null(LogicalType::Int32)]);

    let cmp = RowComparator::new(vec![
        SortKey { column_index: 0, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast },
    ]);

    assert_eq!(cmp.compare_within(&chunk, 1, 0), Ordering::Greater, "NULLS LAST places NULL after all non-null values in sort order");
}

// ── 3. Key normalization ────────────────────────────────────────────

#[test]
fn test_key_normalizer() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(100)]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Int64(50)]);

    let keys = vec![
        SortKey { column_index: 0, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast },
        SortKey { column_index: 1, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast },
    ];

    let k1 = KeyNormalizer::normalize(&chunk, 0, &keys);
    let k2 = KeyNormalizer::normalize(&chunk, 1, &keys);

    // k1 should be < k2 (row 0 has smaller first key)
    assert!(k1 < k2, "normalized keys enable byte-wise comparison that preserves sort order across types");
}

// ── 4. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_row_comparator_equal_rows() {
    // Edge case: comparing a row with itself should return Equal
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(42)]);

    let cmp = RowComparator::new(vec![
        SortKey { column_index: 0, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast },
    ]);

    assert_eq!(cmp.compare_within(&chunk, 0, 0), Ordering::Equal, "comparing a row with itself must return Equal");
}

// ── 5. Inner merge join ─────────────────────────────────────────────

#[test]
fn test_merge_join_inner() {
    let left = make_sorted_left();
    let right = make_sorted_right();

    let left_keys = vec![SortKey { column_index: 0, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast }];
    let right_keys = vec![SortKey { column_index: 0, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast }];

    let mut join = MergeJoinOperator::new(
        JoinType::Inner,
        left_keys,
        right_keys,
        vec![LogicalType::Int32, LogicalType::Varchar],
        vec![LogicalType::Int32, LogicalType::Int64],
    );
    join.add_left(left);
    join.add_right(right);

    let results = join.merge().unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 2, "sort-merge join finds matches by advancing two sorted cursors in lockstep");
}

// ── 6. Duplicate handling ───────────────────────────────────────────

#[test]
fn test_merge_join_duplicates() {
    let mut left = DataChunk::new(&[LogicalType::Int32]);
    left.append_row(&[ScalarValue::Int32(1)]);
    left.append_row(&[ScalarValue::Int32(1)]);
    left.append_row(&[ScalarValue::Int32(2)]);

    let mut right = DataChunk::new(&[LogicalType::Int32]);
    right.append_row(&[ScalarValue::Int32(1)]);
    right.append_row(&[ScalarValue::Int32(1)]);

    let keys = vec![SortKey { column_index: 0, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast }];

    let mut join = MergeJoinOperator::new(
        JoinType::Inner,
        keys.clone(),
        keys,
        vec![LogicalType::Int32],
        vec![LogicalType::Int32],
    );
    join.add_left(left);
    join.add_right(right);

    let results = join.merge().unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 4, "duplicate keys produce a cross product: 2 left * 2 right = 4 output rows for key=1");
}

// ── 7. Left outer merge join ────────────────────────────────────────

#[test]
fn test_merge_join_left_outer() {
    let left = make_sorted_left();
    let right = make_sorted_right();

    let keys = vec![SortKey { column_index: 0, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast }];

    let mut join = MergeJoinOperator::new(
        JoinType::Left,
        keys.clone(),
        keys,
        vec![LogicalType::Int32, LogicalType::Varchar],
        vec![LogicalType::Int32, LogicalType::Int64],
    );
    join.add_left(left);
    join.add_right(right);

    let results = join.merge().unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 3, "left outer merge join preserves all left rows, emitting NULLs for unmatched right columns");
}
