//! Lesson 31: Data Partitioning
//!
//! Hash, range, and round-robin partitioning for distributed tables.

use crate::chunk::DataChunk;
use crate::types::{LogicalType, ScalarValue};

/// Partitioning schemes.
#[derive(Debug, Clone)]
pub enum PartitionScheme {
    Hash { columns: Vec<usize>, num_partitions: usize },
    Range { column: usize, boundaries: Vec<ScalarValue> },
    RoundRobin { num_partitions: usize },
}

/// Assigns rows to partitions.
pub struct Partitioner {
    scheme: PartitionScheme,
}

impl Partitioner {
    pub fn new(scheme: PartitionScheme) -> Self {
        Self { scheme }
    }

    /// Partition a chunk, returning one chunk per partition.
    pub fn partition(&self, chunk: &DataChunk) -> Vec<DataChunk> {
        todo!()
    }

    /// Get the partition ID for a single row.
    pub fn partition_for_row(&self, chunk: &DataChunk, row: usize) -> usize {
        todo!()
    }

    /// Number of partitions.
    pub fn num_partitions(&self) -> usize {
        todo!()
    }
}

/// A partitioned table stored across multiple partitions.
pub struct PartitionedTable {
    name: String,
    schema: Vec<(String, LogicalType)>,
    scheme: PartitionScheme,
    partitions: Vec<Vec<DataChunk>>,
}

impl PartitionedTable {
    pub fn new(name: String, schema: Vec<(String, LogicalType)>, scheme: PartitionScheme) -> Self {
        todo!()
    }

    /// Insert a chunk, routing rows to correct partitions.
    pub fn insert(&mut self, chunk: DataChunk) {
        todo!()
    }

    /// Scan all partitions.
    pub fn scan_all(&self) -> Vec<DataChunk> {
        todo!()
    }

    /// Scan a specific partition.
    pub fn scan_partition(&self, partition_id: usize) -> &[DataChunk] {
        todo!()
    }

    /// Number of partitions.
    pub fn num_partitions(&self) -> usize {
        self.partitions.len()
    }

    /// Repartition the table with a new scheme.
    pub fn repartition(&mut self, new_scheme: PartitionScheme) {
        todo!()
    }
}

/// Partition pruner: determines which partitions to scan.
pub struct PartitionPruner;

impl PartitionPruner {
    /// Given a predicate, determine which partitions need to be scanned.
    pub fn prune(
        scheme: &PartitionScheme,
        num_partitions: usize,
        column: usize,
        value: &ScalarValue,
    ) -> Vec<usize> {
        todo!()
    }
}
