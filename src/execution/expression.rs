//! Lesson 13: Expression Evaluation
//!
//! Vectorized expression evaluation over data chunks. This module defines
//! an expression tree (constants, column references, binary/unary ops, casts)
//! and an executor that evaluates expressions against a [`DataChunk`],
//! producing a result [`Vector`].
//!
//! **Key idea:** Walk the expression tree recursively. Leaf nodes (constants,
//! column refs) produce vectors directly; internal nodes evaluate children
//! first and then apply the operator element-wise.

use crate::types::{LogicalType, ScalarValue};
use crate::vector::Vector;
use crate::chunk::DataChunk;

/// Binary operators supported in expressions.
///
/// Arithmetic operators (`Add` through `Modulo`) operate on numeric types.
/// Comparison operators (`Equal` through `GreaterThanOrEqual`) return booleans.
/// Logical operators (`And`, `Or`) combine boolean predicates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

/// Unary operators supported in expressions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    /// Arithmetic negation (e.g., `-x`).
    Negate,
    /// Logical negation (e.g., `NOT x`).
    Not,
    /// NULL check — returns true when the value is NULL.
    IsNull,
    /// Inverse NULL check — returns true when the value is non-NULL.
    IsNotNull,
}

/// An expression tree for vectorized evaluation.
///
/// Each variant represents a node in the tree. Evaluation proceeds
/// bottom-up: leaf nodes produce vectors, and parent nodes combine
/// their children's results.
#[derive(Debug, Clone)]
pub enum Expression {
    /// A constant value broadcast to every row.
    Constant(ScalarValue),
    /// A reference to a column in the input chunk (by index).
    ColumnRef(usize),
    /// A binary operation on two sub-expressions.
    BinaryOp {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    /// A unary operation on a sub-expression.
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    /// A type cast converting a sub-expression's result to `target_type`.
    Cast {
        expr: Box<Expression>,
        target_type: LogicalType,
    },
}

impl Expression {
    /// Determine the result type of this expression given the input column types.
    ///
    /// Recurse through the tree: constants return their own type,
    /// column refs look up the type by index, binary ops apply type
    /// promotion rules, and casts return their target type.
    pub fn result_type(&self, input_types: &[LogicalType]) -> Result<LogicalType, String> {
        // Hint: match on self — for ColumnRef, index into input_types;
        // for BinaryOp, compute child types then determine the result
        // (e.g., comparison ops always return Boolean).
        todo!()
    }
}

/// Stateless executor that evaluates expression trees against data chunks.
pub struct ExpressionExecutor;

impl ExpressionExecutor {
    /// Evaluate an expression against a data chunk, producing a result vector.
    ///
    /// Recursively walks the expression tree. For `Constant`, create a
    /// vector filled with the value. For `ColumnRef`, clone the column
    /// from the chunk. For operators, evaluate children first then apply
    /// the operation element-wise.
    pub fn execute(expr: &Expression, chunk: &DataChunk) -> Result<Vector, String> {
        // Hint: match on expr and recurse; call execute_binary / execute_unary
        // for operator nodes.
        todo!()
    }

    /// Execute a binary operation element-wise on two vectors.
    ///
    /// Both vectors must have the same length. Iterate over paired elements,
    /// applying `op` to each pair and collecting results into a new vector.
    pub fn execute_binary(op: BinaryOp, left: &Vector, right: &Vector) -> Result<Vector, String> {
        // Hint: match on (op, left_value, right_value) for each row index.
        // Handle NULL propagation: if either operand is NULL, the result is NULL.
        todo!()
    }

    /// Execute a unary operation element-wise on a vector.
    ///
    /// Applies `op` to each element and returns a new vector with the results.
    pub fn execute_unary(op: UnaryOp, input: &Vector) -> Result<Vector, String> {
        // Hint: for Negate, flip the sign; for Not, invert the boolean;
        // for IsNull/IsNotNull, check the NULL bitmap.
        todo!()
    }
}
