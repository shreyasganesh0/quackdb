//! # Lesson 33: Distributed Execution — Shuffle & Data Movement (File 1 of 2)
//!
//! This file implements the data movement primitives for distributed execution:
//! channel-based communication (sender/receiver pairs), plus shuffle, broadcast,
//! gather, and ordered-gather operators that route `DataChunk`s between plan
//! fragments.
//!
//! It works together with:
//! - `coordinator.rs` — the distributed executor that creates exchange channels
//!   defined here, wires them between plan fragments, and spawns worker threads.
//!
//! **Start here**: Implement `shuffle.rs` first, then `coordinator.rs`. The
//! coordinator uses `ExchangeChannel::new()` to create sender/receiver pairs
//! and passes them to the operators defined in this file.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use crate::execution::pipeline::{OperatorResult, PhysicalOperator};
use std::sync::mpsc;

/// Factory for creating paired exchange sender/receiver endpoints.
///
/// Internally wraps `std::sync::mpsc` channels. A `None` sentinel signals
/// end-of-stream.
pub struct ExchangeChannel {
    sender: mpsc::Sender<Option<DataChunk>>,
    receiver: mpsc::Receiver<Option<DataChunk>>,
}

impl ExchangeChannel {
    /// Create a new sender/receiver pair.
    ///
    /// Returns `(ExchangeSender, ExchangeReceiver)` connected by an mpsc channel.
    pub fn new() -> (ExchangeSender, ExchangeReceiver) {
        let (sender, receiver) = mpsc::channel();
        (
            ExchangeSender { sender },
            ExchangeReceiver { receiver },
        )
    }
}

/// Sender half of an exchange channel.
///
/// Sends `DataChunk`s to the paired `ExchangeReceiver`. Call `close()` when done.
pub struct ExchangeSender {
    sender: mpsc::Sender<Option<DataChunk>>,
}

impl ExchangeSender {
    /// Send a data chunk to the receiver.
    pub fn send(&self, chunk: DataChunk) -> Result<(), String> {
        self.sender.send(Some(chunk)).map_err(|e| e.to_string())
    }

    /// Signal end-of-stream by sending `None`, then drop the sender.
    pub fn close(self) {
        let _ = self.sender.send(None);
    }
}

/// Receiver half of an exchange channel.
///
/// Returns `None` from `recv()` when the sender has closed.
pub struct ExchangeReceiver {
    receiver: mpsc::Receiver<Option<DataChunk>>,
}

impl ExchangeReceiver {
    /// Receive the next chunk, or `None` if the stream is finished.
    pub fn recv(&self) -> Option<DataChunk> {
        match self.receiver.recv() {
            Ok(Some(chunk)) => Some(chunk),
            Ok(None) | Err(_) => None,
        }
    }
}

/// Shuffle operator: hash-partitions each incoming chunk across multiple senders.
///
/// Used for repartitioning data before a distributed hash join or aggregation.
pub struct ShuffleOperator {
    /// Column indices used to compute the partition hash.
    partition_columns: Vec<usize>,
    /// One sender per target partition.
    senders: Vec<ExchangeSender>,
    output_types: Vec<LogicalType>,
}

impl ShuffleOperator {
    /// Create a shuffle operator.
    pub fn new(partition_columns: Vec<usize>, senders: Vec<ExchangeSender>, output_types: Vec<LogicalType>) -> Self {
        Self { partition_columns, senders, output_types }
    }
}

/// Gather operator: reads from multiple input channels and emits chunks in
/// arrival order (non-deterministic).
pub struct GatherOperator {
    receivers: Vec<ExchangeReceiver>,
    output_types: Vec<LogicalType>,
    /// Index of the receiver to try next (round-robin).
    current: usize,
}

impl GatherOperator {
    /// Create a gather operator from a set of receivers.
    pub fn new(receivers: Vec<ExchangeReceiver>, output_types: Vec<LogicalType>) -> Self {
        Self { receivers, output_types, current: 0 }
    }

    /// Pull the next available chunk from any receiver.
    ///
    /// Returns `None` when all receivers are exhausted.
    pub fn next_chunk(&mut self) -> Option<DataChunk> {
        // Hint: round-robin through receivers, skipping closed ones.
        todo!()
    }
}

/// Broadcast operator: sends each chunk to *all* receivers.
///
/// Used when a small table must be available on every node (e.g., broadcast
/// join for dimension tables).
pub struct BroadcastOperator {
    senders: Vec<ExchangeSender>,
    output_types: Vec<LogicalType>,
}

impl BroadcastOperator {
    /// Create a broadcast operator targeting all senders.
    pub fn new(senders: Vec<ExchangeSender>, output_types: Vec<LogicalType>) -> Self {
        Self { senders, output_types }
    }

    /// Clone and send the chunk to every receiver.
    pub fn broadcast(&self, chunk: &DataChunk) -> Result<(), String> {
        for sender in &self.senders {
            sender.send(chunk.clone())?;
        }
        Ok(())
    }
}

/// Ordered gather: k-way merge from pre-sorted input channels.
///
/// Maintains a priority queue (min-heap) of the front element from each
/// receiver to produce globally sorted output.
pub struct OrderedGather {
    receivers: Vec<ExchangeReceiver>,
    /// Column indices to compare for merge ordering.
    sort_columns: Vec<usize>,
    output_types: Vec<LogicalType>,
}

impl OrderedGather {
    /// Create an ordered gather merging from the given receivers.
    pub fn new(receivers: Vec<ExchangeReceiver>, sort_columns: Vec<usize>, output_types: Vec<LogicalType>) -> Self {
        Self { receivers, sort_columns, output_types }
    }

    /// Return the next chunk in globally sorted order.
    ///
    /// Uses a k-way merge: peek at the front row of each receiver's buffer,
    /// pick the smallest, and emit it.
    pub fn next_chunk(&mut self) -> Option<DataChunk> {
        // Hint: use a BinaryHeap or manual min-selection across receiver fronts.
        todo!()
    }
}
