# Lesson 09: Pages & Layout

## What You're Building
A fixed-size page abstraction that serves as the fundamental unit of storage in the database. Each page carries a header with metadata (type, ID, checksum, record count), followed by a data region. You will also build a `PageBuilder` that lets you incrementally fill a page and finalize it with a CRC32 checksum. Pages are what the buffer pool (next lesson) reads and writes to disk.

## Concept Recap
Building on Lesson 08: You built a compression framework that produces `CompressionFrame` byte vectors. Now you need somewhere to *store* those bytes persistently. Pages are the fixed-size containers that hold compressed column data. The buffer pool (Lesson 10) will manage these pages in memory, and the columnar writer (Lesson 11) will fill them with your compressed data.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- converting `PageType` to and from a `u8` discriminant
- [IO and Serialization](../concepts/io_and_serialization.md) -- using the `byteorder` crate to write/read multi-byte integers in a specific endianness
- [Slices and Bytes](../concepts/slices_and_bytes.md) -- working with `&[u8]` slices, `Vec<u8>` buffers, and sub-slice indexing

## Key Patterns

### Binary Serialization with byteorder
The `byteorder` crate provides `WriteBytesExt` and `ReadBytesExt` traits that extend `Write` and `Read`. Wrap a `Vec<u8>` or `&[u8]` in a `Cursor` to get positioned I/O. Think of it like a tape recorder head -- it moves forward as you read or write, and the endianness setting ensures bytes go in the right order.

```rust
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

struct Sensor {
    id: u32,
    temperature: u16,
}

impl Sensor {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.write_u32::<LittleEndian>(self.id).unwrap();
        buf.write_u16::<LittleEndian>(self.temperature).unwrap();
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut cursor = Cursor::new(bytes);
        let id = cursor.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
        let temperature = cursor.read_u16::<LittleEndian>().map_err(|e| e.to_string())?;
        Ok(Self { id, temperature })
    }
}
```

Your `PageHeader::to_bytes` and `from_bytes` follow this same pattern. Remember that the header has a fixed `SIZE` of 16 bytes -- make sure your fields add up.

### CRC32 Checksums
The `crc32fast` crate computes checksums over byte slices. This catches corruption.
Think of it like a receipt total -- if the itemized amounts do not add up to the total, something was changed.

```rust
fn verify_payload(data: &[u8], expected: u32) -> bool {
    let computed = crc32fast::hash(data);
    computed == expected
}
```

Compute the checksum over the page's data region (not the header), then store it in the header before writing to disk.

### Builder Pattern
A builder accumulates state and produces a finalized object. Think of building a sandwich -- you add layers one at a time, and "finish" wraps it up and hands you the completed product.

```rust
struct MessageBuilder {
    buffer: Vec<u8>,
    offset: usize,
}

impl MessageBuilder {
    fn append(&mut self, payload: &[u8]) -> Result<usize, String> {
        if self.offset + payload.len() > self.buffer.len() {
            return Err("no space".into());
        }
        let start = self.offset;
        self.buffer[start..start + payload.len()].copy_from_slice(payload);
        self.offset += payload.len();
        Ok(start)
    }
}
```

Your `PageBuilder` tracks a `write_offset` and appends data into the page's data region, returning the offset where each piece was written.

### Discriminant-to-Enum Conversion
`PageType` uses a `u8` discriminant on disk. You need a `from_u8` function that converts
back, returning `None` for unknown values. Think of it like a menu number at a restaurant --
valid numbers get you a dish, invalid numbers get you "sorry, not on the menu."

```rust
enum PageType { Data = 1, Index = 2, Overflow = 3, Meta = 4 }

fn from_u8(val: u8) -> Option<PageType> {
    match val {
        1 => Some(PageType::Data),
        2 => Some(PageType::Index),
        // ...
        _ => None,
    }
}
```

