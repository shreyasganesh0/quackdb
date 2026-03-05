//! Lesson 04: Data Chunks
//!
//! DataChunk bundles multiple Vectors into a row group for vectorized processing.

use crate::types::LogicalType;
use crate::vector::Vector;
use std::fmt;

/// A chunk of columnar data — multiple vectors of the same length.
pub struct DataChunk {
    columns: Vec<Vector>,
    count: usize,
}

impl DataChunk {
    /// Create a new data chunk with columns of the given types.
    pub fn new(types: &[LogicalType]) -> Self {
        todo!()
    }

    /// Create a data chunk with the given capacity.
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

    /// Number of columns.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get a reference to a column vector.
    pub fn column(&self, index: usize) -> &Vector {
        &self.columns[index]
    }

    /// Get a mutable reference to a column vector.
    pub fn column_mut(&mut self, index: usize) -> &mut Vector {
        &mut self.columns[index]
    }

    /// Get a slice of all columns.
    pub fn columns(&self) -> &[Vector] {
        &self.columns
    }

    /// Append a row of values. Each value corresponds to one column.
    pub fn append_row(&mut self, values: &[crate::types::ScalarValue]) {
        todo!()
    }

    /// Slice this chunk to produce a new chunk with rows [offset..offset+length].
    pub fn slice(&self, offset: usize, length: usize) -> DataChunk {
        todo!()
    }

    /// Flatten all constant vectors in this chunk.
    pub fn flatten(&mut self) {
        todo!()
    }

    /// Reset this chunk for reuse (set count to 0).
    pub fn reset(&mut self) {
        todo!()
    }

    /// Get the logical types of all columns.
    pub fn types(&self) -> Vec<LogicalType> {
        self.columns.iter().map(|c| c.logical_type().clone()).collect()
    }
}

impl fmt::Display for DataChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// A collection of data chunks, used for materializing intermediate results.
pub struct ChunkCollection {
    types: Vec<LogicalType>,
    chunks: Vec<DataChunk>,
    total_count: usize,
}

impl ChunkCollection {
    /// Create a new empty chunk collection with the given schema.
    pub fn new(types: Vec<LogicalType>) -> Self {
        todo!()
    }

    /// Append a chunk to this collection.
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

    /// Get the schema types.
    pub fn types(&self) -> &[LogicalType] {
        &self.types
    }

    /// Iterate over all chunks.
    pub fn chunks(&self) -> &[DataChunk] {
        &self.chunks
    }
}
