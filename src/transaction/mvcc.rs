//! Lesson 27: Multi-Version Concurrency Control
//!
//! Snapshot isolation via versioned rows.

use crate::types::ScalarValue;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Transaction ID.
pub type TxnId = u64;

/// Transaction status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TxnStatus {
    Active,
    Committed,
    Aborted,
}

/// A versioned row with MVCC metadata.
#[derive(Debug, Clone)]
pub struct VersionedRow {
    pub data: Vec<ScalarValue>,
    pub created_by: TxnId,
    pub deleted_by: Option<TxnId>,
    pub prev_version: Option<Box<VersionedRow>>,
}

impl VersionedRow {
    /// Check if this row is visible to the given transaction.
    pub fn is_visible(&self, txn_id: TxnId, active_txns: &[TxnId]) -> bool {
        todo!()
    }
}

/// Transaction state.
#[derive(Debug)]
pub struct Transaction {
    pub id: TxnId,
    pub status: TxnStatus,
    pub start_ts: TxnId,
    pub snapshot: Vec<TxnId>,
}

/// Transaction manager.
pub struct TransactionManager {
    next_txn_id: AtomicU64,
    transactions: HashMap<TxnId, Transaction>,
}

impl TransactionManager {
    pub fn new() -> Self {
        todo!()
    }

    /// Begin a new transaction.
    pub fn begin(&mut self) -> TxnId {
        todo!()
    }

    /// Commit a transaction.
    pub fn commit(&mut self, txn_id: TxnId) -> Result<(), String> {
        todo!()
    }

    /// Abort a transaction.
    pub fn abort(&mut self, txn_id: TxnId) -> Result<(), String> {
        todo!()
    }

    /// Get the status of a transaction.
    pub fn status(&self, txn_id: TxnId) -> Option<TxnStatus> {
        todo!()
    }

    /// Get the snapshot for a transaction.
    pub fn snapshot(&self, txn_id: TxnId) -> Option<&[TxnId]> {
        todo!()
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// MVCC table with versioned rows.
pub struct MvccTable {
    rows: Vec<VersionedRow>,
    txn_manager: TransactionManager,
}

impl MvccTable {
    pub fn new() -> Self {
        todo!()
    }

    /// Begin a transaction.
    pub fn begin_transaction(&mut self) -> TxnId {
        todo!()
    }

    /// Insert a row within a transaction.
    pub fn insert(&mut self, txn_id: TxnId, data: Vec<ScalarValue>) -> Result<usize, String> {
        todo!()
    }

    /// Delete a row within a transaction.
    pub fn delete(&mut self, txn_id: TxnId, row_id: usize) -> Result<(), String> {
        todo!()
    }

    /// Read all visible rows for a transaction.
    pub fn scan(&self, txn_id: TxnId) -> Vec<Vec<ScalarValue>> {
        todo!()
    }

    /// Commit a transaction.
    pub fn commit(&mut self, txn_id: TxnId) -> Result<(), String> {
        todo!()
    }

    /// Abort a transaction.
    pub fn abort(&mut self, txn_id: TxnId) -> Result<(), String> {
        todo!()
    }

    /// Run garbage collection, removing old versions.
    pub fn garbage_collect(&mut self) -> usize {
        todo!()
    }
}

impl Default for MvccTable {
    fn default() -> Self {
        Self::new()
    }
}
