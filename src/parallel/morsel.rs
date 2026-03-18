//! # Lesson 29: Parallel Execution — Morsel Queue (File 1 of 2)
//!
//! This file implements the morsel-driven data structures: `MorselQueue` (a
//! thread-safe queue of data chunks) and `ParallelCollector` (a thread-safe
//! result aggregator). Morsels are fixed-size chunks distributed to worker
//! threads on demand, achieving dynamic load balancing.
//!
//! It works together with:
//! - `scheduler.rs` — the parallel pipeline scheduler that spawns worker threads,
//!   pulls morsels from the `MorselQueue`, and pushes results to the
//!   `ParallelCollector`.
//!
//! **Start here**: Implement `morsel.rs` first, then `scheduler.rs`. The
//! scheduler depends on `MorselQueue::take()` and `ParallelCollector::push()`
//! to drive the parallel execution loop.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use std::sync::{Arc, Mutex};

/// A morsel: a single DataChunk assigned to a worker thread for processing.
pub struct Morsel {
    /// The data to process.
    pub chunk: DataChunk,
    /// A unique identifier for this morsel (used for debugging/ordering).
    pub morsel_id: usize,
}

/// Thread-safe queue of morsels for parallel consumption.
///
/// Workers call [`take`] in a loop until `None` is returned, meaning all
/// morsels have been consumed. The internal `Mutex` ensures safe concurrent access.
pub struct MorselQueue {
    // Mutex<Vec<...>> is the simplest thread-safe queue; for production you
    // might use a lock-free queue, but Mutex is fine for learning.
    morsels: Mutex<Vec<Morsel>>,
    total_morsels: usize,
}

impl MorselQueue {
    /// Create a new morsel queue by wrapping each `DataChunk` as a `Morsel`.
    ///
    /// Assigns sequential `morsel_id` values starting at 0.
    pub fn new(chunks: Vec<DataChunk>) -> Self {
        let total_morsels = chunks.len();
        let morsels = chunks
            .into_iter()
            .enumerate()
            .map(|(i, chunk)| Morsel { chunk, morsel_id: i })
            .collect();
        Self {
            morsels: Mutex::new(morsels),
            total_morsels,
        }
    }

    /// Take the next available morsel (FIFO). Returns `None` when the queue is empty.
    ///
    /// This is the hot path called by every worker thread.
    pub fn take(&self) -> Option<Morsel> {
        let mut morsels = self.morsels.lock().unwrap();
        morsels.pop()
    }

    /// Number of morsels remaining in the queue.
    pub fn remaining(&self) -> usize {
        self.morsels.lock().unwrap().len()
    }

    /// Total number of morsels originally enqueued.
    pub fn total(&self) -> usize {
        self.total_morsels
    }
}

/// Thread-safe collector that aggregates result chunks from parallel workers.
///
/// Each worker pushes its output here; after all workers finish, call
/// `into_results` to retrieve the combined output.
pub struct ParallelCollector {
    // Mutex protects concurrent pushes from multiple worker threads.
    results: Mutex<Vec<DataChunk>>,
}

impl ParallelCollector {
    /// Create an empty collector.
    pub fn new() -> Self {
        Self { results: Mutex::new(Vec::new()) }
    }

    /// Push a result chunk (called by worker threads).
    pub fn push(&self, chunk: DataChunk) {
        self.results.lock().unwrap().push(chunk);
    }

    /// Consume the collector and return all collected result chunks.
    ///
    /// `into_inner()` unwraps the Mutex, which is safe because we have
    /// exclusive ownership (no other references exist).
    pub fn into_results(self) -> Vec<DataChunk> {
        self.results.into_inner().unwrap()
    }
}

impl Default for ParallelCollector {
    fn default() -> Self {
        Self::new()
    }
}
