//! # Lesson 19: External Sort — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. MinHeap basics (`test_min_heap`, `test_min_heap_single`)
//! 2. In-memory sort ascending (`test_sort_in_memory`)
//! 3. In-memory sort descending (`test_sort_descending`)
//! 4. Edge cases (sort with nulls, single element, empty)
//! 5. Multi-column sort (`test_sort_multi_column`)
//! 6. String sort (`test_sort_strings`)
//! 7. K-way merge (`test_k_way_merge`)
//! 8. Pipeline integration (`test_external_sort_pipeline`)
//! 9. Top-N optimization (`test_top_n`)

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::execution::sort::*;
use quackdb::execution::sort_merge_join::{SortKey, SortDirection, NullOrder};
use quackdb::execution::pipeline::*;

fn make_unsorted_chunk() -> DataChunk {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Varchar]);
    chunk.append_row(&[ScalarValue::Int32(5), ScalarValue::Varchar("eve".into())]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Varchar("bob".into())]);
    chunk.append_row(&[ScalarValue::Int32(8), ScalarValue::Varchar("hank".into())]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Varchar("alice".into())]);
    chunk.append_row(&[ScalarValue::Int32(4), ScalarValue::Varchar("dave".into())]);
    chunk
}

// ── 1. MinHeap basics ───────────────────────────────────────────────

#[test]
fn test_min_heap() {
    let mut heap = MinHeap::new(|a: &i32, b: &i32| a.cmp(b));
    heap.push(5);
    heap.push(2);
    heap.push(8);
    heap.push(1);
    heap.push(4);

    assert_eq!(heap.len(), 5);
    assert_eq!(heap.pop(), Some(1), "min-heap always extracts the smallest element first");
    assert_eq!(heap.pop(), Some(2));
    assert_eq!(heap.pop(), Some(4));
    assert_eq!(heap.pop(), Some(5));
    assert_eq!(heap.pop(), Some(8));
    assert_eq!(heap.pop(), None);
}

#[test]
fn test_min_heap_single() {
    let mut heap = MinHeap::new(|a: &i32, b: &i32| a.cmp(b));
    heap.push(42);
    assert_eq!(heap.peek(), Some(&42));
    assert_eq!(heap.pop(), Some(42));
    assert!(heap.is_empty());
}

#[test]
fn test_min_heap_empty() {
    // Edge case: popping from an empty heap
    let mut heap = MinHeap::new(|a: &i32, b: &i32| a.cmp(b));
    assert!(heap.is_empty(), "newly created heap must be empty");
    assert_eq!(heap.pop(), None, "popping from an empty heap must return None");
    assert_eq!(heap.peek(), None, "peeking at an empty heap must return None");
}

// ── 2. In-memory sort ascending ─────────────────────────────────────

#[test]
fn test_sort_in_memory() {
    let chunk = make_unsorted_chunk();
    let keys = vec![SortKey {
        column_index: 0,
        direction: SortDirection::Ascending,
        null_order: NullOrder::NullsLast,
    }];

    let sorted = ExternalSortOperator::sort_chunk(&chunk, &keys);
    assert_eq!(sorted.count(), 5, "sort must preserve all rows, not lose any");
    assert_eq!(sorted.column(0).get_value(0), ScalarValue::Int32(1), "smallest value should appear first in ascending order");
    assert_eq!(sorted.column(0).get_value(1), ScalarValue::Int32(2));
    assert_eq!(sorted.column(0).get_value(2), ScalarValue::Int32(4));
    assert_eq!(sorted.column(0).get_value(3), ScalarValue::Int32(5));
    assert_eq!(sorted.column(0).get_value(4), ScalarValue::Int32(8));
}

// ── 3. In-memory sort descending ────────────────────────────────────

#[test]
fn test_sort_descending() {
    let chunk = make_unsorted_chunk();
    let keys = vec![SortKey {
        column_index: 0,
        direction: SortDirection::Descending,
        null_order: NullOrder::NullsLast,
    }];

    let sorted = ExternalSortOperator::sort_chunk(&chunk, &keys);
    assert_eq!(sorted.column(0).get_value(0), ScalarValue::Int32(8), "descending sort should place the largest value first");
    assert_eq!(sorted.column(0).get_value(4), ScalarValue::Int32(1));
}

// ── 4. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_sort_with_nulls() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(3)]);
    chunk.append_row(&[ScalarValue::Null(LogicalType::Int32)]);
    chunk.append_row(&[ScalarValue::Int32(1)]);

    let keys = vec![SortKey {
        column_index: 0,
        direction: SortDirection::Ascending,
        null_order: NullOrder::NullsLast,
    }];

    let sorted = ExternalSortOperator::sort_chunk(&chunk, &keys);
    assert_eq!(sorted.column(0).get_value(0), ScalarValue::Int32(1));
    assert_eq!(sorted.column(0).get_value(1), ScalarValue::Int32(3));
    // NULL should be last
    assert!(!sorted.column(0).validity().is_valid(2), "NULLS LAST: null values sort after all non-null values");
}

#[test]
fn test_sort_single_element() {
    // Edge case: sorting a single-element chunk
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(42)]);

    let keys = vec![SortKey {
        column_index: 0,
        direction: SortDirection::Ascending,
        null_order: NullOrder::NullsLast,
    }];

    let sorted = ExternalSortOperator::sort_chunk(&chunk, &keys);
    assert_eq!(sorted.count(), 1, "sorting a single element must preserve it");
    assert_eq!(sorted.column(0).get_value(0), ScalarValue::Int32(42));
}

