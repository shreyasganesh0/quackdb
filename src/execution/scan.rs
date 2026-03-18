//! Lesson 15: Table Scan Operator
//!
//! Scans data from a [`DataSource`] with optional column pruning (projection
//! pushdown). This is typically the leaf operator in an execution pipeline,
//! responsible for reading base table data.
//!
//! **Key idea:** If a `projection` is specified, only the listed column
//! indices are materialized in the output chunk, reducing memory and CPU cost.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::pipeline::{DataSource, OperatorResult, PhysicalOperator};

/// Table scan operator that reads chunks from a data source and optionally
/// projects (selects) a subset of columns.
pub struct TableScanOperator {
    /// The underlying data source to read from.
    source: Box<dyn DataSource>,
    /// If `Some`, only these column indices are included in output chunks.
    projection: Option<Vec<usize>>,
    /// The output column types (after projection, if any).
    output_types: Vec<LogicalType>,
}

impl TableScanOperator {
    /// Create a new table scan operator.
    ///
    /// If `projection` is `Some`, the output will contain only the specified
    /// columns (in order). If `None`, all columns are returned.
    pub fn new(source: Box<dyn DataSource>, projection: Option<Vec<usize>>) -> Self {
        // Hint: derive output_types from source.schema(). If projection is
        // Some, select only those indices; otherwise use the full schema.
        todo!()
    }

    /// Pull the next chunk from the source, applying projection if configured.
    ///
    /// Returns `None` when the source is exhausted.
    pub fn next_chunk(&mut self) -> Result<Option<DataChunk>, String> {
        // Hint: call self.source.next_chunk(). If a chunk is returned and
        // projection is set, build a new DataChunk containing only the
        // projected columns.
        todo!()
    }
}
