//! # Lesson 13: Expression Evaluation — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Constant expressions (`test_constant_expression`)
//! 2. Column references (`test_column_ref_expression`)
//! 3. Arithmetic operations (`test_binary_add`, `test_binary_subtract`, `test_binary_multiply`)
//! 4. Comparison operations (`test_comparison_equal`, `test_comparison_less_than`)
//! 5. Boolean logic (`test_boolean_and`)
//! 6. Unary operations (`test_unary_negate`, `test_unary_not`, `test_unary_is_null`)
//! 7. Type casting (`test_cast_expression`)
//! 8. Edge cases — null propagation, empty chunk
//! 9. Result type inference (`test_expression_result_type`)
//! 10. Nested/complex expressions (`test_nested_expression`)

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::vector::Vector;
use quackdb::chunk::DataChunk;
use quackdb::execution::expression::*;

// ── 1. Constant expressions ────────────────────────────────────────

#[test]
fn test_constant_expression() {
    let chunk = DataChunk::new(&[LogicalType::Int32]);
    let expr = Expression::Constant(ScalarValue::Int32(42));
    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Int32(42), "constant expressions should evaluate to their literal value regardless of input data");
}

// ── 2. Column references ───────────────────────────────────────────

#[test]
fn test_column_ref_expression() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int64]);
    chunk.append_row(&[ScalarValue::Int32(10), ScalarValue::Int64(100)]);
    chunk.append_row(&[ScalarValue::Int32(20), ScalarValue::Int64(200)]);

    let expr = Expression::ColumnRef(0);
    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Int32(10));
    assert_eq!(result.get_value(1), ScalarValue::Int32(20));
}

// ── 3. Arithmetic operations ────────────────────────────────────────

#[test]
fn test_binary_add() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(10), ScalarValue::Int32(5)]);
    chunk.append_row(&[ScalarValue::Int32(20), ScalarValue::Int32(3)]);

    let expr = Expression::BinaryOp {
        op: BinaryOp::Add,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::ColumnRef(1)),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Int32(15));
    assert_eq!(result.get_value(1), ScalarValue::Int32(23));
}

#[test]
fn test_binary_subtract() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(100)]);

    let expr = Expression::BinaryOp {
        op: BinaryOp::Subtract,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::Constant(ScalarValue::Int32(30))),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Int32(70));
}

#[test]
fn test_binary_multiply() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(7)]);

    let expr = Expression::BinaryOp {
        op: BinaryOp::Multiply,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::Constant(ScalarValue::Int32(6))),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Int32(42));
}

// ── 4. Comparison operations ────────────────────────────────────────

#[test]
fn test_comparison_equal() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(42)]);
    chunk.append_row(&[ScalarValue::Int32(10)]);

    let expr = Expression::BinaryOp {
        op: BinaryOp::Equal,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::Constant(ScalarValue::Int32(42))),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Boolean(true), "equality comparison should return true when values match");
    assert_eq!(result.get_value(1), ScalarValue::Boolean(false), "equality comparison should return false when values differ");
}

#[test]
fn test_comparison_less_than() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(5)]);
    chunk.append_row(&[ScalarValue::Int32(15)]);

    let expr = Expression::BinaryOp {
        op: BinaryOp::LessThan,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::Constant(ScalarValue::Int32(10))),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Boolean(true));
    assert_eq!(result.get_value(1), ScalarValue::Boolean(false));
}

// ── 5. Boolean logic ────────────────────────────────────────────────

#[test]
fn test_boolean_and() {
    let mut chunk = DataChunk::new(&[LogicalType::Boolean, LogicalType::Boolean]);
    chunk.append_row(&[ScalarValue::Boolean(true), ScalarValue::Boolean(true)]);
    chunk.append_row(&[ScalarValue::Boolean(true), ScalarValue::Boolean(false)]);
    chunk.append_row(&[ScalarValue::Boolean(false), ScalarValue::Boolean(true)]);

    let expr = Expression::BinaryOp {
        op: BinaryOp::And,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::ColumnRef(1)),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Boolean(true), "AND requires both operands to be true");
    assert_eq!(result.get_value(1), ScalarValue::Boolean(false), "AND returns false if either operand is false");
    assert_eq!(result.get_value(2), ScalarValue::Boolean(false));
}

