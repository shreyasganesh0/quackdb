//! Lesson 19: External Sort
//!
//! In-memory and external sorting with k-way merge and TopN optimization.
//! The external sort operator handles data larger than memory by partitioning
//! input into sorted *runs* that fit in a memory budget, then merging runs
//! with a k-way merge using a min-heap.
//!
//! **Key idea:** When data fits in memory, sort it directly. When it exceeds
//! the budget, flush sorted runs to the run buffer, then merge all runs using
//! a heap that always picks the smallest next element across all runs.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::sort_merge_join::{SortKey, RowComparator};
use super::pipeline::{OperatorResult, PhysicalOperator};

/// A generic min-heap for k-way merging.
///
/// The comparison function is stored as a boxed closure, allowing the heap
/// to be parameterized over any ordering.
// Generic type parameter T: the heap stores elements of type T.
// The `compare` closure defines the ordering.
pub struct MinHeap<T> {
    data: Vec<T>,
    // Boxed closure: trait object that holds the comparison function.
    // The 'static bound means the closure cannot borrow local variables.
    compare: Box<dyn Fn(&T, &T) -> std::cmp::Ordering>,
}

impl<T> MinHeap<T> {
    /// Create a new empty min-heap with the given comparison function.
    ///
    /// The closure should return `Ordering::Less` when the first argument
    /// should be popped before the second (i.e., has higher priority).
    // `impl Fn` sugar: accepts any closure matching the signature.
    // `+ 'static` required because we store it in a Box<dyn Fn>.
    pub fn new(compare: impl Fn(&T, &T) -> std::cmp::Ordering + 'static) -> Self {
        // Hint: store an empty Vec and box the comparison closure.
        todo!()
    }

    /// Push a value onto the heap, maintaining the heap invariant.
    pub fn push(&mut self, value: T) {
        // Hint: append to data, then sift-up from the last position.
        // Sift-up: while the new element is less than its parent, swap them.
        todo!()
    }

    /// Pop and return the minimum value from the heap.
    pub fn pop(&mut self) -> Option<T> {
        // Hint: swap data[0] with data[last], pop the last element,
        // then sift-down from position 0 to restore the heap invariant.
        todo!()
    }

    /// Peek at the minimum value without removing it.
    pub fn peek(&self) -> Option<&T> {
        self.data.first()
    }

    /// Return the number of elements in the heap.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Return `true` if the heap contains no elements.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// External sort operator with a configurable memory budget.
///
/// This is a pipeline breaker: `execute` accumulates input into sorted
/// runs, and `finalize` merges the runs to produce sorted output.
pub struct ExternalSortOperator {
    /// Sort key specifications (column index, direction, null ordering).
    sort_keys: Vec<SortKey>,
    /// Output column types (same as input — sort does not change schema).
    output_types: Vec<LogicalType>,
    /// Maximum bytes of data to hold in memory before flushing a run.
    memory_budget: usize,
    /// Completed sorted runs (each run is a sorted sequence of chunks).
    runs: Vec<Vec<DataChunk>>,
    /// The run currently being built from incoming chunks.
    current_run: Vec<DataChunk>,
    /// Approximate byte size of the current run.
    current_run_size: usize,
    /// Whether finalize has been called.
    finalized: bool,
}

impl ExternalSortOperator {
    /// Create a new external sort operator.
    ///
    /// `memory_budget` limits how much data is held in a single run before
    /// flushing. Set to `usize::MAX` for a pure in-memory sort.
    pub fn new(sort_keys: Vec<SortKey>, output_types: Vec<LogicalType>, memory_budget: usize) -> Self {
        // Hint: initialize runs and current_run as empty, current_run_size = 0,
        // finalized = false.
        todo!()
    }

    /// Sort a single chunk in memory according to the sort keys.
    ///
    /// Returns a new chunk with rows reordered.
    pub fn sort_chunk(chunk: &DataChunk, sort_keys: &[SortKey]) -> DataChunk {
        // Hint: build a Vec of row indices [0..chunk.len()), sort them
        // using a RowComparator, then reorder the chunk's columns
        // according to the sorted indices.
        todo!()
    }

    /// Perform k-way merge of multiple sorted runs into a single sorted
    /// sequence of chunks.
    ///
    /// Uses a [`MinHeap`] to always pick the next smallest row across all runs.
    pub fn k_way_merge(runs: &[Vec<DataChunk>], sort_keys: &[SortKey]) -> Vec<DataChunk> {
        // Hint: initialize the heap with one entry per run (the first row
        // of each run). Pop the min, emit it, and push the next row from
        // that same run. Continue until the heap is empty.
        todo!()
    }
}

// Trait impl: pipeline breaker — accumulates runs during execute, merges during finalize.
impl PhysicalOperator for ExternalSortOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        // Hint: add the input chunk to current_run and update current_run_size.
        // If current_run_size exceeds memory_budget, sort current_run,
        // move it into self.runs, and start a new run.
        // Always return NeedMoreInput.
        todo!()
    }

    fn finalize(&mut self) -> Result<Option<DataChunk>, String> {
        // Hint: flush current_run as a final sorted run if non-empty.
        // Then call k_way_merge on all runs. Return the first chunk
        // of the merged result (or None if empty).
        todo!()
    }

    fn name(&self) -> &str {
        "ExternalSort"
    }
}

/// TopN operator for `ORDER BY ... LIMIT N` queries.
///
/// Optimization: instead of sorting all data and then taking the first N
/// rows, maintain a bounded buffer of the top N rows seen so far.
pub struct TopNOperator {
    /// Sort key specifications.
    sort_keys: Vec<SortKey>,
    /// Maximum number of rows to keep.
    limit: usize,
    /// Output column types.
    output_types: Vec<LogicalType>,
    /// Buffer of chunks holding the current top-N candidates.
    buffer: Vec<DataChunk>,
    /// Whether finalize has been called.
    finalized: bool,
}

impl TopNOperator {
    /// Create a new TopN operator.
    ///
    /// `limit` is the maximum number of output rows.
    pub fn new(sort_keys: Vec<SortKey>, limit: usize, output_types: Vec<LogicalType>) -> Self {
        // Hint: initialize buffer as empty, finalized = false.
        todo!()
    }
}

// Trait impl: pipeline breaker — accumulates candidates during execute,
// sorts and truncates during finalize.
impl PhysicalOperator for TopNOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        // Hint: add input to buffer. Optionally, periodically sort and
        // truncate the buffer to keep at most `limit` rows, preventing
        // unbounded memory growth.
        todo!()
    }

    fn finalize(&mut self) -> Result<Option<DataChunk>, String> {
        // Hint: sort all buffered data by sort_keys, then take the first
        // `limit` rows and return them as a chunk.
        todo!()
    }

    fn name(&self) -> &str {
        "TopN"
    }
}
