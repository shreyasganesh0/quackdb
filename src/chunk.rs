//! Lesson 04: Data Chunks
//!
//! A `DataChunk` bundles multiple `Vector`s (columns) of equal length into a
//! row group for vectorized processing. Operators in a query pipeline pass
//! `DataChunk`s between each other rather than individual rows.
//!
//! Key Rust concepts: `Vec` of heterogeneous-typed columns, `Display` trait,
//! slice operations, and iterator combinators.

use crate::types::LogicalType;
use crate::vector::Vector;
use std::fmt;

/// A chunk of columnar data -- multiple vectors of the same length.
///
/// This is the unit of data exchange between operators in the query engine.
/// Typical chunk size is 1024-2048 rows for cache-friendly vectorized execution.
pub struct DataChunk {
    columns: Vec<Vector>,
    count: usize,
}

impl DataChunk {
    /// Create a new empty data chunk with columns of the given types.
    ///
    /// Each column is initialized with zero rows.
    pub fn new(types: &[LogicalType]) -> Self {
        todo!()
    }

    /// Create a data chunk with columns pre-allocated to hold `capacity` rows.
    pub fn with_capacity(types: &[LogicalType], capacity: usize) -> Self {
        todo!()
    }

    /// Number of rows in this chunk.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Set the number of rows.
    pub fn set_count(&mut self, count: usize) {
        self.count = count;
    }

    /// Number of columns in this chunk.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get a reference to the column vector at the given index.
    pub fn column(&self, index: usize) -> &Vector {
        &self.columns[index]
    }

    /// Get a mutable reference to the column vector at the given index.
    pub fn column_mut(&mut self, index: usize) -> &mut Vector {
        &mut self.columns[index]
    }

    /// Get a slice of all column vectors.
    pub fn columns(&self) -> &[Vector] {
        &self.columns
    }

    /// Append a row of values. Each value corresponds to one column.
    ///
    /// Panics if the number of values does not match the number of columns.
    // Hint: iterate over columns and values in parallel, calling
    // `set_value` on each column at offset `self.count`, then increment count.
    pub fn append_row(&mut self, values: &[crate::types::ScalarValue]) {
        todo!()
    }

    /// Produce a new chunk containing rows `[offset .. offset + length]`.
    // Hint: use `Vector::copy_with_selection` or build a SelectionVector
    // for the requested range.
    pub fn slice(&self, offset: usize, length: usize) -> DataChunk {
        todo!()
    }

    /// Flatten all constant vectors in this chunk into flat vectors.
    pub fn flatten(&mut self) {
        todo!()
    }

    /// Reset this chunk for reuse (set count to 0, clear vectors).
    pub fn reset(&mut self) {
        todo!()
    }

    /// Get the logical types of all columns.
    pub fn types(&self) -> Vec<LogicalType> {
        self.columns.iter().map(|c| c.logical_type().clone()).collect()
    }
}

// Display a chunk as a simple tabular text representation.
impl fmt::Display for DataChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// A collection of data chunks, used for materializing intermediate results.
///
/// Acts as an append-only list of `DataChunk`s that share the same schema.
pub struct ChunkCollection {
    types: Vec<LogicalType>,
    chunks: Vec<DataChunk>,
    total_count: usize,
}

impl ChunkCollection {
    /// Create a new empty chunk collection with the given column types.
    pub fn new(types: Vec<LogicalType>) -> Self {
        todo!()
    }

    /// Append a chunk to this collection.
    ///
    /// The chunk's column types must match the collection's schema.
    pub fn append(&mut self, chunk: DataChunk) {
        todo!()
    }

    /// Total number of rows across all chunks.
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    /// Number of chunks in this collection.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Get a reference to a chunk by index.
    pub fn get_chunk(&self, index: usize) -> &DataChunk {
        &self.chunks[index]
    }

    /// Get the schema types for this collection.
    pub fn types(&self) -> &[LogicalType] {
        &self.types
    }

    /// Iterate over all chunks as a slice.
    pub fn chunks(&self) -> &[DataChunk] {
        &self.chunks
    }
}
