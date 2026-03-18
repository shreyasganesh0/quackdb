//! Lesson 28: Write-Ahead Logging Tests

use quackdb::types::ScalarValue;
use quackdb::transaction::wal::*;
use std::io::Cursor;

#[test]
fn test_wal_record_roundtrip() {
    let record = WalRecord::Insert {
        txn_id: 1,
        table: "users".to_string(),
        row_id: 42,
        data: vec![ScalarValue::Int32(100), ScalarValue::Varchar("hello".into())],
    };

    let entry = WalEntry { lsn: 1, record: record.clone() };
    let bytes = entry.to_bytes();
    let restored = WalEntry::from_bytes(&bytes).unwrap();

    assert_eq!(restored.lsn, 1);
    assert_eq!(restored.record, record);
}

#[test]
fn test_wal_write_read() {
    let mut buf = Vec::new();
    let mut writer = WalWriter::new(Cursor::new(&mut buf));

    let lsn1 = writer.append(WalRecord::Begin { txn_id: 1 }).unwrap();
    let lsn2 = writer.append(WalRecord::Insert {
        txn_id: 1,
        table: "t".to_string(),
        row_id: 0,
        data: vec![ScalarValue::Int32(42)],
    }).unwrap();
    let lsn3 = writer.append(WalRecord::Commit { txn_id: 1 }).unwrap();
    writer.flush().unwrap();

    assert!(lsn1 < lsn2, "LSNs must be monotonically increasing to establish a total order of WAL entries");
    assert!(lsn2 < lsn3);

    let mut reader = WalReader::new(Cursor::new(&buf));
    let entries = reader.read_all().unwrap();
    assert_eq!(entries.len(), 3, "WAL should contain exactly Begin, Insert, Commit for one complete transaction");
    assert!(matches!(entries[0].record, WalRecord::Begin { txn_id: 1 }));
    assert!(matches!(entries[2].record, WalRecord::Commit { txn_id: 1 }));
}

#[test]
fn test_recovery_committed() {
    let mut buf = Vec::new();
    let mut writer = WalWriter::new(Cursor::new(&mut buf));
    writer.append(WalRecord::Begin { txn_id: 1 }).unwrap();
    writer.append(WalRecord::Insert {
        txn_id: 1,
        table: "t".to_string(),
        row_id: 0,
        data: vec![ScalarValue::Int32(1)],
    }).unwrap();
    writer.append(WalRecord::Commit { txn_id: 1 }).unwrap();
    writer.flush().unwrap();

    let result = RecoveryManager::recover(Cursor::new(&buf)).unwrap();
    assert!(result.committed.contains(&1), "recovery should identify txn 1 as committed since its Commit record is in the WAL");
    assert!(!result.redo_ops.is_empty(), "committed transactions need redo operations to restore their effects after a crash");
}

#[test]
fn test_recovery_uncommitted() {
    let mut buf = Vec::new();
    let mut writer = WalWriter::new(Cursor::new(&mut buf));
    writer.append(WalRecord::Begin { txn_id: 1 }).unwrap();
    writer.append(WalRecord::Insert {
        txn_id: 1,
        table: "t".to_string(),
        row_id: 0,
        data: vec![ScalarValue::Int32(1)],
    }).unwrap();
    // No commit! Simulating a crash
    writer.flush().unwrap();

    let result = RecoveryManager::recover(Cursor::new(&buf)).unwrap();
    assert!(!result.committed.contains(&1), "transaction without a Commit record must not be treated as committed");
    assert!(result.aborted.contains(&1), "a transaction with Begin but no Commit in the WAL was in-flight during crash and must be aborted");
    assert!(!result.undo_ops.is_empty(), "aborted transactions need undo operations to reverse their partial effects");
}

