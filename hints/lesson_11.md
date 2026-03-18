# Lesson 11: Columnar Writer

## What You're Building
A streaming columnar file writer that produces a Parquet-like file format: magic bytes, followed by row groups containing column chunks, and a footer at the end with schema and statistics metadata. The writer is generic over `W: Write`, so it can write to files, in-memory buffers, or network streams. This is the write half of the storage format that the reader (Lesson 12) will consume.

**Core concept count: 2** — the streaming file format (header/data/footer layout) and column statistics tracking. Everything else (generic writer, row group state machine, serialization) is scaffolding.

## Where to Start
Start with `ColumnStats` (simple min/max/null tracking — 3 small functions), then the `FileFooter` serialization (`to_bytes`/`from_bytes`), then the writer pipeline (`new` → `begin_row_group` → `write_column` → `end_row_group` → `finish`). The stats and footer are self-contained; the writer just orchestrates them.

## Concept Recap
Building on Lesson 10: You built the buffer pool for caching pages in memory. Now you will create the file format that organizes column data on disk. The writer uses `DataChunk` (from Lesson 04) as its input and produces a self-contained file with embedded schema and statistics metadata. The `CompressionFrame` format from Lesson 08 can be used to compress individual column chunks.

## Rust Concepts You'll Need
- [Generics](../concepts/generics.md) -- `ColumnarFileWriter<W: Write>` works with any writable destination
- [IO and Serialization](../concepts/io_and_serialization.md) -- writing structured data as bytes, serializing the footer
- [Error Handling](../concepts/error_handling.md) -- propagating `io::Error` from write calls as `String`

## Key Patterns

### Generic Writer over W: Write
By parameterizing over `W: Write`, the same code works for files and in-memory buffers.
Think of a printer that does not care whether it is printing to paper or to a PDF --
the output interface is the same.

```rust
use std::io::Write;

struct LogWriter<W: Write> {
    writer: W,
    bytes_written: u64,
}

impl<W: Write> LogWriter<W> {
    fn new(mut writer: W) -> Result<Self, String> {
        writer.write_all(b"LOG1").map_err(|e| e.to_string())?;
        Ok(Self { writer, bytes_written: 4 })
    }

    fn write_entry(&mut self, data: &[u8]) -> Result<(), String> {
        let len = (data.len() as u32).to_le_bytes();
        self.writer.write_all(&len).map_err(|e| e.to_string())?;
        self.writer.write_all(data).map_err(|e| e.to_string())?;
        self.bytes_written += 4 + data.len() as u64;
        Ok(())
    }

    fn finish(mut self) -> Result<W, String> {
        self.writer.write_all(&self.bytes_written.to_le_bytes())
            .map_err(|e| e.to_string())?;
        Ok(self.writer)
    }
}
```

Your `ColumnarFileWriter::new()` should write the `MAGIC` bytes immediately and track how many bytes have been written so you can record column chunk offsets.

### Streaming File Format: Header / Data / Footer
Many columnar formats follow this layout. Think of a book: title page (magic), chapters
(row groups), and index at the back (footer). The index goes last because you do not
know all the page numbers until you have finished writing.

```
[MAGIC bytes]
[Row Group 1: Column 0 data | Column 1 data | ...]
[Row Group 2: Column 0 data | Column 1 data | ...]
...
[Footer: schema + row group metadata]
[Footer length as u64]
[MAGIC bytes (repeated)]
```

The footer goes last because you do not know all the metadata (offsets, sizes, statistics) until you have finished writing all the data. The reader reads the footer length from the end of the file to locate the footer.

### Tracking State with an In-Progress Struct
Use a separate struct to track partially-built row groups. Think of a shopping cart --
you add items (columns) one at a time, and "checkout" (end_row_group) finalizes the order.

```rust
struct ChapterInProgress {
    title: String,
    paragraphs: Vec<String>,
}

struct BookWriter<W: Write> {
    writer: W,
    chapters: Vec<ChapterMeta>,
    current: Option<ChapterInProgress>,
}
```

Guard against misuse: `write_column` should fail if no row group is in progress; `begin_row_group` should fail if a previous one was not ended.

### Statistics Tracking
Column statistics (min, max, null count) enable predicate pushdown in the reader.
Think of a library catalog card: it tells you the date range of a journal volume
so you can skip volumes that cannot contain the article you want.

