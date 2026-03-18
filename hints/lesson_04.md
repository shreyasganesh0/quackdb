# Lesson 04: Data Chunks

## What You're Building
A DataChunk bundles multiple Vectors (columns) of the same row count into one unit.
This is the batch-processing primitive that flows through the query engine -- operators
produce and consume chunks rather than individual rows. ChunkCollection accumulates
multiple chunks for materializing intermediate results like hash table build sides.

## Rust Concepts You'll Need
- [Ownership and Borrowing](../concepts/ownership_and_borrowing.md) -- DataChunk owns its Vec<Vector>; methods return references to columns
- [Traits and Derive](../concepts/traits_and_derive.md) -- implementing Display for tabular output

## Key Patterns

### Struct Wrapping a Vec of Structs
DataChunk is essentially a `Vec<Vector>` with a shared `count` and convenience methods.
The pattern is to delegate per-column work to the Vector API you already built.

```rust
// Analogy: a spreadsheet page (NOT the QuackDB solution)
struct SpreadsheetRow { cells: Vec<String> }
struct SpreadsheetPage {
    rows: Vec<SpreadsheetRow>,
    column_names: Vec<String>,
}

impl SpreadsheetPage {
    fn add_row(&mut self, values: Vec<String>) {
        assert_eq!(values.len(), self.column_names.len());
        self.rows.push(SpreadsheetRow { cells: values });
    }

    fn get_cell(&self, row: usize, col: usize) -> &str {
        &self.rows[row].cells[col]
    }
}
```

### Delegating to Inner Types
`append_row` iterates over the provided values and calls `set_value` on each column.
`flatten` iterates over columns and calls `flatten` on each Vector. Keep the chunk's
`count` in sync after each mutation.

```rust
// Analogy: a multi-track audio mixer (NOT the QuackDB solution)
struct Track { samples: Vec<f32> }
struct Mixer { tracks: Vec<Track>, length: usize }

impl Mixer {
    fn normalize_all(&mut self) {
        for track in &mut self.tracks {
            let max = track.samples.iter().cloned().fold(0.0f32, f32::max);
            if max > 0.0 {
                for s in &mut track.samples { *s /= max; }
            }
        }
    }
}
```

### Display for Tabular Output
Implement `fmt::Display` to print the chunk as a simple table. Iterate over rows
(0..count), and for each row iterate over columns calling `get_value`.

```rust
// Analogy: printing a CSV-like table
use std::fmt;

struct Table { headers: Vec<String>, rows: Vec<Vec<String>> }

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.headers.join("\t"))?;
        for row in &self.rows {
            writeln!(f, "{}", row.join("\t"))?;
        }
        Ok(())
    }
}
```

## Step-by-Step Implementation Order
1. Start with `DataChunk::new()` -- create a Vector for each LogicalType in the slice, set count to 0
2. Implement `with_capacity()` -- same but pass capacity to each Vector constructor
3. Implement `append_row()` -- for each column, call `set_value(self.count, value)`, then increment count; also update each vector's count
4. Implement `slice()` -- create a new DataChunk, copy values from offset..offset+length from each column
5. Implement `flatten()` -- call `flatten()` on each column vector
6. Implement `reset()` -- set count to 0; optionally reset column vectors
7. Implement `Display` -- iterate rows then columns, write each cell separated by tabs or pipes
8. Implement `ChunkCollection::new()`, `append()` -- track total_count by adding each chunk's count
9. Watch out for: `append_row` must keep the chunk's `count` and each Vector's count in sync

## Reading the Tests
- **`test_chunk_append_row`** appends 3 rows with Int32 and Int64 values, then reads each cell back via `chunk.column(0).get_value(i)`. This confirms that append_row must write to column index `i` at row position equal to the current count, then increment.
- **`test_chunk_slice`** appends 10 rows, slices at offset=2 length=5, and verifies the sliced chunk starts at value 2 and has 5 rows. Your slice must produce a new independent chunk.
- **`test_chunk_collection`** appends two chunks (2 rows + 1 row) and checks total_count is 3. Simple accumulation.

## What Comes Next
You now have the complete in-memory data model: Arena (memory), Types (schema),
Vectors (columns), and Chunks (row groups). Part II tackles **compression** —
how to shrink these columnar vectors for efficient storage. Lesson 05 starts with
Run-Length Encoding, which directly compresses the `Vector` data buffers you just
built. The `DataChunk` you created here becomes the primary unit that flows through
the entire query engine starting in Part IV.
