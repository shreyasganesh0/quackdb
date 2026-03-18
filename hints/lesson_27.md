# Lesson 27: MVCC

## What You're Building
A multi-version concurrency control system that provides snapshot isolation. Each row has a version chain (linked list via `Option<Box<VersionedRow>>`), and each transaction sees a consistent snapshot of the database as of its start time. The `TransactionManager` generates unique IDs using `AtomicU64`, tracks transaction status (active/committed/aborted), and records snapshots of active transactions. The `MvccTable` ties it all together with insert, delete, scan, commit, abort, and garbage collection.

## Concept Recap
Building on Lessons 1-8 (storage types): The `ScalarValue` type you built early on is now the data stored in each versioned row. The `Vec<ScalarValue>` row format is familiar from DataChunk. This lesson shifts from "how to compute on data" to "how to safely share data across concurrent transactions" -- the same rows, but with visibility rules layered on top.

## Rust Concepts You'll Need
- [Concurrency](../concepts/concurrency.md) -- `AtomicU64` with `Ordering::SeqCst` for thread-safe transaction ID generation
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- `prev_version: Option<Box<VersionedRow>>` forms a linked list of row versions
- [Collections](../concepts/collections.md) -- `HashMap<TxnId, Transaction>` for tracking transaction state, `Vec<VersionedRow>` for table storage

## Key Patterns

### Atomic Counter for ID Generation
Transaction IDs must be unique and monotonically increasing. An atomic counter achieves this without locks. Think of it like a number dispenser at a bakery -- each customer pulls the next number, and the counter itself is thread-safe.

```rust
// Analogy: a ticket dispenser at a deli counter (NOT the QuackDB solution)
use std::sync::atomic::{AtomicU64, Ordering};

struct TicketDispenser {
    next: AtomicU64,
}

impl TicketDispenser {
    fn new() -> Self { Self { next: AtomicU64::new(1) } }

    fn take_ticket(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }
}
```

### Version Chain via Box Linked List
Each update to a row creates a new version, linking back to the previous one. When scanning, you walk the chain to find the version visible to your transaction. This is a singly-linked list built with `Option<Box<T>>`. It is like a stack of sticky notes on a document -- each new edit is placed on top, but older edits are still underneath if you peel back.

```rust
// Analogy: document revision history (NOT the QuackDB solution)
struct Revision {
    content: String,
    author: u64,
    prev: Option<Box<Revision>>,
}

impl Revision {
    fn new(content: String, author: u64, prev: Option<Box<Revision>>) -> Self {
        Self { content, author, prev }
    }

    fn history_len(&self) -> usize {
        1 + self.prev.as_ref().map_or(0, |p| p.history_len())
    }
}
```

### Snapshot Isolation Visibility Rules
A row version is visible to transaction T if:
1. It was created by T itself (read-your-own-writes), OR
2. It was created by a transaction that was committed before T started (i.e., not in T's active snapshot)

AND the row has not been deleted, or if deleted, the deleting transaction is either T itself or was not yet committed when T started.

This works like a library catalog with time-stamped entries: you can see any book cataloged before your visit, plus any books you brought in yourself, but not books someone else is still processing.

```rust
// Analogy: deciding if a library book is available to a patron (NOT the QuackDB solution)
fn is_available(
    placed_by: u64,    // who put the book on the shelf
    removed_by: Option<u64>, // who checked it out (if anyone)
    patron: u64,       // the patron asking
    checked_out_patrons: &[u64], // patrons currently holding books
) -> bool {
    let placed_visible = placed_by == patron || !checked_out_patrons.contains(&placed_by);
    let not_removed = match removed_by {
        None => true,
        Some(r) => r != patron && checked_out_patrons.contains(&r),
    };
    placed_visible && not_removed
}
```

## Common Mistakes
- **Confusing "active" with "uncommitted".** A transaction in the active snapshot is one that was running when your transaction started. Even if it commits later, your snapshot should not see its changes. The snapshot is frozen at begin time.
- **Getting the delete visibility logic backwards.** A deleted row should be invisible if the deleter has committed (and is not in your active set). A common bug is making deleted rows visible when the deleter committed.
- **Forgetting read-your-own-writes for deletes.** If your transaction deletes a row, your own scan should NOT see that row anymore. The visibility check must handle the `deleted_by == current_txn` case.

## Step-by-Step Implementation Order
1. Start with `TransactionManager::new()` -- initialize `next_txn_id` as `AtomicU64::new(1)` and `transactions` as an empty HashMap.
2. Implement `begin()` -- fetch-and-increment the atomic counter, snapshot the currently active transaction IDs, create a `Transaction` struct with status `Active`, and insert it into the HashMap. Return the new ID.
3. Implement `commit()` and `abort()` -- look up the transaction, verify it is `Active`, and change its status. Return `Err` if the transaction does not exist or is not active.
4. Implement `status()` and `snapshot()` -- simple HashMap lookups.
5. Implement `MvccTable::new()` -- create empty rows Vec and a new TransactionManager.
6. Implement `insert()` -- verify the transaction is active, create a `VersionedRow` with `created_by` set to the txn ID and `deleted_by` as `None`, push it to `rows`, and return the row index.
7. Implement `is_visible()` on `VersionedRow` -- apply the visibility rules: the creator must be the current txn OR must not be in the active set, AND `deleted_by` must be `None` or the deleter must be in the active set (not yet committed) and not the current txn.
8. Implement `scan()` -- iterate over all rows, calling `is_visible()` with the transaction's snapshot, and collect visible rows' data.
9. Implement `delete()`, `garbage_collect()`, and edge cases.

## Reading the Tests
- **`test_transaction_manager`** creates two transactions, checks both are Active, commits one and aborts the other, then verifies their statuses. This validates the basic lifecycle: begin produces Active, commit changes to Committed, abort changes to Aborted.
- **`test_versioned_row_visibility`** directly tests `is_visible()`. A row created by txn 1 is visible to txn 2 when the active list is empty (txn 1 implicitly committed), but not visible when `[1]` is in the active list (txn 1 still running). This pins down the core visibility logic without involving the full table.
- **`test_begin_commit`** inserts a row in txn1, commits, then scans in txn2 and expects to see the row. This is the simplest end-to-end test: committed writes are visible to future transactions.
- **`test_snapshot_isolation`** is the key test. Txn1 inserts and commits. Txn2 starts. Txn3 inserts and commits. Txn2 scans and should see only txn1's row, not txn3's -- because txn3 started after txn2. This validates that your snapshot captures the active set at begin time.
- **`test_abort_not_visible`** inserts in txn1, aborts, then scans in txn2 and expects zero rows. This verifies the atomicity guarantee -- aborted transactions leave no trace.
- **`test_read_own_writes`** inserts in a transaction and scans within the same transaction before committing. It expects to see the uncommitted row. This tests the "created_by == current_txn" special case in visibility.
- **`test_delete`** inserts, commits, then deletes in a new transaction, commits, and verifies the row is gone. **`test_delete_not_visible_before_commit`** checks that an uncommitted delete does not affect concurrent transactions, and that even after the delete commits, the concurrent transaction's snapshot remains unchanged. These two tests together validate the full delete visibility lifecycle.
- **`test_concurrent_insert`** has two concurrent transactions each insert a row, both commit, and a third transaction sees both rows. This confirms that concurrent inserts to different rows do not interfere.
- **`test_garbage_collection`** inserts 10 rows, deletes the first 5, and expects `garbage_collect()` to return a positive count. This validates that your GC identifies rows that are deleted by committed transactions and no longer needed by any active transaction.
