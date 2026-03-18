# I/O and Serialization

> **Prerequisites:** [traits_and_derive](./traits_and_derive.md)

## Quick Reference
- `Read` trait: `.read_exact(&mut buf)` reads exactly N bytes; `.read_to_end(&mut vec)` reads all
- `Write` trait: `.write_all(&buf)` writes all bytes (always use this over `.write()`)
- `Seek` trait: `.seek(SeekFrom::Start(n))` repositions within a stream
- `Cursor<Vec<u8>>` = in-memory buffer implementing `Read + Write + Seek` (great for tests)
- `BufReader`/`BufWriter` wrap any `Read`/`Write` with an internal buffer for performance

## Common Compiler Errors

**`error[E0277]: the trait bound 'impl Write: Seek' is not satisfied`**
Your function requires `Seek` but the caller passed a type that only implements `Write`.
Fix: change the function signature to only require `Write`, or ensure the caller provides a seekable type.

**`error[E0599]: no method named 'write_u32' found`**
You forgot to import the `byteorder` extension traits.
Fix: add `use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};`.

**`error[E0015]: cannot call non-const fn 'read_exact' in statics`**
You tried to do I/O in a const context. I/O is a runtime operation.
Fix: move the I/O into a function body or initialization block.

## When You'll Use This
- **Lesson 8 (Compression Frame):** packing struct fields into bytes and reading them back
- **Lesson 9 (Pages):** using `byteorder` to write/read multi-byte integers
- **Lesson 10 (Buffer Pool):** converting pages to/from bytes for disk persistence
- **Lesson 11 (Columnar Write):** writing structured data as bytes, serializing the footer
- **Lesson 12 (Columnar Read):** seeking to end, reading footer length, seeking back
- **Lesson 28 (WAL):** converting `WalEntry` to/from bytes for durable storage

## What This Is

Rust's standard library provides a set of traits for reading and writing bytes: `Read`, `Write`,
and `Seek`. If you come from Python, these are analogous to the methods on file objects (`read()`,
`write()`, `seek()`). In C++ they correspond to `std::istream`, `std::ostream`, and their
`seekg`/`seekp` methods. In Node.js, think of `Buffer` and readable/writable streams.

The key difference is that Rust makes these **traits**, meaning any type can implement them. A
`File` implements `Read + Write + Seek`, but so does a `Cursor<Vec<u8>>` (an in-memory buffer),
a `TcpStream`, or your own custom type. This abstraction lets you write functions that work
with any byte source or sink, which is critical for database engines that must read from disk
files, network sockets, or in-memory test buffers interchangeably.

For binary serialization -- encoding structs as sequences of bytes -- Rust does not include
helpers in the standard library. The `byteorder` crate fills this gap, providing extension
methods like `read_u32::<LittleEndian>()` that work on any `Read` implementor. This is similar
to Python's `struct.pack`/`struct.unpack` or Node's `Buffer.readUInt32LE()`.

## Syntax

```rust
use std::io::{Read, Write, Seek, SeekFrom, Cursor};

// Reading bytes from any source
fn count_bytes(reader: &mut impl Read) -> std::io::Result<usize> {
    let mut buf = Vec::new();
    let n = reader.read_to_end(&mut buf)?;  // ? propagates io::Error
    Ok(n)
}

// Writing bytes to any sink
fn write_header(writer: &mut impl Write) -> std::io::Result<()> {
    writer.write_all(b"MAGIC")?;   // write_all ensures ALL bytes are written
    writer.flush()?;                // flush buffered data
    Ok(())
}

// Seeking within a stream
fn read_at_offset(file: &mut (impl Read + Seek), offset: u64) -> std::io::Result<[u8; 4]> {
    file.seek(SeekFrom::Start(offset))?;
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;    // read_exact fills the whole buffer or errors
    Ok(buf)
}

fn main() {
    // Cursor: an in-memory buffer that implements Read + Write + Seek
    let mut cursor = Cursor::new(vec![0u8; 1024]);
    cursor.write_all(b"hello").unwrap();
    cursor.seek(SeekFrom::Start(0)).unwrap();
}
```

## Common Patterns

### Pattern 1: Binary Encoding with `byteorder`

