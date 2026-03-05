//! Lesson 15: Filter Operator
//!
//! Filters rows based on a predicate expression.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use crate::vector::SelectionVector;
use super::expression::Expression;
use super::pipeline::{OperatorResult, PhysicalOperator};

/// Filter operator that evaluates a predicate and produces a selection vector.
pub struct FilterOperator {
    predicate: Expression,
}

impl FilterOperator {
    /// Create a new filter operator with the given predicate.
    pub fn new(predicate: Expression) -> Self {
        Self { predicate }
    }

    /// Evaluate the predicate on a chunk and return matching row indices.
    pub fn evaluate(&self, chunk: &DataChunk) -> Result<SelectionVector, String> {
        todo!()
    }
}

impl PhysicalOperator for FilterOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        todo!()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        todo!()
    }

    fn name(&self) -> &str {
        "Filter"
    }
}
