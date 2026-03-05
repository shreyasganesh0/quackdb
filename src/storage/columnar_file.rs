//! Lessons 11-12: Columnar File Format (Writer)
//!
//! A Parquet-like columnar file format with row groups, column chunks, and statistics.

use crate::types::LogicalType;
use crate::chunk::DataChunk;
use std::io::Write;

/// Magic bytes identifying a QuackDB columnar file.
pub const MAGIC: &[u8; 4] = b"QUAK";

/// Column statistics for pruning.
#[derive(Debug, Clone)]
pub struct ColumnStats {
    pub null_count: u64,
    pub min_value: Option<Vec<u8>>,
    pub max_value: Option<Vec<u8>>,
    pub distinct_count: Option<u64>,
}

impl ColumnStats {
    pub fn new() -> Self {
        todo!()
    }

    /// Update stats with a new value.
    pub fn update(&mut self, value: &[u8], is_null: bool) {
        todo!()
    }

    /// Merge another stats into this one.
    pub fn merge(&mut self, other: &ColumnStats) {
        todo!()
    }
}

/// Metadata for a column chunk within a row group.
#[derive(Debug, Clone)]
pub struct ColumnChunkMeta {
    pub column_index: usize,
    pub logical_type: LogicalType,
    pub offset: u64,
    pub size: u64,
    pub num_values: u64,
    pub stats: ColumnStats,
    pub compression: u8,
}

/// Metadata for a row group.
#[derive(Debug, Clone)]
pub struct RowGroupMeta {
    pub num_rows: u64,
    pub columns: Vec<ColumnChunkMeta>,
}

/// File footer containing schema and row group metadata.
#[derive(Debug, Clone)]
pub struct FileFooter {
    pub schema: Vec<(String, LogicalType)>,
    pub row_groups: Vec<RowGroupMeta>,
    pub total_rows: u64,
}

impl FileFooter {
    /// Serialize the footer to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    /// Deserialize the footer from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        todo!()
    }
}

/// Writer for columnar files.
pub struct ColumnarFileWriter<W: Write> {
    writer: W,
    schema: Vec<(String, LogicalType)>,
    row_groups: Vec<RowGroupMeta>,
    current_row_group: Option<RowGroupInProgress>,
    bytes_written: u64,
}

struct RowGroupInProgress {
    num_rows: u64,
    columns: Vec<ColumnChunkMeta>,
}

impl<W: Write> ColumnarFileWriter<W> {
    /// Create a new columnar file writer with the given schema.
    pub fn new(writer: W, schema: Vec<(String, LogicalType)>) -> Result<Self, String> {
        todo!()
    }

    /// Begin a new row group.
    pub fn begin_row_group(&mut self) -> Result<(), String> {
        todo!()
    }

    /// Write a column chunk for the current row group.
    pub fn write_column(&mut self, column_index: usize, data: &[u8], num_values: u64, stats: ColumnStats) -> Result<(), String> {
        todo!()
    }

    /// End the current row group.
    pub fn end_row_group(&mut self, num_rows: u64) -> Result<(), String> {
        todo!()
    }

    /// Write a DataChunk as a complete row group.
    pub fn write_chunk(&mut self, chunk: &DataChunk) -> Result<(), String> {
        todo!()
    }

    /// Finish writing the file (writes footer).
    pub fn finish(self) -> Result<W, String> {
        todo!()
    }
}
