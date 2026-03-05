//! Lesson 30: Window Functions
//!
//! Window function evaluation with partitioning, ordering, and framing.

use crate::chunk::DataChunk;
use crate::types::{LogicalType, ScalarValue};
use super::pipeline::{OperatorResult, PhysicalOperator};
use super::sort_merge_join::SortKey;

/// Window function types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowFunctionType {
    RowNumber,
    Rank,
    DenseRank,
    Lag,
    Lead,
    Sum,
    Avg,
    Count,
    Min,
    Max,
}

/// Window frame bounds.
#[derive(Debug, Clone)]
pub enum FrameBound {
    UnboundedPreceding,
    Preceding(usize),
    CurrentRow,
    Following(usize),
    UnboundedFollowing,
}

/// Window frame.
#[derive(Debug, Clone)]
pub struct WindowFrame {
    pub start: FrameBound,
    pub end: FrameBound,
}

impl WindowFrame {
    pub fn default_frame() -> Self {
        Self {
            start: FrameBound::UnboundedPreceding,
            end: FrameBound::CurrentRow,
        }
    }
}

/// A window function definition.
#[derive(Debug, Clone)]
pub struct WindowDef {
    pub function_type: WindowFunctionType,
    pub partition_by: Vec<usize>,
    pub order_by: Vec<SortKey>,
    pub frame: WindowFrame,
    pub arg_column: Option<usize>,
    pub offset: Option<usize>,
    pub default_value: Option<ScalarValue>,
}

/// Trait for window function implementations.
pub trait WindowFunction {
    fn evaluate(&self, partition: &DataChunk, order: &[usize], frame: &WindowFrame) -> Vec<ScalarValue>;
    fn result_type(&self) -> LogicalType;
}

/// Create a window function implementation.
pub fn create_window_function(func_type: WindowFunctionType, input_type: Option<&LogicalType>) -> Box<dyn WindowFunction> {
    todo!()
}

/// Window operator for pipeline execution.
pub struct WindowOperator {
    window_defs: Vec<WindowDef>,
    output_types: Vec<LogicalType>,
    input_buffer: Vec<DataChunk>,
    finalized: bool,
}

impl WindowOperator {
    pub fn new(window_defs: Vec<WindowDef>, input_types: Vec<LogicalType>) -> Self {
        todo!()
    }
}

impl PhysicalOperator for WindowOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        todo!()
    }

    fn finalize(&mut self) -> Result<Option<DataChunk>, String> {
        todo!()
    }

    fn name(&self) -> &str {
        "Window"
    }
}
