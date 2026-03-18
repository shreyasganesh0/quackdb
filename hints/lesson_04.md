# Lesson 04: Data Chunks

## What You're Building
A DataChunk bundles multiple Vectors (columns) of the same row count into one unit.
This is the batch-processing primitive that flows through the query engine -- operators
produce and consume chunks rather than individual rows. ChunkCollection accumulates
multiple chunks for materializing intermediate results like hash table build sides.

## Concept Recap
Building on Lesson 03: You built `Vector` for single columns and `ValidityMask` for null tracking. Now you will compose multiple Vectors into a `DataChunk`, using `Vector::set_value()` and `Vector::get_value()` to read and write individual cells, and `Vector::flatten()` to materialize constant columns.

## Rust Concepts You'll Need
- [Ownership and Borrowing](../concepts/ownership_and_borrowing.md) -- DataChunk owns its Vec<Vector>; methods return references to columns
- [Traits and Derive](../concepts/traits_and_derive.md) -- implementing Display for tabular output

## Key Patterns

### Struct Wrapping a Vec of Structs
DataChunk is essentially a `Vec<Vector>` with a shared `count` and convenience methods.
The pattern is to delegate per-column work to the Vector API you already built. Think
of a filing cabinet -- each drawer (Vector) holds one type of document, and the cabinet
(DataChunk) keeps them organized together.

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
`count` in sync after each mutation. This is the delegation pattern -- the chunk
coordinates, the vectors do the real work.

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
(0..count), and for each row iterate over columns calling `get_value`. Think of it
like a report printer -- walk each row, format each cell, join with separators.

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

### Slicing as Copy, Not View
`slice()` produces a new, independent DataChunk by copying values from a range.
Think of photocopying pages 5-10 of a report -- the copy is standalone and the
original is unaffected.

```rust
// Analogy: extracting a sublist
fn sublist<T: Clone>(items: &[T], offset: usize, length: usize) -> Vec<T> {
    items[offset..offset + length].to_vec()
}
```

## Step-by-Step Implementation Order
1. Start with `DataChunk::new()` -- create a Vector for each LogicalType in the slice, set count to 0
2. Implement `with_capacity()` -- same but pass capacity to each Vector constructor
3. Implement `column()` and `column_mut()` -- return references to individual vectors by index
4. Implement `types()` -- collect the logical type from each column vector
5. Implement `append_row()` -- for each column, call `set_value(self.count, value)`, then increment count; also update each vector's count
6. Implement `slice()` -- create a new DataChunk, copy values from offset..offset+length from each column
7. Implement `flatten()` -- call `flatten()` on each column vector
8. Implement `reset()` -- set count to 0; optionally reset column vectors
9. Implement `Display` -- iterate rows then columns, write each cell separated by tabs or pipes

## Common Mistakes
- **Count desynchronization**: The chunk has a `count` and each Vector has its own `count`. When you call `append_row`, you must increment both. If they get out of sync, `get_value` may read uninitialized memory or tests will see wrong row counts.
- **Forgetting to copy all columns in `slice()`**: The sliced chunk must have the same number of columns as the original. Iterate over every column, not just the first.
- **Not producing output in `Display`**: The test checks that `format!("{}", chunk)` is non-empty and contains at least some of the stored values. Even a minimal implementation that prints values separated by whitespace will pass.

## Reading the Tests
- **`test_chunk_creation`** creates a chunk with Int32 and Varchar columns and checks `column_count() == 2` and `count() == 0`. A new chunk starts empty.
- **`test_chunk_with_capacity`** creates a chunk with capacity 100 and verifies it still starts with count 0. Capacity is pre-allocation, not initial rows.
- **`test_chunk_append_row`** appends 3 rows with Int32 and Int64 values, then reads each cell back via `chunk.column(0).get_value(i)`. This confirms that append_row must write to the correct column at the current count position, then increment.
- **`test_chunk_multi_type`** appends rows with Int32, Float64, and Boolean columns. It verifies that different types can coexist in the same chunk and that each column independently stores and retrieves its values.
- **`test_chunk_slice`** appends 10 rows, slices at offset=2 length=5, and verifies the sliced chunk starts at value 2 and has 5 rows. Your slice must produce a new independent chunk where index 0 maps to the original's index 2.
- **`test_chunk_flatten`** calls `flatten()` on a chunk. This delegates to each vector's flatten method, converting constant vectors to flat representation.
- **`test_chunk_reset`** appends 2 rows, resets, verifies count is 0, then appends a new row and checks the value. Reset must allow the chunk to be reused, with new data overwriting old.
- **`test_chunk_types`** creates a 3-column chunk and checks that `types()` returns the schema in column order. This is used throughout the engine for schema propagation.
- **`test_chunk_display`** appends rows and formats the chunk as a string. The test just checks the output is non-empty and contains expected values. Keep your Display implementation simple.
- **`test_chunk_collection`** appends two chunks (2 rows + 1 row) and checks `chunk_count() == 2` and `total_count() == 3`. Collection is straightforward accumulation.
- **`test_chunk_collection_empty`** verifies an empty collection reports 0 chunks and 0 total rows.
- **`test_chunk_column_access`** appends a row, mutates column 0 via `column_mut(0)`, and checks that column 0 is updated while column 1 is unchanged. Mutable access to one column must not affect others.

## What Comes Next
You now have the complete in-memory data model: Arena (memory), Types (schema),
Vectors (columns), and Chunks (row groups). Part II tackles **compression** --
how to shrink these columnar vectors for efficient storage. Lesson 05 starts with
Run-Length Encoding, which directly compresses the `Vector` data buffers you just
built. The `DataChunk` you created here becomes the primary unit that flows through
the entire query engine starting in Part IV.
