//! # Lesson 29: Parallel Execution — Scheduler (File 2 of 2)
//!
//! This file implements the parallel pipeline scheduler, which spawns N worker
//! threads that each pull morsels from a shared `MorselQueue`, apply a pipeline
//! of physical operators, and push results to a shared `ParallelCollector`.
//!
//! It works together with:
//! - `morsel.rs` — provides the `MorselQueue` and `ParallelCollector` data
//!   structures that the scheduler's worker threads interact with.
//!
//! **Implementation order**: Implement `morsel.rs` first, then this file.
//! The scheduler's `execute` method calls `MorselQueue::take()` in a loop and
//! `ParallelCollector::push()` for each result, so those must be working first.

use super::morsel::{MorselQueue, ParallelCollector};
use crate::chunk::DataChunk;
use crate::execution::pipeline::PhysicalOperator;
use std::sync::Arc;

/// Parallel pipeline executor that spawns `num_workers` threads.
///
/// Each worker gets its own operator instance (via the factory closure) to
/// avoid shared mutable state inside operators.
pub struct ParallelPipelineExecutor {
    num_workers: usize,
}

impl ParallelPipelineExecutor {
    /// Create an executor with the given worker count.
    pub fn new(num_workers: usize) -> Self {
        Self { num_workers }
    }

    /// Execute a pipeline in parallel using morsel-driven parallelism.
    ///
    /// `operator_factory` creates a fresh operator instance for each worker,
    /// ensuring operators don't need interior mutability or synchronization.
    ///
    /// Workers loop: take morsel -> execute operator -> push result to collector.
    pub fn execute(
        &self,
        morsel_queue: Arc<MorselQueue>,
        // The Fn() -> ... + Send + Sync bounds allow the factory closure to
        // be shared safely across threads (Send) and called concurrently (Sync).
        operator_factory: impl Fn() -> Box<dyn PhysicalOperator + Send> + Send + Sync,
        collector: Arc<ParallelCollector>,
    ) -> Result<(), String> {
        // Hint: use `std::thread::scope` (or `crossbeam::scope`) to spawn
        // workers. Each worker: loop { take morsel, execute, push result }.
        // After all workers join, call `finalize` on one operator instance
        // if needed.
        todo!()
    }
}

/// A hash table partitioned across N segments for parallel aggregation.
///
/// Each worker thread writes to its own partition (determined by hash),
/// avoiding contention. After all workers finish, partitions are merged.
pub struct PartitionedHashTable {
    num_partitions: usize,
    partitions: Vec<crate::execution::hash_aggregate::AggregateHashTable>,
}

impl PartitionedHashTable {
    /// Create a partitioned hash table with one `AggregateHashTable` per partition.
    pub fn new(
        num_partitions: usize,
        group_types: Vec<crate::types::LogicalType>,
        agg_types: Vec<crate::execution::hash_aggregate::AggregateType>,
        agg_input_types: Vec<crate::types::LogicalType>,
    ) -> Self {
        // Hint: create `num_partitions` independent AggregateHashTable instances,
        // each with cloned copies of the type vectors.
        todo!()
    }

    /// Map a hash value to a partition index using modular arithmetic.
    pub fn partition_for_hash(&self, hash: u64) -> usize {
        hash as usize % self.num_partitions
    }

    /// Merge all per-partition tables and produce the final aggregated result chunks.
    pub fn merge_and_finalize(&self) -> Result<Vec<DataChunk>, String> {
        todo!()
    }
}
