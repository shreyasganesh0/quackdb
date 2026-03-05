//! Exchange operators for the execution engine.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::pipeline::{OperatorResult, PhysicalOperator};

/// Exchange operator placeholder for pipeline integration.
pub struct ExchangeOperator {
    output_types: Vec<LogicalType>,
}

impl ExchangeOperator {
    pub fn new(output_types: Vec<LogicalType>) -> Self {
        Self { output_types }
    }
}

impl PhysicalOperator for ExchangeOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        todo!()
    }

    fn name(&self) -> &str {
        "Exchange"
    }
}
