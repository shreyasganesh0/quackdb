//! Lesson 15: Table Scan Operator
//!
//! Scans data from a source with optional column pruning.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::pipeline::{DataSource, OperatorResult, PhysicalOperator};

/// Table scan operator that reads from a data source.
pub struct TableScanOperator {
    source: Box<dyn DataSource>,
    projection: Option<Vec<usize>>,
    output_types: Vec<LogicalType>,
}

impl TableScanOperator {
    /// Create a new table scan operator.
    pub fn new(source: Box<dyn DataSource>, projection: Option<Vec<usize>>) -> Self {
        todo!()
    }

    /// Get the next chunk from the source with projection applied.
    pub fn next_chunk(&mut self) -> Result<Option<DataChunk>, String> {
        todo!()
    }
}
