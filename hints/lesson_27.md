# Lesson 27: MVCC

## What You're Building
A multi-version concurrency control system that provides snapshot isolation. Each row has a version chain (linked list via `Option<Box<VersionedRow>>`), and each transaction sees a consistent snapshot of the database as of its start time. The `TransactionManager` generates unique IDs using `AtomicU64`, tracks transaction status (active/committed/aborted), and records snapshots of active transactions. The `MvccTable` ties it all together with insert, delete, scan, commit, abort, and garbage collection.

## Rust Concepts You'll Need
- [Concurrency](../concepts/concurrency.md) -- `AtomicU64` with `Ordering::SeqCst` for thread-safe transaction ID generation
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- `prev_version: Option<Box<VersionedRow>>` forms a linked list of row versions
- [Collections](../concepts/collections.md) -- `HashMap<TxnId, Transaction>` for tracking transaction state, `Vec<VersionedRow>` for table storage

## Key Patterns

### Atomic Counter for ID Generation
Transaction IDs must be unique and monotonically increasing. An atomic counter achieves this without locks.

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
Each update to a row creates a new version, linking back to the previous one. When scanning, you walk the chain to find the version visible to your transaction. This is a singly-linked list built with `Option<Box<T>>`.

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

## Step-by-Step Implementation Order
1. Start with `TransactionManager::new()` -- initialize `next_txn_id` as `AtomicU64::new(1)` and `transactions` as an empty HashMap.
2. Implement `begin()` -- fetch-and-increment the atomic counter, snapshot the currently active transaction IDs, create a `Transaction` struct with status `Active`, and insert it into the HashMap. Return the new ID.
3. Implement `commit()` and `abort()` -- look up the transaction, verify it is `Active`, and change its status. Return `Err` if the transaction does not exist or is not active.
4. Implement `status()` and `snapshot()` -- simple HashMap lookups.
5. Implement `MvccTable::new()` -- create empty rows Vec and a new TransactionManager.
6. Implement `insert()` -- verify the transaction is active, create a `VersionedRow` with `created_by` set to the txn ID and `deleted_by` as `None`, push it to `rows`, and return the row index.
7. Implement `delete()` -- set `deleted_by` on the target row to `Some(txn_id)`. Check that the transaction is active and the row exists.
8. Implement `is_visible()` on `VersionedRow` -- apply the visibility rules: the creator must be the current txn OR must not be in the active set, AND `deleted_by` must be `None` or the deleter must be in the active set (not yet committed) and not the current txn.
9. Implement `scan()` -- iterate over all rows, calling `is_visible()` with the transaction's snapshot, and collect visible rows' data.
10. Implement `garbage_collect()` -- remove rows that are deleted by committed transactions and have no active transaction that could still see them. Count how many were cleaned.
11. Watch out for "read your own writes" -- a transaction must see its own uncommitted inserts and must not see its own uncommitted deletes (or rather, it should see the delete effect).

## Reading the Tests
- **`test_snapshot_isolation`** is the key test. Txn1 inserts and commits. Txn2 starts. Txn3 inserts and commits. Txn2 scans and should see only txn1's row, not txn3's -- because txn3 started after txn2. This validates that your snapshot captures the active set at begin time.
- **`test_versioned_row_visibility`** directly tests `is_visible()`. A row created by txn 1 is visible to txn 2 when the active list is empty (txn 1 implicitly committed), but not visible when `[1]` is in the active list (txn 1 still running). This pins down the core visibility logic.
- **`test_garbage_collection`** inserts 10 rows, deletes the first 5, and expects `garbage_collect()` to return a positive count.
