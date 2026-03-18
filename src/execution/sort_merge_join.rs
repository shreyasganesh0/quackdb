//! Lesson 18: Sort-Merge Join
//!
//! Merge join on pre-sorted inputs with multi-column row comparators. Both
//! inputs must be sorted on their join keys. The algorithm advances two
//! cursors in lockstep, emitting matches when keys are equal.
//!
//! **Key idea:** Because both sides are sorted, a single linear pass
//! suffices to find all matches — no hash table needed. This is efficient
//! when inputs are already sorted (e.g., from an index or a preceding sort).

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::pipeline::{OperatorResult, PhysicalOperator};
use super::hash_join::JoinType;
use std::cmp::Ordering;

/// Sort direction for a single key column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Where NULLs should appear relative to non-NULL values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NullOrder {
    NullsFirst,
    NullsLast,
}

/// Specification for a single sort key column.
#[derive(Debug, Clone)]
pub struct SortKey {
    /// Index of the column to sort on.
    pub column_index: usize,
    /// Whether to sort ascending or descending.
    pub direction: SortDirection,
    /// Where NULLs are ordered relative to non-NULL values.
    pub null_order: NullOrder,
}

/// Multi-column row comparator.
///
/// Compares two rows by evaluating sort keys in priority order. The first
/// key that differs determines the ordering; if all keys are equal, the
/// rows are considered equal.
pub struct RowComparator {
    sort_keys: Vec<SortKey>,
}

impl RowComparator {
    /// Create a new comparator with the given sort key specifications.
    pub fn new(sort_keys: Vec<SortKey>) -> Self {
        Self { sort_keys }
    }

    /// Compare row `left_row` from `left_chunk` with row `right_row`
    /// from `right_chunk`, returning an [`Ordering`].
    ///
    /// Iterates through sort keys in order; the first key that produces
    /// a non-equal comparison determines the result.
    pub fn compare(
        &self,
        left_chunk: &DataChunk,
        left_row: usize,
        right_chunk: &DataChunk,
        right_row: usize,
    ) -> Ordering {
        // Hint: for each SortKey, extract the scalar value from both chunks
        // at the given row, compare them (respecting direction and null_order),
        // and return immediately if not Equal.
        todo!()
    }

    /// Compare two rows within the same chunk.
    ///
    /// Convenience wrapper that avoids requiring two separate chunk references.
    pub fn compare_within(
        &self,
        chunk: &DataChunk,
        row_a: usize,
        row_b: usize,
    ) -> Ordering {
        // Hint: delegate to self.compare(chunk, row_a, chunk, row_b).
        todo!()
    }
}

/// Normalizes composite keys into a single byte-comparable format.
///
/// This enables fast memcmp-style comparisons instead of per-column
/// scalar comparisons. Each column value is encoded so that byte ordering
/// matches the logical sort order.
pub struct KeyNormalizer;

impl KeyNormalizer {
    /// Serialize a row's sort key columns into a single byte vector
    /// whose lexicographic order matches the desired sort order.
    ///
    /// For descending keys, flip all bits so that byte ordering is inverted.
    /// For NULLs, use a sentinel byte (0x00 or 0xFF) depending on null_order.
    pub fn normalize(chunk: &DataChunk, row: usize, key_columns: &[SortKey]) -> Vec<u8> {
        // Hint: for each SortKey, extract the scalar, encode it as
        // big-endian bytes (so byte order matches numeric order),
        // and flip bits for Descending. Concatenate all keys.
        todo!()
    }
}

/// Sort-merge join operator on pre-sorted inputs.
///
/// Both the left and right inputs must be added (already sorted on their
/// respective keys) before calling `merge`.
pub struct MergeJoinOperator {
    /// The type of join to perform.
    join_type: JoinType,
    /// Sort keys for the left input.
    left_keys: Vec<SortKey>,
    /// Sort keys for the right input.
    right_keys: Vec<SortKey>,
    /// Output column types (left types ++ right types).
    output_types: Vec<LogicalType>,
    /// Buffered sorted chunks from the left input.
    left_buffer: Vec<DataChunk>,
    /// Buffered sorted chunks from the right input.
    right_buffer: Vec<DataChunk>,
    /// Current cursor position in the left buffer.
    left_pos: usize,
    /// Current cursor position in the right buffer.
    right_pos: usize,
}

impl MergeJoinOperator {
    /// Create a new merge join operator.
    ///
    /// `left_keys` and `right_keys` describe how each side is sorted.
    /// Both inputs must already be sorted according to these keys.
    pub fn new(
        join_type: JoinType,
        left_keys: Vec<SortKey>,
        right_keys: Vec<SortKey>,
        left_types: Vec<LogicalType>,
        right_types: Vec<LogicalType>,
    ) -> Self {
        // Hint: compute output_types as left_types ++ right_types.
        // Initialize buffers and positions.
        todo!()
    }

    /// Add a sorted chunk from the left input.
    pub fn add_left(&mut self, chunk: DataChunk) {
        self.left_buffer.push(chunk);
    }

    /// Add a sorted chunk from the right input.
    pub fn add_right(&mut self, chunk: DataChunk) {
        self.right_buffer.push(chunk);
    }

    /// Execute the merge join over the buffered inputs and produce results.
    ///
    /// Advances left and right cursors in lockstep:
    /// - If keys are equal, emit the joined row(s) and handle duplicates.
    /// - If left < right, advance the left cursor.
    /// - If left > right, advance the right cursor.
    pub fn merge(&mut self) -> Result<Vec<DataChunk>, String> {
        // Hint: use a RowComparator (or KeyNormalizer) to compare keys.
        // Be careful with duplicate keys on either side — you may need
        // to "rewind" the right cursor for groups of equal left keys.
        todo!()
    }
}
