# Lesson 01: Arena Allocator

## What You're Building
An arena (bump) allocator that hands out memory from pre-allocated blocks of bytes.
Databases use arenas to batch-allocate many short-lived objects during query execution,
then free everything at once instead of tracking individual allocations. Your arena
manages `Vec<Vec<u8>>` blocks, an `ArenaString` type backed by raw pointers, and a
`ScopedArena` that can roll back to a checkpoint.

## Rust Concepts You'll Need
- [Structs and Impl](../concepts/structs_and_impl.md) -- Arena fields (blocks, offset, block_size) and methods
- [Ownership and Borrowing](../concepts/ownership_and_borrowing.md) -- returning `&mut [u8]` that borrows from the arena
- [Unsafe Rust](../concepts/unsafe_rust.md) -- converting raw `*const u8` back to `&str` in ArenaString

## Key Patterns

### Bump Allocation
A bump allocator keeps an offset into the current block. Each allocation advances the
offset. When the block is full, allocate a new one. Think of a notepad: you write
line after line, and when the page is full you flip to the next page.

```rust
// Analogy: a simple log buffer (NOT the QuackDB solution)
struct LogBuffer {
    pages: Vec<Vec<u8>>,
    offset: usize,
    page_size: usize,
}

impl LogBuffer {
    fn write_entry(&mut self, data: &[u8]) -> &[u8] {
        if self.pages.is_empty() || self.offset + data.len() > self.page_size {
            self.pages.push(vec![0u8; self.page_size]);
            self.offset = 0;
        }
        let page = self.pages.last_mut().unwrap();
        let start = self.offset;
        page[start..start + data.len()].copy_from_slice(data);
        self.offset += data.len();
        &page[start..start + data.len()]
    }
}
```

### Alignment via Padding
Hardware expects certain types at aligned addresses. To align an offset to `align`
bytes, round up: `aligned = (offset + align - 1) & !(align - 1)`. The wasted bytes
between the old and new offset are padding.

```rust
// Analogy: seating people in a theater at every 4th seat
fn next_aligned_seat(current: usize, alignment: usize) -> usize {
    (current + alignment - 1) & !(alignment - 1)
}
assert_eq!(next_aligned_seat(5, 4), 8);
assert_eq!(next_aligned_seat(8, 4), 8);
```

### Unsafe Raw-Pointer-to-Slice Conversion
`ArenaString` stores `*const u8` and `len`. To get a `&str`, you must use
`unsafe { std::slice::from_raw_parts(ptr, len) }` and then `std::str::from_utf8_unchecked`.

```rust
// Analogy: reading a C string from a known-good buffer
fn read_tag(ptr: *const u8, len: usize) -> &str {
    unsafe {
        let bytes = std::slice::from_raw_parts(ptr, len);
        std::str::from_utf8_unchecked(bytes)
    }
}
```

## Step-by-Step Implementation Order
1. Start with `Arena::new()` and `with_block_size()` -- initialize empty blocks vec, offset at 0
2. Implement `alloc()` -- compute aligned offset, check if current block has room, allocate new block if not, copy nothing but return the mutable slice; remember to handle the case where requested size exceeds block_size
3. Implement `alloc_typed<T>()` -- call `alloc(size_of::<T>(), align_of::<T>())` and cast the slice pointer to `&mut T` using unsafe
4. Implement `alloc_string()` -- call `alloc()` for the bytes, copy the string in, construct ArenaString from the pointer and length
5. Implement `ArenaString::as_str()` -- unsafe `from_raw_parts` + `from_utf8_unchecked`
6. Implement `reset()` -- set offset to 0 and total_allocated to 0 but keep existing blocks
7. Implement `ScopedArena` -- snapshot current block index and offset on creation; delegate `alloc` to parent; `reset` restores the snapshot

## Reading the Tests
- **`test_arena_alignment`** allocates 7 bytes (unaligned), then 16 bytes with align=8, then 32 bytes with align=16. It checks the returned pointer modulo the alignment. This tells you your padding math must produce correctly aligned addresses.
- **`test_arena_typed_alloc`** allocates an `i32` and an `f64`, writes to them, and later reads the `i32` through a raw pointer. This confirms arena memory remains stable across allocations -- do not reallocate or move existing blocks.
- **`test_scoped_arena`** creates a scope, allocates inside it, and expects the parent arena to be usable after the scope ends. The scope captures the parent by `&'a mut Arena`.
