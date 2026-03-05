//! Lesson 15: Projection Operator
//!
//! Projects columns and evaluates expressions.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::expression::Expression;
use super::pipeline::{OperatorResult, PhysicalOperator};

/// Projection operator that selects/computes columns.
pub struct ProjectionOperator {
    expressions: Vec<Expression>,
    output_types: Vec<LogicalType>,
}

impl ProjectionOperator {
    /// Create a new projection operator.
    pub fn new(expressions: Vec<Expression>, output_types: Vec<LogicalType>) -> Self {
        Self {
            expressions,
            output_types,
        }
    }
}

impl PhysicalOperator for ProjectionOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        todo!()
    }

    fn name(&self) -> &str {
        "Projection"
    }
}
