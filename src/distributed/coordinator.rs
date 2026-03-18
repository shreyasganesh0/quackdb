//! Lesson 33: Distributed Executor Coordinator
//!
//! The coordinator receives a distributed plan (a set of `PlanFragment`s),
//! sets up exchange channels between fragments, spawns worker threads, and
//! collects the final results. This is the top-level entry point for
//! distributed query execution.

use super::planner::PlanFragment;
use super::shuffle::{ExchangeChannel, ExchangeSender, ExchangeReceiver};
use crate::chunk::DataChunk;
use crate::planner::catalog::Catalog;

/// Coordinates execution of distributed query plans.
///
/// Each plan fragment runs on its own thread; fragments communicate via
/// `ExchangeChannel` pairs (sender/receiver). The coordinator thread
/// collects the final Gather output.
pub struct DistributedExecutor {
    num_workers: usize,
}

impl DistributedExecutor {
    /// Create an executor targeting `num_workers` parallel threads.
    pub fn new(num_workers: usize) -> Self {
        Self { num_workers }
    }

    /// Execute a distributed query plan and return the result chunks.
    ///
    /// Steps:
    /// 1. Create exchange channels between fragments.
    /// 2. Spawn a thread per fragment, passing it the fragment plan,
    ///    its sender(s), and receiver(s).
    /// 3. The root fragment gathers results via its receiver channel.
    /// 4. Join all threads and return collected chunks.
    pub fn execute(
        &self,
        fragments: Vec<PlanFragment>,
        catalog: &Catalog,
    ) -> Result<Vec<DataChunk>, String> {
        // Hint: use `ExchangeChannel::new()` to create (sender, receiver)
        // pairs. Use `std::thread::scope` to spawn scoped threads that
        // borrow `catalog`.
        todo!()
    }
}
