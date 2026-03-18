//! # Lesson 15: Execution Operators — Filter (File 2 of 3)
//!
//! This file implements the filter operator, which evaluates a boolean predicate
//! against each input chunk and forwards only matching rows downstream.
//!
//! It works together with:
//! - `scan.rs` — the table scan operator that produces the chunks this filter
//!   consumes.
//! - `projection.rs` — the projection operator that reshapes columns after
//!   filtering.
//!
//! **Implementation order**: Implement `scan.rs` first, then this file, then
//! `projection.rs`. The filter sits between scan and projection in the pipeline.
//!
//! **Key idea:** Evaluate the predicate expression to get a boolean vector,
//! then build a selection vector of row indices where the result is `true`.
//! Use that selection vector to compact the chunk before emitting it.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use crate::vector::SelectionVector;
use super::expression::Expression;
use super::pipeline::{OperatorResult, PhysicalOperator};

/// Filter operator that evaluates a boolean predicate and only passes
/// through rows that satisfy it.
pub struct FilterOperator {
    /// The boolean predicate expression (must evaluate to a boolean vector).
    predicate: Expression,
}

impl FilterOperator {
    /// Create a new filter operator with the given predicate expression.
    pub fn new(predicate: Expression) -> Self {
        Self { predicate }
    }

    /// Evaluate the predicate on `chunk` and return a selection vector
    /// containing the indices of rows that satisfy the predicate.
    ///
    /// Returns an empty selection vector if no rows match.
    pub fn evaluate(&self, chunk: &DataChunk) -> Result<SelectionVector, String> {
        // Hint: use ExpressionExecutor::execute(&self.predicate, chunk) to
        // get a boolean result vector, then iterate over it collecting
        // indices where the value is true (and not NULL).
        todo!()
    }
}

// Trait impl: makes FilterOperator usable inside a Pipeline.
impl PhysicalOperator for FilterOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        // Hint: a filter does not change the schema — return the same
        // column types as the input. You may need to store input types
        // or accept them at construction time.
        todo!()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        // Hint: call self.evaluate(input) to get matching row indices,
        // then build a new DataChunk containing only those rows.
        // If no rows match, return NeedMoreInput instead of an empty chunk.
        todo!()
    }

    fn name(&self) -> &str {
        "Filter"
    }
}
