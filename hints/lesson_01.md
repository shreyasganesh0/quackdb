# Lesson 01: Arena Allocator

## What You're Building
An arena (bump) allocator that hands out memory from pre-allocated blocks of bytes.
Databases use arenas to batch-allocate many short-lived objects during query execution,
then free everything at once instead of tracking individual allocations. Your arena
manages `Vec<Vec<u8>>` blocks, an `ArenaString` type backed by raw pointers, and a
`ScopedArena` that can roll back to a checkpoint.

**Core concept count: 2** — bump allocation (advance an offset through a block) and block management (allocate new blocks when full). Everything else (ArenaString, ScopedArena, alignment) is scaffolding that supports these two.

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
between the old and new offset are padding. Think of parking a wide truck: you cannot
park at any spot, you need one aligned to the truck's width.

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
This is like handing someone a street address and a house size -- they trust you that
the address is valid and the house is really that big.

```rust
// Analogy: reading a C string from a known-good buffer
fn read_tag(ptr: *const u8, len: usize) -> &str {
    unsafe {
        let bytes = std::slice::from_raw_parts(ptr, len);
        std::str::from_utf8_unchecked(bytes)
    }
}
```

### Block Reuse on Reset
When you call `reset()`, the arena does not deallocate blocks -- it simply moves
the offset back to zero. This is like erasing a whiteboard: the board is still there,
you just start writing from the top again. This avoids repeated allocation overhead.

```rust
// Analogy: reusable shipping containers
struct ContainerYard {
    containers: Vec<Vec<u8>>,
    next_slot: usize,
}

impl ContainerYard {
    fn clear(&mut self) {
        self.next_slot = 0;  // keep containers, just start over
    }
}
```

## Where to Start
Start with `Arena::new()` and `alloc()` — they are the heart of the lesson and have the clearest tests. Once basic allocation works, add alignment. Then tackle `ArenaString` (the unsafe part). Leave `ScopedArena` for last.

## Step-by-Step Implementation Order
1. Start with `Arena::new()` and `with_block_size()` -- initialize empty blocks vec, offset at 0, store the block size
2. Implement `Arena::default()` -- delegate to `new()` so a fresh arena has zero blocks and zero bytes allocated
3. Implement `alloc()` -- compute aligned offset, check if current block has room, allocate a new block if not, return the mutable slice; handle the case where requested size exceeds block_size by allocating a larger block
4. Handle zero-size allocations in `alloc()` -- return an empty slice without allocating
5. Implement `alloc_typed<T>()` -- call `alloc(size_of::<T>(), align_of::<T>())` and cast the slice pointer to `&mut T` using unsafe
6. Implement `alloc_string()` -- call `alloc()` for the bytes, copy the string in, construct ArenaString from the pointer and length; handle empty strings gracefully
7. Implement `ArenaString::as_str()`, `len()`, `is_empty()` -- unsafe `from_raw_parts` + `from_utf8_unchecked` for as_str
8. Implement `reset()` -- set offset to 0 and total_allocated to 0 but keep existing blocks for reuse
9. Implement `ScopedArena` -- snapshot current block index and offset on creation; delegate `alloc` and `alloc_string` to parent; `reset` restores the snapshot

## Common Mistakes
- **Forgetting to track `total_allocated`**: Every `alloc` call must increment `total_allocated` by the number of bytes used (including alignment padding). Tests check this counter after multiple allocations.
- **Moving or reallocating existing blocks**: When a new block is needed, push a new one -- never resize an existing block. Earlier allocations returned pointers into those blocks, and moving them would invalidate those pointers.
- **Not handling oversized allocations**: If the requested size exceeds `block_size`, you still need to allocate. Create a block large enough for the request rather than panicking or returning an error.

## Reading the Tests
- **`test_arena_basic_alloc`** allocates 64 bytes then 128 bytes, writes a pattern into the first buffer, and checks that `total_allocated() >= 192`. This tells you that your arena must track cumulative bytes across allocations, not just the current block usage.
- **`test_arena_alignment`** allocates 7 bytes (unaligned), then 16 bytes with align=8, then 32 bytes with align=16. It checks the returned pointer modulo the alignment. This tells you your padding math must produce correctly aligned addresses.
- **`test_arena_block_growth`** uses a 64-byte block size and allocates 3 chunks of 32 bytes each. The third should spill into a new block. It asserts `block_count() >= 2`, confirming that your arena must lazily create blocks as needed.
- **`test_arena_reset_and_reuse`** allocates, records the block count, resets, then checks that `total_allocated()` is 0 but `block_count()` is unchanged. This means reset must zero the counter while keeping blocks for reuse, and you must be able to allocate again after a reset.
- **`test_arena_string`** allocates "hello" and "world" as ArenaStrings, then re-reads "hello" after the second allocation. This confirms that earlier ArenaString pointers remain valid -- the arena must not move or invalidate previous allocations.
- **`test_arena_string_empty`** allocates an empty string and checks that `as_str()` returns `""`, `len()` returns 0, and `is_empty()` returns true. Your empty-string path must not crash on a zero-length pointer.
- **`test_arena_typed_alloc`** allocates an `i32` and an `f64`, writes to them, and later reads the `i32` through a raw pointer. This confirms arena memory remains stable across allocations -- do not reallocate or move existing blocks.
- **`test_arena_large_alloc`** uses a 64-byte block size but requests 256 bytes. This oversized allocation must still succeed and return the full 256-byte buffer, meaning your alloc must handle sizes exceeding block_size.
- **`test_arena_zero_size_alloc`** requests 0 bytes and expects a 0-length slice back without panicking. Make sure your alignment and offset logic does not break on a zero-size request.
- **`test_scoped_arena`** creates a scope from a mutable arena reference, allocates memory and a string inside the scope, then drops the scope. The scoped arena must borrow the parent arena by `&mut Arena` and delegate allocations to it.
- **`test_scoped_arena_reset`** creates a scope, allocates, calls `reset()` on the scope, then allocates again. This confirms that resetting a scoped arena lets you reuse its portion of the parent's memory.
- **`test_arena_default`** checks that a default-constructed arena has 0 bytes allocated and 0 blocks, confirming lazy block allocation.

## Rust Sidebar: Unsafe Pointer Conversion
If you hit `cannot convert *const u8 to &str` or `expected &str, found *const u8`, here's what's happening: `ArenaString` stores a raw pointer (`*const u8`) because the arena owns the memory, not the string. Rust cannot verify the pointer is valid at compile time, so you must use `unsafe` to convert it back.
The fix: `unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.ptr, self.len)) }`. This is safe as long as the arena (which owns the memory) outlives the `ArenaString`. The `unsafe` block is telling the compiler "I guarantee this pointer is valid and points to valid UTF-8."

## What Comes Next
This arena allocator is the memory foundation for the entire database. In Lesson 02,
you'll build the **type system** (LogicalType, ScalarValue) that defines what kinds
of data QuackDB can store. Then Lesson 03 creates **columnar vectors** that use byte
buffers (similar to the arena's blocks) to store typed column data efficiently. Every
subsequent lesson builds on these first four foundational types.
