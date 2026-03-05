//! Lesson 28: Write-Ahead Logging
//!
//! WAL for crash recovery with analysis/redo/undo phases.

use crate::types::ScalarValue;
use super::mvcc::TxnId;
use std::io::{Read, Write, Seek};

/// WAL record types.
#[derive(Debug, Clone, PartialEq)]
pub enum WalRecord {
    Begin { txn_id: TxnId },
    Insert { txn_id: TxnId, table: String, row_id: u64, data: Vec<ScalarValue> },
    Delete { txn_id: TxnId, table: String, row_id: u64 },
    Commit { txn_id: TxnId },
    Abort { txn_id: TxnId },
    Checkpoint { active_txns: Vec<TxnId> },
}

/// Log sequence number.
pub type Lsn = u64;

/// A WAL record with its LSN.
#[derive(Debug, Clone)]
pub struct WalEntry {
    pub lsn: Lsn,
    pub record: WalRecord,
}

impl WalEntry {
    /// Serialize to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        todo!()
    }
}

/// WAL writer: appends records to a log file.
pub struct WalWriter<W: Write> {
    writer: W,
    next_lsn: Lsn,
}

impl<W: Write> WalWriter<W> {
    pub fn new(writer: W) -> Self {
        todo!()
    }

    /// Append a record to the WAL. Returns the LSN.
    pub fn append(&mut self, record: WalRecord) -> Result<Lsn, String> {
        todo!()
    }

    /// Flush the WAL to durable storage.
    pub fn flush(&mut self) -> Result<(), String> {
        todo!()
    }
}

/// WAL reader: reads records from a log file.
pub struct WalReader<R: Read> {
    reader: R,
}

impl<R: Read> WalReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Read the next WAL entry.
    pub fn next(&mut self) -> Result<Option<WalEntry>, String> {
        todo!()
    }

    /// Read all entries.
    pub fn read_all(&mut self) -> Result<Vec<WalEntry>, String> {
        todo!()
    }
}

/// Recovery manager: replays WAL for crash recovery.
pub struct RecoveryManager;

impl RecoveryManager {
    /// Recover from a WAL, returning (committed_txns, operations_to_apply).
    pub fn recover<R: Read>(reader: R) -> Result<RecoveryResult, String> {
        todo!()
    }
}

/// Result of WAL recovery.
#[derive(Debug)]
pub struct RecoveryResult {
    pub committed: Vec<TxnId>,
    pub aborted: Vec<TxnId>,
    pub redo_ops: Vec<WalRecord>,
    pub undo_ops: Vec<WalRecord>,
}