```rust
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

struct PageHeader {
    page_id: u32,
    num_records: u16,
    checksum: u64,
}

fn serialize_header(header: &PageHeader, writer: &mut impl std::io::Write) -> std::io::Result<()> {
    writer.write_u32::<LittleEndian>(header.page_id)?;
    writer.write_u16::<LittleEndian>(header.num_records)?;
    writer.write_u64::<LittleEndian>(header.checksum)?;
    Ok(())
}

fn deserialize_header(reader: &mut impl std::io::Read) -> std::io::Result<PageHeader> {
    Ok(PageHeader {
        page_id: reader.read_u32::<LittleEndian>()?,
        num_records: reader.read_u16::<LittleEndian>()?,
        checksum: reader.read_u64::<LittleEndian>()?,
    })
}
```

### Pattern 2: Using `Cursor` for In-Memory Testing

```rust
use std::io::{Cursor, Read, Write, Seek, SeekFrom};

fn test_round_trip() {
    let mut buf = Cursor::new(Vec::new());

    // Write some data
    buf.write_all(&[1, 2, 3, 4]).unwrap();

    // Rewind and read it back
    buf.seek(SeekFrom::Start(0)).unwrap();
    let mut output = [0u8; 4];
    buf.read_exact(&mut output).unwrap();
    assert_eq!(output, [1, 2, 3, 4]);
}
```

### Pattern 3: `BufReader` and `BufWriter` for Performance

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

fn copy_file(src: &str, dst: &str) -> std::io::Result<()> {
    // BufReader wraps any Read with an internal buffer (default 8 KB),
    // reducing the number of actual syscalls.
    let mut reader = BufReader::new(File::open(src)?);
    let mut writer = BufWriter::new(File::create(dst)?);

    let mut chunk = [0u8; 4096];
    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 { break; }
        writer.write_all(&chunk[..n])?;
    }
    // BufWriter flushes on drop, but explicit flush catches errors
    writer.flush()?;
    Ok(())
}
```

## Gotchas

1. **`read()` vs `read_exact()`**: `read()` can return fewer bytes than the buffer size -- this
   is *not* an error. If you need exactly N bytes, use `read_exact()`, which returns an error if
   the stream ends early. Forgetting this leads to subtle data-corruption bugs, especially when
   reading from network sockets.

2. **Forgetting to seek after writing**: A `Cursor` tracks a position. After writing 100 bytes,
   the position is at byte 100. If you then try to read, you get nothing (you are at the end).
   You must `seek(SeekFrom::Start(0))` to rewind. This catches many people writing tests.

3. **`write()` vs `write_all()`**: Just like `read()`, a bare `write()` may not write all
   bytes. Always use `write_all()` unless you have a specific reason to handle partial writes
   yourself. In Python, `file.write()` always writes everything; in Rust it does not.

## Related Concepts

- [Traits and Derive](./traits_and_derive.md) -- `Read`, `Write`, `Seek` are traits; any type can implement them
- [Error Handling](./error_handling.md) -- I/O operations return `io::Result<T>` and use `?` for propagation
- [Generics](./generics.md) -- `fn write<W: Write>(w: &mut W)` makes functions work with any writer
- [Slices and Bytes](./slices_and_bytes.md) -- `&[u8]` byte slices are the currency of I/O operations

## Quick Reference

| Trait / Type          | Purpose                              | Python Equivalent        |
|-----------------------|--------------------------------------|--------------------------|
| `Read`                | Read bytes from a source             | `file.read()`            |
| `Write`              | Write bytes to a sink                | `file.write()`           |
| `Seek`               | Reposition within a stream           | `file.seek()`            |
| `Cursor<Vec<u8>>`    | In-memory read/write/seek buffer     | `io.BytesIO()`           |
| `BufReader<R>`       | Buffered wrapper around any `Read`   | (built-in to Python I/O) |
| `BufWriter<W>`       | Buffered wrapper around any `Write`  | (built-in to Python I/O) |
| `read_exact(&mut buf)`| Read exactly `buf.len()` bytes      | `file.read(n)`           |
| `write_all(&buf)`    | Write all bytes or error             | `file.write(buf)`        |
| `byteorder` crate    | Encode/decode integers with endianness | `struct.pack/unpack`   |