## Step-by-Step Implementation Order
1. Start with `PageType::from_u8()` -- match on known discriminants, return None for unknown values
2. Implement `PageHeader::to_bytes()` -- write `page_type as u8`, then a padding byte, then `page_id` as `u32`, `checksum` as `u32`, `free_space` as `u16`, `num_records` as `u16`. Use `WriteBytesExt` methods. Verify the total is exactly 16 bytes.
3. Implement `PageHeader::from_bytes()` -- wrap the slice in a `Cursor` and read the fields back in the same order. Convert the first byte to `PageType` using `from_u8`, returning an error for unknown values.
4. Implement `Page::new()` -- allocate a `Vec<u8>` of `page_size - PageHeader::SIZE` for the data region, and set `free_space` to that capacity.
5. Implement `write_data` and `read_data` -- bounds-check the offset and length against the data buffer, then copy or slice.
6. Implement `compute_checksum` and `update_checksum` using `crc32fast::hash` on `self.data`; verify_checksum compares stored vs computed.
7. Implement `to_bytes` and `from_bytes` for the full `Page` -- concatenate header bytes and data bytes; on read, split at `PageHeader::SIZE`.
8. Implement `PageBuilder` -- `new` creates a fresh `Page`, `append` writes at the current offset and advances it, `remaining` returns how much space is left, and `finish` updates the checksum and returns the page.
9. Handle overflow in PageBuilder: if `append` would exceed remaining space, return an error.

## Common Mistakes
- **Header size mismatch**: If your serialized header is not exactly `PageHeader::SIZE` bytes, the data region offset will be wrong, and `from_bytes` will misparse. Double-check that all fields (including any padding bytes) sum to the expected constant.
- **Checksumming the wrong region**: The checksum covers only the data region, not the header. If you include the header in the hash, `verify_checksum` will fail because the header itself contains the checksum field (chicken-and-egg problem).
- **Not returning the write offset from `append`**: The PageBuilder's `append` method must return the offset where the data was written, so the caller can later read back specific records. Forgetting this makes the builder much less useful.

## Reading the Tests
- **`test_page_creation`** creates a page with ID 0 and Data type, checking that the header stores the correct values and `page_size()` matches the default.
- **`test_page_write_read`** writes "hello world" at offset 0 and reads it back, confirming your `write_data`/`read_data` faithfully stores and returns bytes.
- **`test_page_free_space`** checks that a fresh page has positive free space. The exact tracking depends on your implementation.
- **`test_page_checksum`** writes data, calls `update_checksum()`, verifies it passes, then corrupts a byte and checks that verification fails. This confirms your CRC32 logic detects single-byte corruption.
- **`test_page_serialize_roundtrip`** creates a page with ID 42 and Index type, writes data, updates checksum, serializes to bytes, deserializes, and checks every field matches. This is the full persistence path.
- **`test_page_header_roundtrip`** creates a header with specific values (including checksum 0xDEADBEEF) and checks that `to_bytes`/`from_bytes` preserves all fields. The assertion on SIZE confirms the header is exactly 16 bytes.
- **`test_page_builder`** appends two records, checks that offsets advance, remaining space decreases, and `finish()` produces a page with a valid checksum.
- **`test_page_builder_overflow`** uses a tiny 64-byte page size, appends 30 bytes, then tries to append 40 more. The second append must fail with an error rather than corrupting memory.
- **`test_page_type_from_u8`** checks that discriminants 1-4 map to Data, Index, Overflow, Meta respectively, and that 0 and 255 return None. This tells you the exact discriminant values.
- **`test_page_boundary_write`** calculates the maximum data capacity (page_size - header size) and writes exactly that many bytes, confirming writes up to the boundary succeed.

## What Comes Next
With pages as the storage unit, Lesson 10 builds the **buffer pool** -- an in-memory
cache that manages a fixed number of page frames, evicts LRU pages, and flushes dirty
pages to a `DiskManager` backend. Your `Page::to_bytes` and `Page::from_bytes` methods
will be used by the buffer pool for disk I/O.
