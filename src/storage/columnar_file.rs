//! Lessons 11-12: Columnar File Format (Writer)
//!
//! A Parquet-like columnar file format with row groups, column chunks, and
//! per-column statistics. The file layout is:
//!
//!   [MAGIC] [row group 1 columns...] [row group 2 columns...] ... [footer] [footer_size] [MAGIC]
//!
//! The footer contains schema info and metadata for every row group/column,
//! enabling the reader to seek directly to the data it needs.
//!
//! Key Rust concepts: `Write` trait for streaming output, builder pattern
//! for `ColumnarFileWriter`, and binary serialization of structured metadata.

use crate::types::LogicalType;
use crate::chunk::DataChunk;
use std::io::Write;

/// Magic bytes identifying a QuackDB columnar file.
pub const MAGIC: &[u8; 4] = b"QUAK";

/// Column-level statistics for predicate pushdown and row group pruning.
///
/// Min/max values are stored as raw bytes so they are type-agnostic.
#[derive(Debug, Clone)]
pub struct ColumnStats {
    /// Number of null values in the column chunk.
    pub null_count: u64,
    /// Minimum value (serialized bytes), or `None` if all null.
    pub min_value: Option<Vec<u8>>,
    /// Maximum value (serialized bytes), or `None` if all null.
    pub max_value: Option<Vec<u8>>,
    /// Approximate count of distinct values, if computed.
    pub distinct_count: Option<u64>,
}

impl ColumnStats {
    /// Create empty stats with no values observed yet.
    pub fn new() -> Self {
        todo!()
    }

    /// Update stats with a new observed value.
    ///
    /// Tracks min/max by byte-wise comparison and increments the null count
    /// when `is_null` is true.
    pub fn update(&mut self, value: &[u8], is_null: bool) {
        todo!()
    }

    /// Merge another `ColumnStats` into this one (for combining row groups).
    pub fn merge(&mut self, other: &ColumnStats) {
        todo!()
    }
}

/// Metadata for a single column chunk within a row group.
#[derive(Debug, Clone)]
pub struct ColumnChunkMeta {
    /// Index of this column in the schema.
    pub column_index: usize,
    /// The logical type of values in this chunk.
    pub logical_type: LogicalType,
    /// Byte offset of the column chunk data within the file.
    pub offset: u64,
    /// Byte size of the compressed column chunk data.
    pub size: u64,
    /// Number of values (rows) in this column chunk.
    pub num_values: u64,
    /// Per-column statistics for pruning.
    pub stats: ColumnStats,
    /// Compression algorithm ID (0 = none, 1 = RLE, etc.).
    pub compression: u8,
}

/// Metadata for a row group (a horizontal partition of the table).
#[derive(Debug, Clone)]
pub struct RowGroupMeta {
    /// Number of rows in this row group.
    pub num_rows: u64,
    /// Metadata for each column chunk in this row group.
    pub columns: Vec<ColumnChunkMeta>,
}

/// File footer containing schema and all row group metadata.
///
/// Written at the end of the file so the entire file can be written in a
/// single sequential pass.
#[derive(Debug, Clone)]
pub struct FileFooter {
    /// Column names and types.
    pub schema: Vec<(String, LogicalType)>,
    /// Metadata for every row group in the file.
    pub row_groups: Vec<RowGroupMeta>,
    /// Total number of rows across all row groups.
    pub total_rows: u64,
}

impl FileFooter {
    /// Serialize the footer to bytes.
    // Hint: serialize schema, row groups, and stats into a binary format.
    // Consider using a simple length-prefixed encoding for variable-length fields.
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    /// Deserialize the footer from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        todo!()
    }
}

/// Writer for columnar files.
///
/// Usage: create -> write MAGIC -> begin_row_group -> write_column (per col)
/// -> end_row_group -> ... -> finish (writes footer + trailing MAGIC).
// The generic `W: Write` allows writing to files, buffers, or network streams.
pub struct ColumnarFileWriter<W: Write> {
    writer: W,
    schema: Vec<(String, LogicalType)>,
    row_groups: Vec<RowGroupMeta>,
    current_row_group: Option<RowGroupInProgress>,
    bytes_written: u64,
}

/// Internal state for a row group that is being written.
struct RowGroupInProgress {
    num_rows: u64,
    columns: Vec<ColumnChunkMeta>,
}

impl<W: Write> ColumnarFileWriter<W> {
    /// Create a new columnar file writer and write the file header (MAGIC bytes).
    pub fn new(writer: W, schema: Vec<(String, LogicalType)>) -> Result<Self, String> {
        todo!()
    }

    /// Begin a new row group. Must be called before `write_column`.
    pub fn begin_row_group(&mut self) -> Result<(), String> {
        todo!()
    }

    /// Write a column chunk for the current row group.
    ///
    /// `data` is the (possibly compressed) column payload.
    pub fn write_column(&mut self, column_index: usize, data: &[u8], num_values: u64, stats: ColumnStats) -> Result<(), String> {
        todo!()
    }

    /// End the current row group, recording its metadata.
    pub fn end_row_group(&mut self, num_rows: u64) -> Result<(), String> {
        todo!()
    }

    /// Convenience method: write an entire `DataChunk` as one row group.
    pub fn write_chunk(&mut self, chunk: &DataChunk) -> Result<(), String> {
        todo!()
    }

    /// Finish writing the file: serialize and write the footer, then the
    /// footer size and trailing MAGIC bytes.
    pub fn finish(self) -> Result<W, String> {
        todo!()
    }
}
