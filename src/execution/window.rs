//! Lesson 30: Window Functions
//!
//! Evaluates SQL window functions (`ROW_NUMBER`, `RANK`, `SUM OVER`, etc.)
//! with support for `PARTITION BY`, `ORDER BY`, and custom window frames.
//! The operator buffers all input (window functions need full partitions),
//! then evaluates each window definition and appends result columns.

use crate::chunk::DataChunk;
use crate::types::{LogicalType, ScalarValue};
use super::pipeline::{OperatorResult, PhysicalOperator};
use super::sort_merge_join::SortKey;

/// Supported window function types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowFunctionType {
    /// Assigns a sequential integer to each row within its partition.
    RowNumber,
    /// Rank with gaps (ties get the same rank, next rank skips).
    Rank,
    /// Rank without gaps (ties get the same rank, next rank is +1).
    DenseRank,
    /// Value from a preceding row (offset rows before current).
    Lag,
    /// Value from a following row (offset rows after current).
    Lead,
    /// Running or windowed sum.
    Sum,
    /// Running or windowed average.
    Avg,
    /// Running or windowed count.
    Count,
    /// Running or windowed minimum.
    Min,
    /// Running or windowed maximum.
    Max,
}

/// Specifies one end of a window frame boundary.
#[derive(Debug, Clone)]
pub enum FrameBound {
    /// All rows from the start of the partition.
    UnboundedPreceding,
    /// A fixed number of rows before the current row.
    Preceding(usize),
    /// The current row itself.
    CurrentRow,
    /// A fixed number of rows after the current row.
    Following(usize),
    /// All rows through the end of the partition.
    UnboundedFollowing,
}

/// A window frame defined by a start and end bound.
///
/// The default frame is `ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW`.
#[derive(Debug, Clone)]
pub struct WindowFrame {
    pub start: FrameBound,
    pub end: FrameBound,
}

impl WindowFrame {
    /// Returns the SQL-standard default frame.
    pub fn default_frame() -> Self {
        Self {
            start: FrameBound::UnboundedPreceding,
            end: FrameBound::CurrentRow,
        }
    }
}

/// Complete definition of a single window function invocation.
#[derive(Debug, Clone)]
pub struct WindowDef {
    /// Which window function to compute.
    pub function_type: WindowFunctionType,
    /// Column indices for PARTITION BY (rows with equal values form a group).
    pub partition_by: Vec<usize>,
    /// Column indices and directions for ORDER BY within each partition.
    pub order_by: Vec<SortKey>,
    /// The window frame (ROWS BETWEEN ... AND ...).
    pub frame: WindowFrame,
    /// Input column for aggregate window functions (Sum, Avg, etc.).
    pub arg_column: Option<usize>,
    /// Offset for Lag/Lead functions.
    pub offset: Option<usize>,
    /// Default value for Lag/Lead when the offset goes out of bounds.
    pub default_value: Option<ScalarValue>,
}

/// Trait for window function evaluation strategies.
///
/// Each function type gets its own implementation so the evaluation loop
/// can be specialised (e.g., ranking functions don't need frame bounds).
pub trait WindowFunction {
    /// Evaluate the window function over a single partition.
    ///
    /// `partition` contains the sorted rows; `order` maps original row indices
    /// to sorted positions; `frame` defines the window bounds.
    fn evaluate(&self, partition: &DataChunk, order: &[usize], frame: &WindowFrame) -> Vec<ScalarValue>;

    /// The output type of this window function.
    fn result_type(&self) -> LogicalType;
}

/// Create the appropriate `WindowFunction` implementation for the given type.
///
/// `input_type` is needed for aggregate functions (Sum, Avg) to determine
/// the output type.
pub fn create_window_function(func_type: WindowFunctionType, input_type: Option<&LogicalType>) -> Box<dyn WindowFunction> {
    // Hint: match on func_type and return the corresponding struct.
    todo!()
}

/// Physical operator that evaluates window functions in the pipeline.
///
/// Because window functions require the complete partition to be available,
/// this operator buffers all input chunks during `execute` and produces
/// output only in `finalize`.
pub struct WindowOperator {
    window_defs: Vec<WindowDef>,
    output_types: Vec<LogicalType>,
    /// Buffered input chunks; accumulated until `finalize` is called.
    input_buffer: Vec<DataChunk>,
    /// Whether `finalize` has already been called.
    finalized: bool,
}

impl WindowOperator {
    /// Create a new window operator.
    ///
    /// `input_types` describes the schema of incoming chunks; the output
    /// schema appends one column per window definition.
    pub fn new(window_defs: Vec<WindowDef>, input_types: Vec<LogicalType>) -> Self {
        // Hint: output_types = input_types ++ one type per window_def
        // (use create_window_function(...).result_type() for each).
        todo!()
    }
}

// PhysicalOperator trait impl -- buffer input, produce output on finalize.
impl PhysicalOperator for WindowOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        self.input_buffer.push(input.clone());
        Ok(OperatorResult::NeedMoreInput)
    }

    fn finalize(&mut self) -> Result<Option<DataChunk>, String> {
        // Hint: concatenate buffered chunks, sort each partition, evaluate
        // each WindowDef, append result columns, return the final chunk.
        todo!()
    }

    fn name(&self) -> &str {
        "Window"
    }
}
