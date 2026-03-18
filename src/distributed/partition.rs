//! Lesson 31: Data Partitioning
//!
//! Supports hash, range, and round-robin partitioning for distributing table
//! data across nodes. Also includes partition pruning to skip irrelevant
//! partitions during query execution.

use crate::chunk::DataChunk;
use crate::types::{LogicalType, ScalarValue};

/// Describes how rows are assigned to partitions.
#[derive(Debug, Clone)]
pub enum PartitionScheme {
    /// Hash the specified columns and mod by `num_partitions`.
    Hash { columns: Vec<usize>, num_partitions: usize },
    /// Assign rows to partitions based on sorted range boundaries on a single column.
    Range { column: usize, boundaries: Vec<ScalarValue> },
    /// Distribute rows in round-robin order (ignoring content).
    RoundRobin { num_partitions: usize },
}

/// Assigns rows from a `DataChunk` to partitions according to a scheme.
pub struct Partitioner {
    scheme: PartitionScheme,
}

impl Partitioner {
    /// Create a partitioner for the given scheme.
    pub fn new(scheme: PartitionScheme) -> Self {
        Self { scheme }
    }

    /// Partition a chunk, returning one `DataChunk` per partition.
    ///
    /// The returned vector has length `num_partitions()`; empty partitions
    /// get an empty chunk.
    pub fn partition(&self, chunk: &DataChunk) -> Vec<DataChunk> {
        // Hint: iterate rows, call `partition_for_row` for each, then
        // build per-partition chunks by collecting matching rows.
        todo!()
    }

    /// Determine which partition a single row belongs to.
    ///
    /// For Hash: hash the key columns and mod by num_partitions.
    /// For Range: binary-search the boundaries.
    /// For RoundRobin: use `row % num_partitions`.
    pub fn partition_for_row(&self, chunk: &DataChunk, row: usize) -> usize {
        todo!()
    }

    /// Total number of partitions produced by this scheme.
    pub fn num_partitions(&self) -> usize {
        match &self.scheme {
            PartitionScheme::Hash { num_partitions, .. } => *num_partitions,
            PartitionScheme::Range { boundaries, .. } => boundaries.len() + 1,
            PartitionScheme::RoundRobin { num_partitions } => *num_partitions,
        }
    }
}

/// A table whose rows are distributed across multiple partitions.
pub struct PartitionedTable {
    name: String,
    schema: Vec<(String, LogicalType)>,
    scheme: PartitionScheme,
    /// One Vec<DataChunk> per partition.
    partitions: Vec<Vec<DataChunk>>,
}

impl PartitionedTable {
    /// Create a new partitioned table with empty partitions.
    pub fn new(name: String, schema: Vec<(String, LogicalType)>, scheme: PartitionScheme) -> Self {
        let partitioner = Partitioner::new(scheme.clone());
        let num = partitioner.num_partitions();
        Self {
            name,
            schema,
            scheme,
            partitions: (0..num).map(|_| Vec::new()).collect(),
        }
    }

    /// Insert a chunk, routing each row to the correct partition.
    pub fn insert(&mut self, chunk: DataChunk) {
        todo!()
    }

    /// Scan all partitions, returning chunks in partition order.
    pub fn scan_all(&self) -> Vec<DataChunk> {
        self.partitions.iter().flat_map(|p| p.iter().cloned()).collect()
    }

    /// Scan a single partition by ID.
    ///
    /// # Panics
    /// Panics if `partition_id >= num_partitions()`.
    pub fn scan_partition(&self, partition_id: usize) -> &[DataChunk] {
        &self.partitions[partition_id]
    }

    /// Number of partitions in this table.
    pub fn num_partitions(&self) -> usize {
        self.partitions.len()
    }

    /// Re-distribute all data under a new partitioning scheme.
    ///
    /// Reads all existing data, re-partitions it, and replaces the partitions.
    pub fn repartition(&mut self, new_scheme: PartitionScheme) {
        // Hint: scan_all -> create new Partitioner -> re-partition each chunk.
        todo!()
    }
}

/// Determines which partitions can be skipped for a given predicate.
pub struct PartitionPruner;

impl PartitionPruner {
    /// Given an equality predicate on `column` with `value`, return the
    /// partition IDs that could contain matching rows.
    ///
    /// For Hash: compute the hash and return a single partition.
    /// For Range: binary-search boundaries and return the matching partition.
    /// For RoundRobin: return all partitions (no pruning possible).
    pub fn prune(
        scheme: &PartitionScheme,
        num_partitions: usize,
        column: usize,
        value: &ScalarValue,
    ) -> Vec<usize> {
        todo!()
    }
}
