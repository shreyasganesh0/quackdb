//! Lesson 13: Expression Evaluation
//!
//! Vectorized expression evaluation over data chunks.

use crate::types::{LogicalType, ScalarValue};
use crate::vector::Vector;
use crate::chunk::DataChunk;

/// Binary operators.
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

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
    IsNull,
    IsNotNull,
}

/// An expression tree for vectorized evaluation.
#[derive(Debug, Clone)]
pub enum Expression {
    /// A constant value.
    Constant(ScalarValue),
    /// A reference to a column in the input chunk.
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
    /// A type cast.
    Cast {
        expr: Box<Expression>,
        target_type: LogicalType,
    },
}

impl Expression {
    /// Determine the result type of this expression given input types.
    pub fn result_type(&self, input_types: &[LogicalType]) -> Result<LogicalType, String> {
        todo!()
    }
}

/// Executes expressions against data chunks.
pub struct ExpressionExecutor;

impl ExpressionExecutor {
    /// Evaluate an expression against a data chunk, producing a result vector.
    pub fn execute(expr: &Expression, chunk: &DataChunk) -> Result<Vector, String> {
        todo!()
    }

    /// Execute a binary operation on two vectors.
    pub fn execute_binary(op: BinaryOp, left: &Vector, right: &Vector) -> Result<Vector, String> {
        todo!()
    }

    /// Execute a unary operation on a vector.
    pub fn execute_unary(op: UnaryOp, input: &Vector) -> Result<Vector, String> {
        todo!()
    }
}
