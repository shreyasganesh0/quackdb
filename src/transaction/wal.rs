//! Lesson 28: Write-Ahead Logging
//!
//! Implements a WAL for crash recovery. Every mutation is logged *before* being
//! applied to the database. On crash, the recovery manager replays the log
//! using the classic ARIES-style analysis/redo/undo phases to restore the
//! database to a consistent state.

use crate::types::ScalarValue;
use super::mvcc::TxnId;
use std::io::{Read, Write, Seek};

/// The different kinds of records that can appear in the WAL.
#[derive(Debug, Clone, PartialEq)]
pub enum WalRecord {
    /// Transaction begin marker.
    Begin { txn_id: TxnId },
    /// A row insertion.
    Insert { txn_id: TxnId, table: String, row_id: u64, data: Vec<ScalarValue> },
    /// A row deletion.
    Delete { txn_id: TxnId, table: String, row_id: u64 },
    /// Transaction commit marker -- the durability guarantee point.
    Commit { txn_id: TxnId },
    /// Transaction abort marker.
    Abort { txn_id: TxnId },
    /// Checkpoint: snapshot of active transactions at this point.
    Checkpoint { active_txns: Vec<TxnId> },
}

/// Log Sequence Number -- monotonically increasing, uniquely identifies a WAL entry.
pub type Lsn = u64;

/// A WAL entry pairing a record with its LSN.
#[derive(Debug, Clone)]
pub struct WalEntry {
    /// Position of this entry in the log sequence.
    pub lsn: Lsn,
    /// The logged record.
    pub record: WalRecord,
}

impl WalEntry {
    /// Serialize this entry to a byte vector for on-disk storage.
    ///
    /// The format should be self-describing so `from_bytes` can reconstruct it.
    pub fn to_bytes(&self) -> Vec<u8> {
        // Hint: consider a simple format -- 8 bytes LSN, 1 byte tag for the
        // variant, then variant-specific payload. Use little-endian encoding.
        todo!()
    }

    /// Deserialize a WAL entry from a byte slice.
    ///
    /// Returns an error if the bytes are malformed or truncated.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        todo!()
    }
}

/// Appends WAL records sequentially to an underlying writer (file or buffer).
///
/// The generic `W: Write` parameter lets you swap in a `Vec<u8>` for testing
/// or a `BufWriter<File>` for production.
pub struct WalWriter<W: Write> {
    writer: W,
    next_lsn: Lsn,
}

// Generic impl over any `W: Write` -- the trait bound ensures we can
// write bytes to the underlying storage.
impl<W: Write> WalWriter<W> {
    /// Create a new writer starting at LSN 0.
    pub fn new(writer: W) -> Self {
        todo!()
    }

    /// Append a record to the WAL and return its assigned LSN.
    ///
    /// Each call increments `next_lsn` by one.
    pub fn append(&mut self, record: WalRecord) -> Result<Lsn, String> {
        // Hint: serialize via `WalEntry::to_bytes`, write length-prefix + bytes,
        // then increment and return the LSN.
        todo!()
    }

    /// Flush all buffered data to durable storage.
    ///
    /// Must be called after a Commit record to guarantee durability.
    pub fn flush(&mut self) -> Result<(), String> {
        todo!()
    }
}

/// Reads WAL entries sequentially from an underlying reader.
///
/// Generic over `R: Read` for the same testability reasons as `WalWriter`.
pub struct WalReader<R: Read> {
    reader: R,
}

// Generic impl over any `R: Read`.
impl<R: Read> WalReader<R> {
    /// Wrap a reader as a WAL reader.
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Read the next WAL entry, or `None` if the log is exhausted.
    pub fn next(&mut self) -> Result<Option<WalEntry>, String> {
        // Hint: read the length prefix first; if 0 bytes are available,
        // return Ok(None).
        todo!()
    }

    /// Read all remaining entries into a vector.
    pub fn read_all(&mut self) -> Result<Vec<WalEntry>, String> {
        todo!()
    }
}

/// Replays a WAL to determine which operations must be redone or undone
/// after a crash.
pub struct RecoveryManager;

impl RecoveryManager {
    /// Perform crash recovery by scanning the WAL.
    ///
    /// 1. **Analysis**: identify committed and aborted transactions.
    /// 2. **Redo**: collect operations from committed transactions.
    /// 3. **Undo**: collect inverse operations for active (uncommitted) transactions.
    pub fn recover<R: Read>(reader: R) -> Result<RecoveryResult, String> {
        // Hint: first pass -- read all entries, track txn status.
        // Second pass -- partition operations into redo (committed) and
        // undo (active/aborted) sets.
        todo!()
    }
}

/// Outcome of WAL recovery, listing transactions and their required actions.
#[derive(Debug)]
pub struct RecoveryResult {
    /// Transactions that committed before the crash.
    pub committed: Vec<TxnId>,
    /// Transactions that were active or explicitly aborted.
    pub aborted: Vec<TxnId>,
    /// Operations from committed transactions that must be re-applied.
    pub redo_ops: Vec<WalRecord>,
    /// Operations from uncommitted transactions that must be reversed.
    pub undo_ops: Vec<WalRecord>,
}
