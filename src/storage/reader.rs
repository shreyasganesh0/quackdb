//! Lesson 12: Columnar File Reader
//!
//! Read columnar files produced by `ColumnarFileWriter`, supporting column
//! projection (reading only selected columns) and predicate pushdown (skipping
//! row groups whose statistics prove no rows can match).
//!
//! Key Rust concepts: `Read + Seek` trait bounds for random-access I/O,
//! enum-based predicate operators, and combining statistics-based pruning
//! with projection pushdown for efficient scans.

use super::columnar_file::{FileFooter, RowGroupMeta, ColumnStats, MAGIC};
use crate::types::{LogicalType, ScalarValue};
use crate::chunk::DataChunk;
use std::io::{Read, Seek};

/// A predicate for row group pruning (pushed down from the query engine).
///
/// Compares a column's statistics against a constant value to decide whether
/// an entire row group can be skipped.
#[derive(Debug, Clone)]
pub struct ScanPredicate {
    /// Which column this predicate applies to.
    pub column_index: usize,
    /// Comparison operator.
    pub op: PredicateOp,
    /// The constant value to compare against.
    pub value: ScalarValue,
}

/// Supported comparison operators for scan predicates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PredicateOp {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

impl ScanPredicate {
    /// Check if a row group can be pruned (skipped) based on column stats.
    ///
    /// Returns `true` if the stats *guarantee* no rows in the group can
    /// satisfy this predicate (i.e., the group can safely be skipped).
    // Hint: compare `self.value` against `stats.min_value` and `stats.max_value`.
    // For example, `col > 100` can prune a group whose max_value <= 100.
    pub fn can_prune(&self, stats: &ColumnStats, logical_type: &LogicalType) -> bool {
        todo!()
    }
}

/// Reader for columnar files.
///
/// Opens the file, reads the footer, and provides methods to read individual
/// column chunks or perform full scans with projection and predicate pushdown.
// The generic `R: Read + Seek` allows reading from files, in-memory cursors, etc.
pub struct ColumnarFileReader<R: Read + Seek> {
    reader: R,
    footer: FileFooter,
}

impl<R: Read + Seek> ColumnarFileReader<R> {
    /// Open a columnar file and read its footer.
    ///
    // Hint: seek to the end to read the footer size, then seek back to
    // read the footer bytes. Verify the MAGIC bytes at start and end.
    pub fn open(reader: R) -> Result<Self, String> {
        todo!()
    }

    /// Get the file footer (schema + row group metadata).
    pub fn footer(&self) -> &FileFooter {
        &self.footer
    }

    /// Get the schema (column names and types).
    pub fn schema(&self) -> &[(String, LogicalType)] {
        &self.footer.schema
    }

    /// Total number of rows in the file.
    pub fn total_rows(&self) -> u64 {
        self.footer.total_rows
    }

    /// Number of row groups in the file.
    pub fn row_group_count(&self) -> usize {
        self.footer.row_groups.len()
    }

    /// Read a specific column chunk's raw bytes from a row group.
    ///
    // Hint: look up the column's offset and size from `RowGroupMeta`,
    // seek to that offset, and read `size` bytes.
    pub fn read_column(&mut self, row_group: usize, column: usize) -> Result<Vec<u8>, String> {
        todo!()
    }

    /// Scan the file with optional column projection and predicates.
    ///
    /// `projection`: if `Some`, read only the listed column indices.
    /// `predicates`: row groups that fail all predicates are skipped entirely.
    ///
    /// Returns a `Vec<DataChunk>`, one per non-pruned row group.
    // Hint: for each row group, check predicates against column stats to
    // decide whether to skip. For non-skipped groups, read only the
    // projected columns and assemble a DataChunk.
    pub fn scan(
        &mut self,
        projection: Option<&[usize]>,
        predicates: &[ScanPredicate],
    ) -> Result<Vec<DataChunk>, String> {
        todo!()
    }
}
