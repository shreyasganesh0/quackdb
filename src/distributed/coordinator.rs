//! Lesson 33: Distributed Executor Coordinator
//!
//! Coordinates distributed query execution across threads.

use super::planner::PlanFragment;
use super::shuffle::{ExchangeChannel, ExchangeSender, ExchangeReceiver};
use crate::chunk::DataChunk;
use crate::planner::catalog::Catalog;

/// Distributed executor that spawns threads and connects them via channels.
pub struct DistributedExecutor {
    num_workers: usize,
}

impl DistributedExecutor {
    pub fn new(num_workers: usize) -> Self {
        Self { num_workers }
    }

    /// Execute a distributed query plan.
    pub fn execute(
        &self,
        fragments: Vec<PlanFragment>,
        catalog: &Catalog,
    ) -> Result<Vec<DataChunk>, String> {
        todo!()
    }
}
