//! Lesson 27: Multi-Version Concurrency Control
//!
//! Implements snapshot isolation via versioned rows. Each write creates a new
//! row version tagged with the writing transaction's ID; readers see only
//! versions committed before their snapshot timestamp. Old versions are
//! eventually reclaimed by garbage collection.

use crate::types::ScalarValue;
use std::collections::HashMap;
// Atomic counter for generating monotonically increasing transaction IDs.
use std::sync::atomic::{AtomicU64, Ordering};

/// A unique, monotonically increasing transaction identifier.
pub type TxnId = u64;

/// Lifecycle status of a transaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TxnStatus {
    /// Transaction is in progress and may still read/write.
    Active,
    /// Transaction has been committed; its writes are visible to newer snapshots.
    Committed,
    /// Transaction has been rolled back; its writes must be ignored.
    Aborted,
}

/// A row version carrying MVCC metadata.
///
/// Versions form a singly-linked list (newest to oldest) via `prev_version`,
/// enabling readers to walk back to the version visible at their snapshot.
#[derive(Debug, Clone)]
pub struct VersionedRow {
    /// The actual column values for this version.
    pub data: Vec<ScalarValue>,
    /// The transaction that created this version.
    pub created_by: TxnId,
    /// The transaction that logically deleted this version, if any.
    pub deleted_by: Option<TxnId>,
    /// Pointer to the previous (older) version of the same row.
    // Box<VersionedRow> heap-allocates the older version to avoid
    // infinite struct size.
    pub prev_version: Option<Box<VersionedRow>>,
}

impl VersionedRow {
    /// Check if this row version is visible to transaction `txn_id`.
    ///
    /// A row is visible if:
    /// 1. `created_by` is committed and not in `active_txns` (or is `txn_id` itself), AND
    /// 2. `deleted_by` is `None` or was not yet committed at snapshot time.
    pub fn is_visible(&self, txn_id: TxnId, active_txns: &[TxnId]) -> bool {
        todo!()
    }
}

/// In-flight state of a single transaction.
#[derive(Debug)]
pub struct Transaction {
    pub id: TxnId,
    pub status: TxnStatus,
    /// The timestamp at which this transaction's snapshot was taken.
    pub start_ts: TxnId,
    /// IDs of transactions that were active when this transaction began.
    /// Writes by these transactions are invisible to this snapshot.
    pub snapshot: Vec<TxnId>,
}

/// Manages transaction lifecycles: begin, commit, abort, and snapshot queries.
pub struct TransactionManager {
    // AtomicU64 allows lock-free ID generation across threads.
    next_txn_id: AtomicU64,
    transactions: HashMap<TxnId, Transaction>,
}

impl TransactionManager {
    /// Create a new manager with no active transactions.
    pub fn new() -> Self {
        Self {
            next_txn_id: AtomicU64::new(1),
            transactions: HashMap::new(),
        }
    }

    /// Begin a new transaction, capturing a snapshot of currently active transactions.
    ///
    /// Returns the new transaction's ID.
    pub fn begin(&mut self) -> TxnId {
        let id = self.next_txn_id.fetch_add(1, Ordering::SeqCst);
        let snapshot: Vec<TxnId> = self.transactions.iter()
            .filter(|(_, txn)| txn.status == TxnStatus::Active)
            .map(|(&tid, _)| tid)
            .collect();
        self.transactions.insert(id, Transaction {
            id,
            status: TxnStatus::Active,
            start_ts: id,
            snapshot,
        });
        id
    }

    /// Mark a transaction as committed.
    ///
    /// After commit, the transaction's writes become visible to all newer snapshots.
    pub fn commit(&mut self, txn_id: TxnId) -> Result<(), String> {
        match self.transactions.get_mut(&txn_id) {
            Some(txn) if txn.status == TxnStatus::Active => {
                txn.status = TxnStatus::Committed;
                Ok(())
            }
            Some(_) => Err(format!("Transaction {} is not active", txn_id)),
            None => Err(format!("Transaction {} not found", txn_id)),
        }
    }

    /// Mark a transaction as aborted (rolled back).
    ///
    /// The transaction's writes should be treated as invisible by all readers.
    pub fn abort(&mut self, txn_id: TxnId) -> Result<(), String> {
        match self.transactions.get_mut(&txn_id) {
            Some(txn) if txn.status == TxnStatus::Active => {
                txn.status = TxnStatus::Aborted;
                Ok(())
            }
            Some(_) => Err(format!("Transaction {} is not active", txn_id)),
            None => Err(format!("Transaction {} not found", txn_id)),
        }
    }

    /// Query the current status of a transaction.
    pub fn status(&self, txn_id: TxnId) -> Option<TxnStatus> {
        self.transactions.get(&txn_id).map(|txn| txn.status)
    }

    /// Return the snapshot (list of active txn IDs at begin time) for `txn_id`.
    pub fn snapshot(&self, txn_id: TxnId) -> Option<&[TxnId]> {
        self.transactions.get(&txn_id).map(|txn| txn.snapshot.as_slice())
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// An MVCC-enabled table that stores versioned rows and manages transactions.
pub struct MvccTable {
    rows: Vec<VersionedRow>,
    txn_manager: TransactionManager,
}

impl MvccTable {
    /// Create an empty MVCC table.
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            txn_manager: TransactionManager::new(),
        }
    }

    /// Begin a new transaction and return its ID.
    pub fn begin_transaction(&mut self) -> TxnId {
        self.txn_manager.begin()
    }

    /// Insert a new row within the given transaction.
    ///
    /// Returns the row ID (index in the `rows` vector) on success.
    pub fn insert(&mut self, txn_id: TxnId, data: Vec<ScalarValue>) -> Result<usize, String> {
        let row = VersionedRow {
            data,
            created_by: txn_id,
            deleted_by: None,
            prev_version: None,
        };
        let row_id = self.rows.len();
        self.rows.push(row);
        Ok(row_id)
    }

    /// Logically delete a row within the given transaction.
    ///
    /// Sets `deleted_by` on the current version rather than physically removing it.
    pub fn delete(&mut self, txn_id: TxnId, row_id: usize) -> Result<(), String> {
        if row_id >= self.rows.len() {
            return Err(format!("Row {} does not exist", row_id));
        }
        self.rows[row_id].deleted_by = Some(txn_id);
        Ok(())
    }

    /// Scan all rows visible to the given transaction.
    ///
    /// For each row, walk the version chain until a visible version is found.
    pub fn scan(&self, txn_id: TxnId) -> Vec<Vec<ScalarValue>> {
        todo!()
    }

    /// Commit a transaction, making its writes durable and visible.
    pub fn commit(&mut self, txn_id: TxnId) -> Result<(), String> {
        self.txn_manager.commit(txn_id)
    }

    /// Abort a transaction, discarding its writes.
    pub fn abort(&mut self, txn_id: TxnId) -> Result<(), String> {
        self.txn_manager.abort(txn_id)
    }

    /// Run garbage collection, removing row versions that are no longer
    /// visible to any active transaction.
    ///
    /// Returns the number of versions reclaimed.
    pub fn garbage_collect(&mut self) -> usize {
        // Hint: find the oldest active transaction's start_ts; any version
        // older than that with no active reader can be unlinked.
        todo!()
    }
}

impl Default for MvccTable {
    fn default() -> Self {
        Self::new()
    }
}
