# Lesson 12: Columnar Reader

## What You're Building
A columnar file reader that opens files written by the Lesson 11 writer, reads the footer from the end of the file to discover schema and row group metadata, and supports column projection (reading only selected columns) and predicate pushdown (skipping row groups whose min/max stats prove the predicate cannot match). This is how analytical databases avoid reading unnecessary data.

## Concept Recap
Building on Lesson 11: You built the `ColumnarFileWriter` that produces files with MAGIC bytes, row groups, and a footer. Now you will build the reader that consumes those files. The reader uses `FileFooter::from_bytes()` to parse the metadata you serialized, and `ColumnChunkMeta` offsets to seek directly to column data. The `ColumnStats` you stored enable predicate pushdown.

## Rust Concepts You'll Need
- [Trait Bounds](../concepts/trait_bounds.md) -- `R: Read + Seek` lets you both stream bytes forward and jump to arbitrary positions
- [IO and Serialization](../concepts/io_and_serialization.md) -- seeking to the end of the file, reading footer length, then seeking back
- [Enums and Matching](../concepts/enums_and_matching.md) -- matching on `PredicateOp` variants to evaluate pruning conditions

## Key Patterns

### Reading a Footer from the End of a File
When the footer is at the end, you need `Seek` to find it. Think of reading the index
at the back of a textbook -- you flip to the last pages, find what you need, then jump
to the right chapter.

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
If a row group's column stats say `min=50, max=100`, and the predicate is `column < 30`, you can skip the entire row group. Think of a library catalog: if a shelf is labeled "Books published 2000-2010" and you want something from 2020, you skip the entire shelf.

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
When `projection` is `Some(&[0, 2])`, only read column chunks at indices 0 and 2 from each row group. This avoids I/O for columns the query does not need. Think of a
newspaper reader who only reads the sports section -- no need to load the entire paper.

```rust
// Analogy: selective column loading
fn load_columns(all_columns: &[Vec<u8>], projection: Option<&[usize]>) -> Vec<Vec<u8>> {
    match projection {
        Some(indices) => indices.iter().map(|&i| all_columns[i].clone()).collect(),
        None => all_columns.to_vec(),
    }
}
```

### Seek-Based Random Access
Using `SeekFrom::Start(offset)` jumps directly to a column chunk's data without reading
intervening bytes. This is what makes columnar formats fast for analytical queries --
you skip irrelevant data entirely.

```rust
// Analogy: jumping to a bookmark in a file
fn read_at_offset<R: Read + Seek>(reader: &mut R, offset: u64, size: usize) -> Vec<u8> {
    reader.seek(SeekFrom::Start(offset)).unwrap();
    let mut buf = vec![0u8; size];
    reader.read_exact(&mut buf).unwrap();
    buf
}
```

## Step-by-Step Implementation Order
1. Start with `ColumnarFileReader::open()` -- seek to the end to read the footer length and trailing magic, validate the magic, then seek back and read the footer bytes. Deserialize with `FileFooter::from_bytes`. Store the footer's schema, row groups, and total rows.
2. Implement accessor methods: `total_rows()`, `schema()`, `row_group_count()` -- simple getters from the parsed footer.
3. Implement `read_column()` -- look up the `ColumnChunkMeta` for the given row group and column, seek to its `offset`, and read `size` bytes.
4. Implement `ScanPredicate::can_prune()` -- extract min/max from the column stats, convert them to the appropriate type based on `logical_type`, then compare against `self.value` using `self.op`. Return `true` if the predicate guarantees no matches.
5. Implement `scan()` -- iterate over row groups. For each row group, check all predicates to see if the group can be pruned. If not pruned, determine which columns to read (use `projection` if provided, otherwise all columns). Read each needed column, deserialize into a DataChunk, and collect results.
6. Handle empty files: if there are no row groups, return an empty result.
7. Handle missing stats: if `min_value` or `max_value` is `None`, you cannot prune that row group -- read it.

## Common Mistakes
- **Wrong seek math for the footer**: The file ends with `[footer bytes][footer_length as u64][MAGIC]`. To read the footer length, seek to `End(-12)` (8 bytes for length + 4 for magic). Then to read the footer itself, seek to `End(-(12 + footer_len))`. Off-by-one here means reading garbage.
- **Pruning when stats are missing**: If a column's `min_value` or `max_value` is `None`, the predicate cannot be evaluated. You must NOT prune -- assume the row group might contain matching rows.
- **Using relative seeks instead of absolute**: Column chunk offsets in `ColumnChunkMeta` are absolute byte positions from the start of the file. Use `SeekFrom::Start(offset)`, not `SeekFrom::Current`.

## Reading the Tests
- **`test_reader_open`** writes 3 rows via the helper, opens the file, and checks `total_rows() == 3`, `schema().len() == 2`, and `row_group_count() == 1`. This confirms your footer parsing correctly extracts the metadata.
- **`test_reader_scan_all`** writes 3 rows, scans with no projection and no predicates, and checks the total row count across returned chunks is 3. This is the basic full-table scan path.
- **`test_reader_projection`** writes 2 rows with 2 columns, scans with `projection = Some(&[0])`, and checks the returned chunk has `column_count() == 1`. Only the requested column should be read.
- **`test_reader_row_group_pruning`** writes 2 row groups (IDs 1-100 and 101-200) and scans with predicate `id > 150`. The first row group (max=100) should be pruned, so the result should have at most 100 rows. This is the core predicate pushdown test.
- **`test_reader_write_read_roundtrip`** writes 3 rows `[(10, 1000), (20, 2000), (30, 3000)]`, scans them back, and checks every individual value matches. This is the full end-to-end correctness test for the writer+reader pipeline.
- **`test_predicate_ops`** tests `can_prune` directly with a stats object (min=10, max=100). GT 200 must prune (max is only 100). GT 50 must NOT prune (values up to 100 exist). LT 5 must prune (min is 10). This gives you the exact pruning logic for each operator.
- **`test_reader_empty_file`** writes a file with no row groups and checks `total_rows() == 0` and scan returns empty results. Edge case for the empty-file path.

## What Comes Next
You now have a complete storage engine: pages, buffer pool, and columnar file I/O.
Part IV shifts to **vectorized execution** -- processing data in batches. Lesson 13
introduces expression evaluation over the `DataChunk` type you've been building.
The `Vector` and `DataChunk` types from Part I become the data currency that flows
through every operator. The predicate pushdown pattern from this lesson reappears
in the query optimizer (L25).
