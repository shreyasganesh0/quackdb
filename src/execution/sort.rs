//! Lesson 19: External Sort
//!
//! In-memory and external sorting with k-way merge and TopN optimization.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::sort_merge_join::{SortKey, RowComparator};
use super::pipeline::{OperatorResult, PhysicalOperator};

/// A min-heap implementation for k-way merging.
pub struct MinHeap<T> {
    data: Vec<T>,
    compare: Box<dyn Fn(&T, &T) -> std::cmp::Ordering>,
}

impl<T> MinHeap<T> {
    /// Create a new min-heap with the given comparison function.
    pub fn new(compare: impl Fn(&T, &T) -> std::cmp::Ordering + 'static) -> Self {
        todo!()
    }

    /// Push a value onto the heap.
    pub fn push(&mut self, value: T) {
        todo!()
    }

    /// Pop the minimum value from the heap.
    pub fn pop(&mut self) -> Option<T> {
        todo!()
    }

    /// Peek at the minimum value.
    pub fn peek(&self) -> Option<&T> {
        self.data.first()
    }

    /// Number of elements in the heap.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// External sort operator with memory budget.
pub struct ExternalSortOperator {
    sort_keys: Vec<SortKey>,
    output_types: Vec<LogicalType>,
    memory_budget: usize,
    /// Sorted runs (each run is a sorted list of chunks).
    runs: Vec<Vec<DataChunk>>,
    /// Current run being built.
    current_run: Vec<DataChunk>,
    current_run_size: usize,
    finalized: bool,
}

impl ExternalSortOperator {
    /// Create a new external sort operator.
    pub fn new(sort_keys: Vec<SortKey>, output_types: Vec<LogicalType>, memory_budget: usize) -> Self {
        todo!()
    }

    /// Sort a single chunk in memory.
    pub fn sort_chunk(chunk: &DataChunk, sort_keys: &[SortKey]) -> DataChunk {
        todo!()
    }

    /// Perform k-way merge of sorted runs.
    pub fn k_way_merge(runs: &[Vec<DataChunk>], sort_keys: &[SortKey]) -> Vec<DataChunk> {
        todo!()
    }
}

impl PhysicalOperator for ExternalSortOperator {
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
        "ExternalSort"
    }
}

/// TopN operator for ORDER BY ... LIMIT N.
pub struct TopNOperator {
    sort_keys: Vec<SortKey>,
    limit: usize,
    output_types: Vec<LogicalType>,
    buffer: Vec<DataChunk>,
    finalized: bool,
}

impl TopNOperator {
    pub fn new(sort_keys: Vec<SortKey>, limit: usize, output_types: Vec<LogicalType>) -> Self {
        todo!()
    }
}

impl PhysicalOperator for TopNOperator {
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
        "TopN"
    }
}