```rust
struct RangeStats {
    min: Option<Vec<u8>>,
    max: Option<Vec<u8>>,
    null_count: u64,
}

impl RangeStats {
    fn update(&mut self, value: &[u8], is_null: bool) {
        if is_null { self.null_count += 1; return; }
        if self.min.is_none() || value < self.min.as_ref().unwrap() {
            self.min = Some(value.to_vec());
        }
        if self.max.is_none() || value > self.max.as_ref().unwrap() {
            self.max = Some(value.to_vec());
        }
    }
}
```

## Step-by-Step Implementation Order
1. Start with `ColumnStats::new()` -- initialize with zero null count and `None` for min/max/distinct
2. Implement `ColumnStats::update()` -- compare the incoming value bytes against current min/max, increment null count when `is_null` is true
3. Implement `ColumnStats::merge()` -- combine two stats by summing null counts and taking the wider min/max
4. Implement `ColumnarFileWriter::new()` -- write `MAGIC` bytes to the writer, initialize `bytes_written` to 4, store the schema, initialize empty `row_groups` and set `current_row_group` to `None`
5. Implement `begin_row_group()` -- check that no row group is currently in progress, then set `current_row_group` to a new in-progress struct
6. Implement `write_column()` -- record the current `bytes_written` as the offset, write the raw column data, create a `ColumnChunkMeta` with the offset, size, stats, and push it to the in-progress row group
7. Implement `end_row_group()` -- take the in-progress row group, set `num_rows`, convert to `RowGroupMeta`, push to `self.row_groups`, reset current to None
8. Implement `write_chunk()` -- convenience that calls begin_row_group, serializes each column from the DataChunk, calls write_column for each, then end_row_group
9. Implement `FileFooter::to_bytes()` and `from_bytes()` -- serialize the schema and all row group metadata into a byte vector and back

## Common Mistakes
- **Losing track of `bytes_written`**: Every `write_all` call must also add to the byte counter. If you forget even one write, all subsequent column chunk offsets will be wrong, and the reader will seek to the wrong positions.
- **Not writing the footer length before the trailing magic**: The reader needs to know how long the footer is. The standard pattern is: write footer bytes, then write footer_length as u64, then write MAGIC. If you skip the length, the reader cannot locate the footer.
- **Allowing overlapping row groups**: If `begin_row_group` is called while another is in progress, the metadata will be corrupted. Always check and reject this case.

## Reading the Tests
- **`test_columnar_write_single_column`** writes a single Int32 column with 3 values, finishes the file, and checks the output starts with MAGIC bytes `b"QUAK"`. This confirms your constructor writes the magic immediately.
- **`test_columnar_write_multi_column`** writes 2 columns (Int32 and Float64) in one row group and checks the output is non-empty. This tests multi-column coordination within a single row group.
- **`test_columnar_write_multiple_row_groups`** writes 3 separate row groups, each with 1 value. This confirms your begin/end row group cycle works repeatedly.
- **`test_columnar_write_with_stats`** creates stats with specific min/max values and passes them to `write_column`. This confirms your writer accepts and stores statistics metadata.
- **`test_columnar_write_chunk`** uses the convenience `write_chunk` method with a DataChunk containing 2 rows and 2 columns, then checks the file starts with MAGIC. This tests the high-level DataChunk integration path.
- **`test_column_stats_update`** calls `update` with three values and one null, then checks `null_count == 1`. This confirms your stats tracking counts nulls correctly.
- **`test_column_stats_merge`** merges two stats with different null counts and min/max values, checking that null counts sum and min/max widen. This is used when combining stats across row groups.
- **`test_footer_serialize_roundtrip`** creates a footer with a 2-column schema and one row group, serializes and deserializes, and checks all fields survive. This is the most important test -- the footer is how the reader discovers the file's contents.
- **`test_magic_bytes`** simply checks that `MAGIC == b"QUAK"`. Use this constant, not a hardcoded string.

## What Comes Next
Lesson 12 builds the **columnar reader** that opens files produced by this writer,
reads the footer to discover schema and metadata, and supports column projection
(reading only needed columns) and predicate pushdown (skipping row groups based on
min/max statistics). The writer and reader together form the complete storage format.
