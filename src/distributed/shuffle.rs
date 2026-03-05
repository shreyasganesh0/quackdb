//! Lesson 33: Shuffle & Exchange Operators
//!
//! Data movement operators for distributed execution.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use crate::execution::pipeline::{OperatorResult, PhysicalOperator};
use std::sync::mpsc;

/// An exchange channel for sending chunks between operators.
pub struct ExchangeChannel {
    sender: mpsc::Sender<Option<DataChunk>>,
    receiver: mpsc::Receiver<Option<DataChunk>>,
}

impl ExchangeChannel {
    pub fn new() -> (ExchangeSender, ExchangeReceiver) {
        todo!()
    }
}

/// Sender half of an exchange channel.
pub struct ExchangeSender {
    sender: mpsc::Sender<Option<DataChunk>>,
}

impl ExchangeSender {
    pub fn send(&self, chunk: DataChunk) -> Result<(), String> {
        todo!()
    }

    pub fn close(self) {
        let _ = self.sender.send(None);
    }
}

/// Receiver half of an exchange channel.
pub struct ExchangeReceiver {
    receiver: mpsc::Receiver<Option<DataChunk>>,
}

impl ExchangeReceiver {
    pub fn recv(&self) -> Option<DataChunk> {
        todo!()
    }
}

/// Shuffle operator: routes chunks to partitions via hash.
pub struct ShuffleOperator {
    partition_columns: Vec<usize>,
    senders: Vec<ExchangeSender>,
    output_types: Vec<LogicalType>,
}

impl ShuffleOperator {
    pub fn new(partition_columns: Vec<usize>, senders: Vec<ExchangeSender>, output_types: Vec<LogicalType>) -> Self {
        Self { partition_columns, senders, output_types }
    }
}

/// Gather operator: collects from multiple input channels.
pub struct GatherOperator {
    receivers: Vec<ExchangeReceiver>,
    output_types: Vec<LogicalType>,
    current: usize,
}

impl GatherOperator {
    pub fn new(receivers: Vec<ExchangeReceiver>, output_types: Vec<LogicalType>) -> Self {
        todo!()
    }

    pub fn next_chunk(&mut self) -> Option<DataChunk> {
        todo!()
    }
}

/// Broadcast operator: sends each chunk to all receivers.
pub struct BroadcastOperator {
    senders: Vec<ExchangeSender>,
    output_types: Vec<LogicalType>,
}

impl BroadcastOperator {
    pub fn new(senders: Vec<ExchangeSender>, output_types: Vec<LogicalType>) -> Self {
        Self { senders, output_types }
    }

    pub fn broadcast(&self, chunk: &DataChunk) -> Result<(), String> {
        todo!()
    }
}

/// Ordered gather: k-way merge from sorted inputs.
pub struct OrderedGather {
    receivers: Vec<ExchangeReceiver>,
    sort_columns: Vec<usize>,
    output_types: Vec<LogicalType>,
}

impl OrderedGather {
    pub fn new(receivers: Vec<ExchangeReceiver>, sort_columns: Vec<usize>, output_types: Vec<LogicalType>) -> Self {
        Self { receivers, sort_columns, output_types }
    }

    pub fn next_chunk(&mut self) -> Option<DataChunk> {
        todo!()
    }
}
