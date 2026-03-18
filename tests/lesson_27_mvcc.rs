//! Lesson 27: MVCC Tests

use quackdb::types::ScalarValue;
use quackdb::transaction::mvcc::*;

#[test]
fn test_begin_commit() {
    let mut table = MvccTable::new();
    let txn = table.begin_transaction();
    table.insert(txn, vec![ScalarValue::Int32(1)]).unwrap();
    table.commit(txn).unwrap();

    let txn2 = table.begin_transaction();
    let rows = table.scan(txn2);
    assert_eq!(rows.len(), 1, "committed insert should be visible to subsequent transactions");
    assert_eq!(rows[0][0], ScalarValue::Int32(1));
}

#[test]
fn test_snapshot_isolation() {
    let mut table = MvccTable::new();

    // txn1 inserts a row
    let txn1 = table.begin_transaction();
    table.insert(txn1, vec![ScalarValue::Int32(1)]).unwrap();
    table.commit(txn1).unwrap();

    // txn2 starts before txn3's insert
    let txn2 = table.begin_transaction();

    // txn3 inserts another row
    let txn3 = table.begin_transaction();
    table.insert(txn3, vec![ScalarValue::Int32(2)]).unwrap();
    table.commit(txn3).unwrap();

    // txn2 should only see txn1's row (snapshot isolation)
    let rows = table.scan(txn2);
    assert_eq!(rows.len(), 1, "snapshot isolation means txn2 cannot see txn3's insert that committed after txn2 began");
    assert_eq!(rows[0][0], ScalarValue::Int32(1));
}

#[test]
fn test_abort_not_visible() {
    let mut table = MvccTable::new();

    let txn1 = table.begin_transaction();
    table.insert(txn1, vec![ScalarValue::Int32(1)]).unwrap();
    table.abort(txn1).unwrap();

    let txn2 = table.begin_transaction();
    let rows = table.scan(txn2);
    assert_eq!(rows.len(), 0, "aborted transactions must leave no visible side effects -- this is the atomicity guarantee");
}

#[test]
fn test_read_own_writes() {
    let mut table = MvccTable::new();

    let txn = table.begin_transaction();
    table.insert(txn, vec![ScalarValue::Int32(42)]).unwrap();

    // Should see own uncommitted write
    let rows = table.scan(txn);
    assert_eq!(rows.len(), 1, "a transaction must always see its own uncommitted writes (read-your-own-writes guarantee)");
    assert_eq!(rows[0][0], ScalarValue::Int32(42));
}

#[test]
fn test_delete() {
    let mut table = MvccTable::new();

    let txn1 = table.begin_transaction();
    let row_id = table.insert(txn1, vec![ScalarValue::Int32(1)]).unwrap();
    table.commit(txn1).unwrap();

    let txn2 = table.begin_transaction();
    table.delete(txn2, row_id).unwrap();
    table.commit(txn2).unwrap();

    let txn3 = table.begin_transaction();
    let rows = table.scan(txn3);
    assert_eq!(rows.len(), 0, "a committed delete should make the row invisible to all future transactions");
}

#[test]
fn test_delete_not_visible_before_commit() {
    let mut table = MvccTable::new();

    let txn1 = table.begin_transaction();
    let row_id = table.insert(txn1, vec![ScalarValue::Int32(1)]).unwrap();
    table.commit(txn1).unwrap();

    let txn2 = table.begin_transaction();
    let txn3 = table.begin_transaction();

    table.delete(txn2, row_id).unwrap();
    // txn3 started before txn2 committed delete
    let rows = table.scan(txn3);
    assert_eq!(rows.len(), 1, "uncommitted deletes must not be visible to concurrent transactions");

    table.commit(txn2).unwrap();
    // txn3's snapshot shouldn't change
    let rows = table.scan(txn3);
    assert_eq!(rows.len(), 1, "txn3's snapshot was taken before the delete committed, so it must still see the row");
}

#[test]
fn test_concurrent_insert() {
    let mut table = MvccTable::new();

    let txn1 = table.begin_transaction();
    let txn2 = table.begin_transaction();

    table.insert(txn1, vec![ScalarValue::Int32(1)]).unwrap();
    table.insert(txn2, vec![ScalarValue::Int32(2)]).unwrap();

    table.commit(txn1).unwrap();
    table.commit(txn2).unwrap();

    let txn3 = table.begin_transaction();
    let rows = table.scan(txn3);
    assert_eq!(rows.len(), 2, "concurrent inserts to different rows should both be visible after both transactions commit");
}

#[test]
fn test_garbage_collection() {
    let mut table = MvccTable::new();

    // Create and delete some rows
    for i in 0..10 {
        let txn = table.begin_transaction();
        let id = table.insert(txn, vec![ScalarValue::Int32(i)]).unwrap();
        table.commit(txn).unwrap();

        if i < 5 {
            let txn = table.begin_transaction();
            table.delete(txn, id).unwrap();
            table.commit(txn).unwrap();
        }
    }

    let cleaned = table.garbage_collect();
    assert!(cleaned > 0, "Should have cleaned up some old versions");
}

#[test]
fn test_transaction_manager() {
    let mut tm = TransactionManager::new();
    let t1 = tm.begin();
    let t2 = tm.begin();

    assert_eq!(tm.status(t1), Some(TxnStatus::Active));
    assert_eq!(tm.status(t2), Some(TxnStatus::Active));

    tm.commit(t1).unwrap();
    assert_eq!(tm.status(t1), Some(TxnStatus::Committed));

    tm.abort(t2).unwrap();
    assert_eq!(tm.status(t2), Some(TxnStatus::Aborted));
}

#[test]
fn test_versioned_row_visibility() {
    let row = VersionedRow {
        data: vec![ScalarValue::Int32(1)],
        created_by: 1,
        deleted_by: None,
        prev_version: None,
    };

    // Visible to txn 2 if txn 1 is not in active set
    assert!(row.is_visible(2, &[]));

    // Not visible if created_by is still active
    assert!(!row.is_visible(2, &[1]), "rows created by an active (uncommitted) transaction must be invisible to other transactions");
}