// ── 6. Unary operations ────────────────────────────────────────────

#[test]
fn test_unary_negate() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(42)]);
    chunk.append_row(&[ScalarValue::Int32(-10)]);

    let expr = Expression::UnaryOp {
        op: UnaryOp::Negate,
        expr: Box::new(Expression::ColumnRef(0)),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Int32(-42), "negating a positive value should produce a negative");
    assert_eq!(result.get_value(1), ScalarValue::Int32(10), "negating a negative value should produce a positive (double negation)");
}

#[test]
fn test_unary_not() {
    let mut chunk = DataChunk::new(&[LogicalType::Boolean]);
    chunk.append_row(&[ScalarValue::Boolean(true)]);
    chunk.append_row(&[ScalarValue::Boolean(false)]);

    let expr = Expression::UnaryOp {
        op: UnaryOp::Not,
        expr: Box::new(Expression::ColumnRef(0)),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Boolean(false));
    assert_eq!(result.get_value(1), ScalarValue::Boolean(true));
}

#[test]
fn test_unary_is_null() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(42)]);
    chunk.append_row(&[ScalarValue::Null(LogicalType::Int32)]);

    let expr = Expression::UnaryOp {
        op: UnaryOp::IsNull,
        expr: Box::new(Expression::ColumnRef(0)),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Boolean(false), "IS NULL should return false for non-null values");
    assert_eq!(result.get_value(1), ScalarValue::Boolean(true), "IS NULL should return true for null values");
}

// ── 7. Type casting ────────────────────────────────────────────────

#[test]
fn test_cast_expression() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(42)]);

    let expr = Expression::Cast {
        expr: Box::new(Expression::ColumnRef(0)),
        target_type: LogicalType::Float64,
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Float64(42.0), "casting Int32 to Float64 should preserve the numeric value");
}

// ── 8. Edge cases — null propagation ────────────────────────────────

#[test]
fn test_null_propagation() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(10), ScalarValue::Null(LogicalType::Int32)]);

    let expr = Expression::BinaryOp {
        op: BinaryOp::Add,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::ColumnRef(1)),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    // NULL + anything = NULL
    assert!(!result.validity().is_valid(0), "NULL propagation: any arithmetic with NULL must produce NULL");
}

#[test]
fn test_negate_zero() {
    // Edge case: negating zero should produce zero
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(0)]);

    let expr = Expression::UnaryOp {
        op: UnaryOp::Negate,
        expr: Box::new(Expression::ColumnRef(0)),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Int32(0), "negating zero must produce zero");
}

// ── 9. Result type inference ────────────────────────────────────────

#[test]
fn test_expression_result_type() {
    let types = vec![LogicalType::Int32, LogicalType::Int64];

    let expr = Expression::BinaryOp {
        op: BinaryOp::Add,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::ColumnRef(1)),
    };
    let result_type = expr.result_type(&types).unwrap();
    // Int32 + Int64 should produce Int64
    assert_eq!(result_type, LogicalType::Int64, "type promotion: Int32 + Int64 should widen to Int64 to avoid precision loss");

    let cmp = Expression::BinaryOp {
        op: BinaryOp::Equal,
        left: Box::new(Expression::ColumnRef(0)),
        right: Box::new(Expression::Constant(ScalarValue::Int32(5))),
    };
    assert_eq!(cmp.result_type(&types).unwrap(), LogicalType::Boolean, "comparison operators always produce Boolean regardless of input types");
}

// ── 10. Nested/complex expressions ──────────────────────────────────

#[test]
fn test_nested_expression() {
    // (a + b) * 2
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(3), ScalarValue::Int32(4)]);

    let expr = Expression::BinaryOp {
        op: BinaryOp::Multiply,
        left: Box::new(Expression::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(Expression::ColumnRef(0)),
            right: Box::new(Expression::ColumnRef(1)),
        }),
        right: Box::new(Expression::Constant(ScalarValue::Int32(2))),
    };

    let result = ExpressionExecutor::execute(&expr, &chunk).unwrap();
    assert_eq!(result.get_value(0), ScalarValue::Int32(14), "nested expressions should evaluate inner operations first: (3+4)*2=14");
}
