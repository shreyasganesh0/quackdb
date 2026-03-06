# Lesson 11: Columnar Writer

## What You're Building
A streaming columnar file writer that produces a Parquet-like file format: magic bytes, followed by row groups containing column chunks, and a footer at the end with schema and statistics metadata. The writer is generic over `W: Write`, so it can write to files, in-memory buffers, or network streams. This is the write half of the storage format that the reader (Lesson 12) will consume.

## Rust Concepts You'll Need
- [Generics](../concepts/generics.md) -- `ColumnarFileWriter<W: Write>` works with any writable destination
- [IO and Serialization](../concepts/io_and_serialization.md) -- writing structured data as bytes, serializing the footer
- [Error Handling](../concepts/error_handling.md) -- propagating `io::Error` from write calls as `String`

## Key Patterns

### Generic Writer over W: Write
By parameterizing over `W: Write`, the same code works for files and in-memory buffers:

```rust
use std::io::Write;

struct LogWriter<W: Write> {
    writer: W,
    bytes_written: u64,
}

impl<W: Write> LogWriter<W> {
    fn new(mut writer: W) -> Result<Self, String> {
        // Write a file signature first
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
        // Write a footer with total count, then return the writer
        self.writer.write_all(&self.bytes_written.to_le_bytes())
            .map_err(|e| e.to_string())?;
        Ok(self.writer)
    }
}
```

Your `ColumnarFileWriter::new()` should write the `MAGIC` bytes immediately and track how many bytes have been written so you can record column chunk offsets.

### Streaming File Format: Header / Data / Footer
Many columnar formats follow this layout:

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
Use a separate struct to track partially-built row groups:

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

## Step-by-Step Implementation Order
1. Start with `ColumnStats::new()` -- initialize with zero null count and `None` for min/max/distinct.
2. Implement `ColumnStats::update()` -- compare the incoming value bytes against current min/max, increment null count when `is_null` is true.
3. Implement `ColumnarFileWriter::new()` -- write `MAGIC` bytes to the writer, initialize `bytes_written` to 4, store the schema, initialize empty `row_groups` and set `current_row_group` to `None`.
4. Implement `begin_row_group()` -- check that no row group is currently in progress, then set `current_row_group` to a new `RowGroupInProgress`.
5. Implement `write_column()` -- record the current `bytes_written` as the offset, write the raw column data, create a `ColumnChunkMeta` with the offset, size, stats, and push it to the in-progress row group.
6. Implement `end_row_group()` -- take the in-progress row group, set `num_rows`, convert it to a `RowGroupMeta`, push it to `self.row_groups`, and reset `current_row_group` to `None`.
7. Implement `write_chunk()` -- a convenience method that calls `begin_row_group`, serializes each column from the `DataChunk`, calls `write_column` for each, then `end_row_group`.
8. Implement `FileFooter::to_bytes()` -- serialize the schema (column names, types) and all row group metadata into a byte vector. The exact format is up to you, but keep it deterministic.
9. Implement `finish()` -- serialize the footer, write its bytes, write the footer length as a `u64`, write `MAGIC` again, and return the inner writer.
10. Watch out for keeping `bytes_written` in sync -- every `write_all` call must also add to this counter, or your column chunk offsets will be wrong.

## Reading the Tests
- Look for a test that creates a `ColumnarFileWriter` over a `Vec<u8>`, writes one or more row groups, calls `finish()`, and then checks that the output starts with `b"QUAK"` and ends with `b"QUAK"`. This confirms the magic-bytes bookending.
- Look for a test that writes a chunk via `write_chunk` and then reads it back (possibly in the Lesson 12 tests). The assertions on column values tell you that the serialization of each column must be byte-for-byte recoverable.
