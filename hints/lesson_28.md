# Lesson 28: WAL

## What You're Building
A write-ahead log (WAL) that records every database mutation before it happens, enabling crash recovery. `WalWriter` and `WalReader` are generic over `W: Write` and `R: Read`, so they work with files, in-memory buffers, or network streams. Each `WalEntry` has a monotonically increasing LSN (log sequence number). The `RecoveryManager` replays the log using the classic ARIES-style analysis/redo/undo phases to determine which operations to reapply and which to roll back.

## Rust Concepts You'll Need
- [Generics](../concepts/generics.md) -- `WalWriter<W: Write>` and `WalReader<R: Read>` are parameterized over I/O traits, making them testable with `Cursor<Vec<u8>>`
- [IO and Serialization](../concepts/io_and_serialization.md) -- converting `WalEntry` to/from bytes for durable storage, using `Write::write_all` and `Read::read_exact`

## Key Patterns

### Generic Writer/Reader Over I/O Traits
By parameterizing over `std::io::Write` and `std::io::Read`, the same code works for files on disk and in-memory buffers in tests.

```rust
// Analogy: a message logger generic over output destination (NOT the QuackDB solution)
use std::io::Write;

struct Logger<W: Write> {
    output: W,
    seq: u64,
}

impl<W: Write> Logger<W> {
    fn new(output: W) -> Self { Self { output, seq: 0 } }

    fn log(&mut self, msg: &str) -> std::io::Result<u64> {
        let seq = self.seq;
        self.seq += 1;
        let header = seq.to_le_bytes();
        self.output.write_all(&header)?;
        let len = (msg.len() as u32).to_le_bytes();
        self.output.write_all(&len)?;
        self.output.write_all(msg.as_bytes())?;
        Ok(seq)
    }
}
```

### WAL Record Serialization
Each record must be serialized to bytes in a format that can be unambiguously deserialized. A common approach: write a tag byte for the variant, then the fields. Use length-prefixed strings and fixed-width integers.

```rust
// Analogy: serializing chat messages (NOT the QuackDB solution)
enum ChatMsg {
    Text { user: String, body: String },
    Join { user: String },
}

fn serialize(msg: &ChatMsg) -> Vec<u8> {
    let mut buf = Vec::new();
    match msg {
        ChatMsg::Text { user, body } => {
            buf.push(0x01); // tag
            buf.extend(&(user.len() as u32).to_le_bytes());
            buf.extend(user.as_bytes());
            buf.extend(&(body.len() as u32).to_le_bytes());
            buf.extend(body.as_bytes());
        }
        ChatMsg::Join { user } => {
            buf.push(0x02); // tag
            buf.extend(&(user.len() as u32).to_le_bytes());
            buf.extend(user.as_bytes());
        }
    }
    buf
}
```

### Crash Recovery Algorithm
Recovery scans the log and classifies transactions:
- **Analysis**: read all records, track which transactions began and which committed or aborted.
- **Redo**: for committed transactions, collect their Insert/Delete operations as `redo_ops`.
- **Undo**: for transactions that began but never committed (crash victims), collect their operations as `undo_ops` to be rolled back.

```rust
// Analogy: recovering a shared shopping list after a phone crash
// 1. Analysis: who started editing? who saved?
// 2. Redo: replay saved editors' additions
// 3. Undo: discard unsaved editors' additions
fn recover_list(log: &[(u64, &str)]) -> (Vec<u64>, Vec<u64>) {
    let mut began = std::collections::HashSet::new();
    let mut committed = std::collections::HashSet::new();
    for (id, action) in log {
        match *action {
            "begin" => { began.insert(*id); }
            "commit" => { committed.insert(*id); }
            _ => {}
        }
    }
    let aborted: Vec<u64> = began.difference(&committed).copied().collect();
    (committed.into_iter().collect(), aborted)
}
```

## Step-by-Step Implementation Order
1. Start with `WalEntry::to_bytes()` -- serialize the LSN as 8 bytes (little-endian), then a tag byte for the `WalRecord` variant, then variant-specific fields. For strings, write a 4-byte length prefix followed by the UTF-8 bytes. For `Vec<ScalarValue>`, write the count then each value with its own tag and data.
2. Implement `WalEntry::from_bytes()` -- the inverse of `to_bytes()`. Read the LSN, the tag byte, then decode fields based on the variant. Return `Err` on malformed input.
3. Implement `WalWriter::new()` -- store the writer and set `next_lsn` to 0 (or 1).
4. Implement `WalWriter::append()` -- create a `WalEntry` with the current LSN, serialize it, write the bytes, increment the LSN, and return the assigned LSN.
5. Implement `WalWriter::flush()` -- call `self.writer.flush()` and map the IO error to a String.
6. Implement `WalReader::next()` -- try to read the next entry's bytes. If the read hits EOF, return `Ok(None)`. Otherwise, deserialize and return `Ok(Some(entry))`. A practical approach: write a length prefix before each entry in `append()` so the reader knows how many bytes to read.
7. Implement `WalReader::read_all()` -- loop calling `next()` until `None`, collecting entries.
8. Implement `RecoveryManager::recover()` -- read all entries, track which txn IDs have `Begin` and `Commit`/`Abort` records. Transactions with `Begin` but no `Commit` or `Abort` go into `aborted`. Collect redo ops for committed transactions and undo ops for aborted ones.
9. Watch out for the serialization format consistency -- if `to_bytes` and `from_bytes` disagree on field order or sizes, the roundtrip test will fail immediately.

## Reading the Tests
- **`test_wal_record_roundtrip`** serializes an `Insert` record with mixed scalar values, deserializes it, and checks equality. This is your first validation target -- get `to_bytes`/`from_bytes` working correctly.
- **`test_wal_write_read`** writes three records (Begin, Insert, Commit) via `WalWriter`, then reads them back with `WalReader` and checks the count and record types. This confirms that your framing (how entries are delimited in the byte stream) is correct.
- **`test_recovery_uncommitted`** writes a Begin and Insert but no Commit, simulating a crash. It expects `recover()` to place txn 1 in `aborted` (not `committed`) and populate `undo_ops`. This validates the core recovery logic.

## What Comes Next
With transactions and durability in place, Part VIII explores **parallelism and
distribution** — scaling the database across CPU cores and machines. Lesson 29
introduces morsel-driven parallelism using `Arc<Mutex<>>` and thread spawning. The
`Pipeline` from L14 and `PhysicalOperator` trait become the foundation for parallel
execution. Lessons 31-33 extend to distributed query processing with data partitioning,
exchange operators, and shuffling between simulated nodes.
