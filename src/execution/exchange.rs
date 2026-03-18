//! # Lesson 32: Distributed Execution — Exchange Operator (File 2 of 2)
//!
//! This file provides the physical `ExchangeOperator`, which represents a data
//! exchange boundary in a distributed or parallel query plan. Exchange operators
//! decouple pipeline segments, enabling data redistribution between execution
//! threads or nodes.
//!
//! It works together with:
//! - `distributed/planner.rs` — the distributed query planner that decides
//!   where to insert exchange boundaries and what type of exchange to use.
//!
//! **Implementation order**: Implement `distributed/planner.rs` first, then
//! this file. The planner produces `PlanFragment`s with `ExchangeType`
//! annotations; this operator is the runtime component that executes data
//! transfer at those boundaries.

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
        Ok(OperatorResult::Output(input.clone()))
    }

    fn name(&self) -> &str {
        "Exchange"
    }
}
