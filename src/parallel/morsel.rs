//! Lesson 29: Morsel-Driven Parallelism
//!
//! Thread-safe morsel queue and parallel pipeline execution.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use std::sync::{Arc, Mutex};

/// A morsel: a chunk of data assigned to a worker thread.
pub struct Morsel {
    pub chunk: DataChunk,
    pub morsel_id: usize,
}

/// Thread-safe queue of morsels for parallel consumption.
pub struct MorselQueue {
    morsels: Mutex<Vec<Morsel>>,
    total_morsels: usize,
}

impl MorselQueue {
    /// Create a new morsel queue from data chunks.
    pub fn new(chunks: Vec<DataChunk>) -> Self {
        todo!()
    }

    /// Take the next morsel (thread-safe).
    pub fn take(&self) -> Option<Morsel> {
        todo!()
    }

    /// Number of remaining morsels.
    pub fn remaining(&self) -> usize {
        todo!()
    }

    /// Total morsels originally in the queue.
    pub fn total(&self) -> usize {
        self.total_morsels
    }
}

/// Result collector that aggregates results from parallel workers.
pub struct ParallelCollector {
    results: Mutex<Vec<DataChunk>>,
}

impl ParallelCollector {
    pub fn new() -> Self {
        Self { results: Mutex::new(Vec::new()) }
    }

    pub fn push(&self, chunk: DataChunk) {
        todo!()
    }

    pub fn into_results(self) -> Vec<DataChunk> {
        self.results.into_inner().unwrap()
    }
}

impl Default for ParallelCollector {
    fn default() -> Self {
        Self::new()
    }
}
