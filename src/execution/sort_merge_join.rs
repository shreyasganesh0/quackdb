//! Lesson 18: Sort-Merge Join
//!
//! Merge join on pre-sorted inputs with row comparators.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::pipeline::{OperatorResult, PhysicalOperator};
use super::hash_join::JoinType;
use std::cmp::Ordering;

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Null ordering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NullOrder {
    NullsFirst,
    NullsLast,
}

/// A sort key specification.
#[derive(Debug, Clone)]
pub struct SortKey {
    pub column_index: usize,
    pub direction: SortDirection,
    pub null_order: NullOrder,
}

/// Multi-column row comparator.
pub struct RowComparator {
    sort_keys: Vec<SortKey>,
}

impl RowComparator {
    pub fn new(sort_keys: Vec<SortKey>) -> Self {
        Self { sort_keys }
    }

    /// Compare two rows from two chunks.
    pub fn compare(
        &self,
        left_chunk: &DataChunk,
        left_row: usize,
        right_chunk: &DataChunk,
        right_row: usize,
    ) -> Ordering {
        todo!()
    }

    /// Compare two rows within the same chunk.
    pub fn compare_within(
        &self,
        chunk: &DataChunk,
        row_a: usize,
        row_b: usize,
    ) -> Ordering {
        todo!()
    }
}

/// Normalizes keys into byte-comparable format for efficient comparison.
pub struct KeyNormalizer;

impl KeyNormalizer {
    /// Normalize a row's key columns into a single byte-comparable key.
    pub fn normalize(chunk: &DataChunk, row: usize, key_columns: &[SortKey]) -> Vec<u8> {
        todo!()
    }
}

/// Sort-merge join operator on pre-sorted inputs.
pub struct MergeJoinOperator {
    join_type: JoinType,
    left_keys: Vec<SortKey>,
    right_keys: Vec<SortKey>,
    output_types: Vec<LogicalType>,
    left_buffer: Vec<DataChunk>,
    right_buffer: Vec<DataChunk>,
    left_pos: usize,
    right_pos: usize,
}

impl MergeJoinOperator {
    pub fn new(
        join_type: JoinType,
        left_keys: Vec<SortKey>,
        right_keys: Vec<SortKey>,
        left_types: Vec<LogicalType>,
        right_types: Vec<LogicalType>,
    ) -> Self {
        todo!()
    }

    /// Add a chunk from the left side (must be sorted).
    pub fn add_left(&mut self, chunk: DataChunk) {
        self.left_buffer.push(chunk);
    }

    /// Add a chunk from the right side (must be sorted).
    pub fn add_right(&mut self, chunk: DataChunk) {
        self.right_buffer.push(chunk);
    }

    /// Execute the merge join and produce results.
    pub fn merge(&mut self) -> Result<Vec<DataChunk>, String> {
        todo!()
    }
}
