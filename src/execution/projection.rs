//! # Lesson 15: Execution Operators — Projection (File 3 of 3)
//!
//! This file implements the projection operator, which selects and/or computes
//! output columns by evaluating a list of expressions against each input chunk.
//!
//! It works together with:
//! - `scan.rs` — the table scan operator that produces base table data.
//! - `filter.rs` — the filter operator that removes rows before projection.
//!
//! **Implementation order**: Implement `scan.rs` first, then `filter.rs`, then
//! this file. Projection is the final reshaping step in a scan-filter-project
//! pipeline.
//!
//! **Key idea:** For each input chunk, evaluate every expression in the
//! projection list to produce one output vector per expression, then
//! assemble those vectors into a new chunk.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::expression::Expression;
use super::pipeline::{OperatorResult, PhysicalOperator};

/// Projection operator that selects and/or computes output columns.
pub struct ProjectionOperator {
    /// The list of expressions to evaluate (one per output column).
    expressions: Vec<Expression>,
    /// The output column types (one per expression).
    output_types: Vec<LogicalType>,
}

impl ProjectionOperator {
    /// Create a new projection operator.
    ///
    /// `expressions` defines the output columns. `output_types` must have
    /// the same length and contain each expression's result type.
    pub fn new(expressions: Vec<Expression>, output_types: Vec<LogicalType>) -> Self {
        Self {
            expressions,
            output_types,
        }
    }
}

// Trait impl: makes ProjectionOperator usable inside a Pipeline.
impl PhysicalOperator for ProjectionOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        // Hint: for each expression in self.expressions, call
        // ExpressionExecutor::execute(expr, input) to get a Vector.
        // Collect all vectors into a new DataChunk and return Output(chunk).
        todo!()
    }

    fn name(&self) -> &str {
        "Projection"
    }
}