#[test]
fn test_recovery_mixed() {
    let mut buf = Vec::new();
    let mut writer = WalWriter::new(Cursor::new(&mut buf));

    // Txn 1: committed
    writer.append(WalRecord::Begin { txn_id: 1 }).unwrap();
    writer.append(WalRecord::Insert {
        txn_id: 1, table: "t".to_string(), row_id: 0,
        data: vec![ScalarValue::Int32(1)],
    }).unwrap();
    writer.append(WalRecord::Commit { txn_id: 1 }).unwrap();

    // Txn 2: uncommitted (crash)
    writer.append(WalRecord::Begin { txn_id: 2 }).unwrap();
    writer.append(WalRecord::Insert {
        txn_id: 2, table: "t".to_string(), row_id: 1,
        data: vec![ScalarValue::Int32(2)],
    }).unwrap();
    writer.flush().unwrap();

    let result = RecoveryManager::recover(Cursor::new(&buf)).unwrap();
    assert!(result.committed.contains(&1), "txn 1 had a Commit record so recovery must redo it");
    assert!(result.aborted.contains(&2), "txn 2 had no Commit record so recovery must undo it");
}

#[test]
fn test_wal_checkpoint() {
    let mut buf = Vec::new();
    let mut writer = WalWriter::new(Cursor::new(&mut buf));

    writer.append(WalRecord::Begin { txn_id: 1 }).unwrap();
    writer.append(WalRecord::Commit { txn_id: 1 }).unwrap();
    writer.append(WalRecord::Checkpoint { active_txns: vec![] }).unwrap();
    writer.flush().unwrap();

    let mut reader = WalReader::new(Cursor::new(&buf));
    let entries = reader.read_all().unwrap();
    assert_eq!(entries.len(), 3);
    assert!(matches!(entries[2].record, WalRecord::Checkpoint { .. }));
}

#[test]
fn test_wal_abort_record() {
    let mut buf = Vec::new();
    let mut writer = WalWriter::new(Cursor::new(&mut buf));
    writer.append(WalRecord::Begin { txn_id: 1 }).unwrap();
    writer.append(WalRecord::Insert {
        txn_id: 1, table: "t".to_string(), row_id: 0,
        data: vec![ScalarValue::Int32(99)],
    }).unwrap();
    writer.append(WalRecord::Abort { txn_id: 1 }).unwrap();
    writer.flush().unwrap();

    let result = RecoveryManager::recover(Cursor::new(&buf)).unwrap();
    assert!(result.aborted.contains(&1));
    assert!(!result.committed.contains(&1));
}

#[test]
fn test_wal_delete_record() {
    let mut buf = Vec::new();
    let mut writer = WalWriter::new(Cursor::new(&mut buf));
    writer.append(WalRecord::Begin { txn_id: 1 }).unwrap();
    writer.append(WalRecord::Delete { txn_id: 1, table: "t".to_string(), row_id: 5 }).unwrap();
    writer.append(WalRecord::Commit { txn_id: 1 }).unwrap();
    writer.flush().unwrap();

    let mut reader = WalReader::new(Cursor::new(&buf));
    let entries = reader.read_all().unwrap();
    assert!(matches!(&entries[1].record, WalRecord::Delete { row_id: 5, .. }));
}

#[test]
fn test_recovery_idempotent() {
    let mut buf = Vec::new();
    let mut writer = WalWriter::new(Cursor::new(&mut buf));
    writer.append(WalRecord::Begin { txn_id: 1 }).unwrap();
    writer.append(WalRecord::Insert {
        txn_id: 1, table: "t".to_string(), row_id: 0,
        data: vec![ScalarValue::Int32(1)],
    }).unwrap();
    writer.append(WalRecord::Commit { txn_id: 1 }).unwrap();
    writer.flush().unwrap();

    let result1 = RecoveryManager::recover(Cursor::new(&buf)).unwrap();
    let result2 = RecoveryManager::recover(Cursor::new(&buf)).unwrap();

    assert_eq!(result1.committed, result2.committed, "recovery must be idempotent -- running it twice should produce the same committed set");
    assert_eq!(result1.aborted, result2.aborted, "recovery must be idempotent -- running it twice should produce the same aborted set");
}
