//! # Lesson 15: Scan, Filter, Projection — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Basic filter — equality (`test_filter_equality`)
//! 2. Filter — no matches (`test_filter_no_matches`)
//! 3. Edge cases (filter with nulls)
//! 4. Compound filter — range (`test_filter_range`)
//! 5. Projection — column reorder (`test_projection_columns`)
//! 6. Projection — computed expressions (`test_projection_expression`)
//! 7. Combined filter + project pipeline (`test_filter_then_project`)

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::execution::pipeline::*;
use quackdb::execution::expression::*;
use quackdb::execution::filter::FilterOperator;
use quackdb::execution::projection::ProjectionOperator;

fn make_test_data() -> Vec<DataChunk> {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64, LogicalType::Float64]);
    chunk.append_row(&[ScalarValue::Int32(1), ScalarValue::Int64(100), ScalarValue::Float64(1.5)]);
    chunk.append_row(&[ScalarValue::Int32(2), ScalarValue::Int64(200), ScalarValue::Float64(2.5)]);
    chunk.append_row(&[ScalarValue::Int32(3), ScalarValue::Int64(300), ScalarValue::Float64(3.5)]);
    chunk.append_row(&[ScalarValue::Int32(4), ScalarValue::Int64(400), ScalarValue::Float64(4.5)]);
    chunk.append_row(&[ScalarValue::Int32(5), ScalarValue::Int64(500), ScalarValue::Float64(5.5)]);
    vec![chunk]
}

// ── 1. Basic filter — equality ──────────────────────────────────────

#[test]
fn test_filter_equality() {
    let data = make_test_data();
    let source = InMemorySource::new(data, vec![LogicalType::Int32, LogicalType::Int64, LogicalType::Float64]);

    let predicate = Expression::BinaryOp {
        op: BinaryOp::Equal,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::Constant(ScalarValue::Int32(3))),
    };

    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(FilterOperator::new(predicate)));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 1, "equality filter should pass through only the single matching row");
}

// ── 2. Filter — no matches ─────────────────────────────────────────

#[test]
fn test_filter_no_matches() {
    let data = make_test_data();
    let source = InMemorySource::new(data, vec![LogicalType::Int32, LogicalType::Int64, LogicalType::Float64]);

    let predicate = Expression::BinaryOp {
        op: BinaryOp::GreaterThan,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::Constant(ScalarValue::Int32(100))),
    };

    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(FilterOperator::new(predicate)));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 0, "filter should produce zero rows when no data satisfies the predicate");
}

// ── 3. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_filter_with_nulls() {
    // NULL values in filter predicates should be treated as false (SQL three-valued logic)
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(10)]);
    chunk.append_row(&[ScalarValue::Null(LogicalType::Int32)]);
    chunk.append_row(&[ScalarValue::Int32(30)]);

    let source = InMemorySource::new(vec![chunk], vec![LogicalType::Int32]);

    let predicate = Expression::BinaryOp {
        op: BinaryOp::GreaterThan,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::Constant(ScalarValue::Int32(5))),
    };

    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(FilterOperator::new(predicate)));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    // NULL > 5 is NULL (falsy), so only rows with 10 and 30 match
    assert_eq!(total, 2, "NULL comparisons are falsy in SQL: NULL > 5 is not true, so NULL rows are filtered out");
}

#[test]
fn test_filter_all_match() {
    // Edge case: filter where all rows match
    let data = make_test_data();
    let source = InMemorySource::new(data, vec![LogicalType::Int32, LogicalType::Int64, LogicalType::Float64]);

    let predicate = Expression::BinaryOp {
        op: BinaryOp::GreaterThan,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::Constant(ScalarValue::Int32(0))),
    };

    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(FilterOperator::new(predicate)));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 5, "when all rows match the predicate, all rows should pass through");
}

// ── 4. Compound filter — range ──────────────────────────────────────

#[test]
fn test_filter_range() {
    let data = make_test_data();
    let source = InMemorySource::new(data, vec![LogicalType::Int32, LogicalType::Int64, LogicalType::Float64]);

    // id > 2 AND id < 5
    let predicate = Expression::BinaryOp {
        op: BinaryOp::And,
        left: Box::new(Expression::BinaryOp {
            op: BinaryOp::GreaterThan,
            left: Box::new(Expression::ColumnRef(0)),
            right: Box::new(Expression::Constant(ScalarValue::Int32(2))),
        }),
        right: Box::new(Expression::BinaryOp {
            op: BinaryOp::LessThan,
            left: Box::new(Expression::ColumnRef(0)),
            right: Box::new(Expression::Constant(ScalarValue::Int32(5))),
        }),
    };

    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(FilterOperator::new(predicate)));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 2, "compound AND predicate should narrow results to rows satisfying both conditions");
}

// ── 5. Projection — column reorder ──────────────────────────────────

#[test]
fn test_projection_columns() {
    let data = make_test_data();
    let source = InMemorySource::new(data, vec![LogicalType::Int32, LogicalType::Int64, LogicalType::Float64]);

    let proj = ProjectionOperator::new(
        vec![Expression::ColumnRef(2), Expression::ColumnRef(0)],
        vec![LogicalType::Float64, LogicalType::Int32],
    );

    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(proj));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let chunk = &results[0];
    assert_eq!(chunk.column_count(), 2, "projection should reduce output to only the requested columns");
    assert_eq!(chunk.column(0).get_value(0), ScalarValue::Float64(1.5), "projection should reorder columns: col2 is now first in output");
    assert_eq!(chunk.column(1).get_value(0), ScalarValue::Int32(1));
}

// ── 6. Projection — computed expressions ────────────────────────────

#[test]
fn test_projection_expression() {
    let data = make_test_data();
    let source = InMemorySource::new(data, vec![LogicalType::Int32, LogicalType::Int64, LogicalType::Float64]);

    // Project: col0 * 10
    let expr = Expression::BinaryOp {
        op: BinaryOp::Multiply,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::Constant(ScalarValue::Int32(10))),
    };
    let proj = ProjectionOperator::new(vec![expr], vec![LogicalType::Int32]);

    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(proj));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let chunk = &results[0];
    assert_eq!(chunk.column(0).get_value(0), ScalarValue::Int32(10), "projection can compute expressions, not just pass through columns");
    assert_eq!(chunk.column(0).get_value(4), ScalarValue::Int32(50));
}

// ── 7. Combined filter + project pipeline ───────────────────────────

#[test]
fn test_filter_then_project() {
    let data = make_test_data();
    let source = InMemorySource::new(data, vec![LogicalType::Int32, LogicalType::Int64, LogicalType::Float64]);

    let predicate = Expression::BinaryOp {
        op: BinaryOp::GreaterThanOrEqual,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::Constant(ScalarValue::Int32(3))),
    };

    let proj = ProjectionOperator::new(
        vec![Expression::ColumnRef(1)],
        vec![LogicalType::Int64],
    );

    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(FilterOperator::new(predicate)));
    pipeline.add_operator(Box::new(proj));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 3, "filter-then-project: filter reduces rows first, projection selects columns from survivors");

    // Check values are from column 1 (Int64)
    let chunk = &results[0];
    assert_eq!(*chunk.column(0).logical_type(), LogicalType::Int64, "projection output type should match the projected column's type");
}
