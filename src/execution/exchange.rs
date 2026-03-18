//! Lesson 32: Exchange Operators
//!
//! Provides a physical operator that represents a data exchange boundary in
//! a distributed or parallel query plan. Exchange operators decouple pipeline
//! segments, enabling data redistribution between execution threads or nodes.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::pipeline::{OperatorResult, PhysicalOperator};

/// Physical exchange operator for pipeline integration.
///
/// In a full implementation this would read from an `ExchangeReceiver` channel.
/// Here it acts as a placeholder that threads into the `PhysicalOperator` pipeline.
pub struct ExchangeOperator {
    /// Output schema (must match the schema of chunks flowing through the exchange).
    output_types: Vec<LogicalType>,
}

impl ExchangeOperator {
    /// Create an exchange operator with the given output schema.
    pub fn new(output_types: Vec<LogicalType>) -> Self {
        Self { output_types }
    }
}

// PhysicalOperator trait impl -- in a real system, `execute` would pull
// from an exchange channel rather than receiving local input.
impl PhysicalOperator for ExchangeOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        // Hint: for a local pass-through, wrap the input chunk in
        // OperatorResult::Output. For distributed, pull from a channel.
        todo!()
    }

    fn name(&self) -> &str {
        "Exchange"
    }
}
