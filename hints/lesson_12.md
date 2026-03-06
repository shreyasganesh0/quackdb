# Lesson 12: Columnar Reader

## What You're Building
A columnar file reader that opens files written by the Lesson 11 writer, reads the footer from the end of the file to discover schema and row group metadata, and supports column projection (reading only selected columns) and predicate pushdown (skipping row groups whose min/max stats prove the predicate cannot match). This is how analytical databases avoid reading unnecessary data.

## Rust Concepts You'll Need
- [Trait Bounds](../concepts/trait_bounds.md) -- `R: Read + Seek` lets you both stream bytes forward and jump to arbitrary positions
- [IO and Serialization](../concepts/io_and_serialization.md) -- seeking to the end of the file, reading footer length, then seeking back
- [Enums and Matching](../concepts/enums_and_matching.md) -- matching on `PredicateOp` variants to evaluate pruning conditions

## Key Patterns

### Reading a Footer from the End of a File
When the footer is at the end, you need `Seek` to find it:

```rust
use std::io::{Read, Seek, SeekFrom};

fn read_trailer<R: Read + Seek>(reader: &mut R) -> Result<Vec<u8>, String> {
    // Seek to 12 bytes before the end (8 for length + 4 for magic)
    reader.seek(SeekFrom::End(-12)).map_err(|e| e.to_string())?;

    let mut len_buf = [0u8; 8];
    reader.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
    let footer_len = u64::from_le_bytes(len_buf) as usize;

    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic).map_err(|e| e.to_string())?;
    if &magic != b"MYFT" {
        return Err("bad magic".into());
    }

    // Now seek back to where the footer starts
    let footer_start = -(12 + footer_len as i64);
    reader.seek(SeekFrom::End(footer_start)).map_err(|e| e.to_string())?;

    let mut footer_buf = vec![0u8; footer_len];
    reader.read_exact(&mut footer_buf).map_err(|e| e.to_string())?;
    Ok(footer_buf)
}
```

Your `open()` method follows this pattern: seek to the end to read the footer length and trailing magic bytes, then seek back to read the footer itself. Deserialize it with `FileFooter::from_bytes`.

### Predicate Pushdown via Min/Max Pruning
If a row group's column stats say `min=50, max=100`, and the predicate is `column < 30`, you can skip the entire row group:

```rust
enum Comparison { Lt, Gt, Eq }

fn can_skip(op: Comparison, target: i64, min: i64, max: i64) -> bool {
    match op {
        // Looking for values < target: skip if min >= target
        Comparison::Lt => min >= target,
        // Looking for values > target: skip if max <= target
        Comparison::Gt => max <= target,
        // Looking for values == target: skip if target < min or target > max
        Comparison::Eq => target < min || target > max,
    }
}
```

Your `ScanPredicate::can_prune` should convert the `ColumnStats` min/max bytes to comparable values and check whether the predicate can possibly match any row in the group. Return `true` when the row group can be safely skipped.

### Projection: Reading Only Selected Columns
When `projection` is `Some(&[0, 2])`, only read column chunks at indices 0 and 2 from each row group. This avoids I/O for columns the query does not need.

## Step-by-Step Implementation Order
1. Start with `ColumnarFileReader::open()` -- seek to the end to read the footer length and trailing magic, validate the magic, then seek back and read the footer bytes. Deserialize with `FileFooter::from_bytes`.
2. Implement `read_column()` -- look up the `ColumnChunkMeta` for the given row group and column, seek to its `offset`, and read `size` bytes.
3. Implement `ScanPredicate::can_prune()` -- extract min/max from the column stats, convert them to the appropriate type based on `logical_type`, then compare against `self.value` using `self.op`. Return `true` if the predicate guarantees no matches.
4. Implement `scan()` -- iterate over row groups. For each row group, check all predicates to see if the group can be pruned. If not pruned, determine which columns to read (use `projection` if provided, otherwise all columns). Read each needed column with `read_column`, assemble a `DataChunk`, and collect results.
5. Watch out for the seek positions: the `offset` stored in `ColumnChunkMeta` is an absolute byte position from the start of the file. Use `SeekFrom::Start(offset)` when reading columns.
6. Watch out for empty `min_value`/`max_value` in stats -- if stats are `None`, you cannot prune that row group and must read it.

## Reading the Tests
- Look for a round-trip test that writes data with `ColumnarFileWriter`, wraps the resulting bytes in a `Cursor`, opens it with `ColumnarFileReader::open()`, and asserts the schema and row counts match. This confirms your footer reading logic.
- Look for a test that uses `scan` with predicates and checks that fewer row groups are read than exist in the file. The assertions on the returned chunks tell you that pruned groups must be entirely absent from results.
