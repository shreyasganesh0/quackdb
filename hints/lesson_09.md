# Lesson 09: Pages & Layout

## What You're Building
A fixed-size page abstraction that serves as the fundamental unit of storage in the database. Each page carries a header with metadata (type, ID, checksum, record count), followed by a data region. You will also build a `PageBuilder` that lets you incrementally fill a page and finalize it with a CRC32 checksum. Pages are what the buffer pool (next lesson) reads and writes to disk.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- converting `PageType` to and from a `u8` discriminant
- [IO and Serialization](../concepts/io_and_serialization.md) -- using the `byteorder` crate to write/read multi-byte integers in a specific endianness
- [Slices and Bytes](../concepts/slices_and_bytes.md) -- working with `&[u8]` slices, `Vec<u8>` buffers, and sub-slice indexing

## Key Patterns

### Binary Serialization with byteorder
The `byteorder` crate provides `WriteBytesExt` and `ReadBytesExt` traits that extend `Write` and `Read`. Wrap a `Vec<u8>` or `&[u8]` in a `Cursor` to get positioned I/O:

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
The `crc32fast` crate computes checksums over byte slices. This catches corruption:

```rust
fn verify_payload(data: &[u8], expected: u32) -> bool {
    let computed = crc32fast::hash(data);
    computed == expected
}
```

Compute the checksum over the page's data region (not the header), then store it in the header before writing to disk.

### Builder Pattern
A builder accumulates state and produces a finalized object:

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

## Step-by-Step Implementation Order
1. Start with `PageHeader::to_bytes()` -- write `page_type as u8`, then a padding byte, then `page_id` as `u32`, `checksum` as `u32`, `free_space` as `u16`, `num_records` as `u16`. Use `WriteBytesExt` methods. Verify the total is exactly 16 bytes.
2. Then implement `PageHeader::from_bytes()` -- wrap the slice in a `Cursor` and read the fields back in the same order. Convert the first byte to `PageType` using `from_u8`, returning an error for unknown values.
3. Implement `Page::new()` -- allocate a `Vec<u8>` of `page_size - PageHeader::SIZE` for the data region, and set `free_space` to that capacity.
4. Implement `write_data` and `read_data` -- bounds-check the offset and length against the data buffer, then copy or slice.
5. Implement `compute_checksum` using `crc32fast::hash` on `self.data`.
6. Implement `to_bytes` and `from_bytes` for the full `Page` -- concatenate header bytes and data bytes; on read, split at `PageHeader::SIZE`.
7. Implement `PageBuilder` -- `new` creates a fresh `Page`, `append` writes at the current offset and advances it, `remaining` returns how much space is left, and `finish` updates the checksum and returns the page.
8. Watch out for off-by-one errors in `free_space` tracking -- every `write_data` or `append` call should reduce the recorded free space.

## Reading the Tests
- Look for a round-trip test that creates a `PageHeader`, calls `to_bytes()` then `from_bytes()`, and asserts each field matches. This tells you the exact serialization order.
- Look for a test that writes data into a page via `PageBuilder::append`, calls `finish()`, then verifies `verify_checksum()` returns `true`. This confirms that `compute_checksum` and `update_checksum` must agree on which bytes are hashed.