// ── 5. Multi-column sort ────────────────────────────────────────────

#[test]
fn test_sort_multi_column() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int32(30)]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int32(10)]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Int32(20)]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int32(20)]);

    let keys = vec![
        SortKey { column_index: 0, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast },
        SortKey { column_index: 1, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast },
    ];

    let sorted = ExternalSortOperator::sort_chunk(&chunk, &keys);
    // Expected order: (1,10), (1,20), (1,30), (2,20)
    assert_eq!(sorted.column(1).get_value(0), ScalarValue::Int32(10), "multi-column sort breaks ties on the first key using the second key");
    assert_eq!(sorted.column(1).get_value(1), ScalarValue::Int32(20));
    assert_eq!(sorted.column(1).get_value(2), ScalarValue::Int32(30));
}

// ── 6. String sort ──────────────────────────────────────────────────

#[test]
fn test_sort_strings() {
    let mut chunk = DataChunk::new(&[LogicalType::Varchar]);
    chunk.append_row(&[ScalarValue::Varchar("charlie".into())]);
    chunk.append_row(&[ScalarValue::Varchar("alice".into())]);
    chunk.append_row(&[ScalarValue::Varchar("bob".into())]);

    let keys = vec![SortKey {
        column_index: 0,
        direction: SortDirection::Ascending,
        null_order: NullOrder::NullsLast,
    }];

    let sorted = ExternalSortOperator::sort_chunk(&chunk, &keys);
    assert_eq!(sorted.column(0).get_value(0), ScalarValue::Varchar("alice".into()));
    assert_eq!(sorted.column(0).get_value(1), ScalarValue::Varchar("bob".into()));
    assert_eq!(sorted.column(0).get_value(2), ScalarValue::Varchar("charlie".into()));
}

// ── 7. K-way merge ──────────────────────────────────────────────────

#[test]
fn test_k_way_merge() {
    let keys = vec![SortKey {
        column_index: 0,
        direction: SortDirection::Ascending,
        null_order: NullOrder::NullsLast,
    }];

    let mut run1 = DataChunk::new(&[LogicalType::Int32]);
    run1.append_row(&[ScalarValue::Int32(1)]);
    run1.append_row(&[ScalarValue::Int32(4)]);
    run1.append_row(&[ScalarValue::Int32(7)]);

    let mut run2 = DataChunk::new(&[LogicalType::Int32]);
    run2.append_row(&[ScalarValue::Int32(2)]);
    run2.append_row(&[ScalarValue::Int32(5)]);
    run2.append_row(&[ScalarValue::Int32(8)]);

    let mut run3 = DataChunk::new(&[LogicalType::Int32]);
    run3.append_row(&[ScalarValue::Int32(3)]);
    run3.append_row(&[ScalarValue::Int32(6)]);
    run3.append_row(&[ScalarValue::Int32(9)]);

    let runs = vec![vec![run1], vec![run2], vec![run3]];
    let merged = ExternalSortOperator::k_way_merge(&runs, &keys);

    let mut all_values = Vec::new();
    for chunk in &merged {
        for i in 0..chunk.count() {
            if let ScalarValue::Int32(v) = chunk.column(0).get_value(i) {
                all_values.push(v);
            }
        }
    }
    assert_eq!(all_values, vec![1, 2, 3, 4, 5, 6, 7, 8, 9], "k-way merge combines multiple sorted runs into one globally sorted sequence");
}

// ── 8. Pipeline integration ─────────────────────────────────────────

#[test]
fn test_external_sort_pipeline() {
    let chunk = make_unsorted_chunk();
    let source = InMemorySource::new(vec![chunk], vec![LogicalType::Int32, LogicalType::Varchar]);

    let sort_op = ExternalSortOperator::new(
        vec![SortKey {
            column_index: 0,
            direction: SortDirection::Ascending,
            null_order: NullOrder::NullsLast,
        }],
        vec![LogicalType::Int32, LogicalType::Varchar],
        1024 * 1024, // 1MB budget
    );

    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(sort_op));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    assert!(!results.is_empty(), "external sort pipeline must produce at least one output chunk containing all sorted rows");
}

// ── 9. Top-N optimization ───────────────────────────────────────────

#[test]
fn test_top_n() {
    let chunk = make_unsorted_chunk();
    let source = InMemorySource::new(vec![chunk], vec![LogicalType::Int32, LogicalType::Varchar]);

    let top_n = TopNOperator::new(
        vec![SortKey {
            column_index: 0,
            direction: SortDirection::Ascending,
            null_order: NullOrder::NullsLast,
        }],
        3,
        vec![LogicalType::Int32, LogicalType::Varchar],
    );

    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(top_n));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 3, "Top-N should return exactly N rows, avoiding a full sort of all data");

    // Should be the 3 smallest: 1, 2, 4
    let chunk = &results[0];
    assert_eq!(chunk.column(0).get_value(0), ScalarValue::Int32(1));
    assert_eq!(chunk.column(0).get_value(1), ScalarValue::Int32(2));
    assert_eq!(chunk.column(0).get_value(2), ScalarValue::Int32(4));
}
