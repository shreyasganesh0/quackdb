//! Lesson 01: Arena Allocator
//!
//! A bump/arena allocator that hands out memory from pre-allocated `Vec<u8>` blocks.
//! Allocations are fast (pointer bump) but individual frees are not supported --
//! instead the entire arena (or a scope within it) is reset at once.
//!
//! Key Rust concepts: raw pointers, `unsafe` for pointer-to-slice conversion,
//! lifetimes tying borrowed slices to the arena, and `Copy` trait bounds.

const DEFAULT_BLOCK_SIZE: usize = 4096;

/// A simple arena (bump) allocator that allocates memory from pre-allocated blocks.
///
/// Memory is carved out of `Vec<u8>` blocks. When the current block is full,
/// a new one is allocated. All blocks are freed together on `reset` or drop.
pub struct Arena {
    blocks: Vec<Vec<u8>>,
    current_offset: usize,
    block_size: usize,
    total_allocated: usize,
}

impl Arena {
    /// Create a new arena with the default block size (4096 bytes).
    pub fn new() -> Self {
        Self::with_block_size(DEFAULT_BLOCK_SIZE)
    }

    /// Create a new arena with a custom block size.
    pub fn with_block_size(block_size: usize) -> Self {
        let mut blocks = Vec::new();
        blocks.push(vec![0u8; block_size]);
        Self {
            blocks,
            current_offset: 0,
            block_size,
            total_allocated: 0,
        }
    }

    /// Allocate `size` bytes with the given `align`ment from the arena.
    ///
    /// Returns a mutable slice of the allocated region. If the current block
    /// is too small, a new block is allocated.
    ///
    // Hint: round `current_offset` up to the next multiple of `align`.
    // Use unsafe to convert a raw pointer into a mutable slice (`std::slice::from_raw_parts_mut`).
    pub fn alloc(&mut self, size: usize, align: usize) -> &mut [u8] {
        todo!()
    }

    /// Allocate space for a value of type `T` and return a mutable reference.
    ///
    /// The allocation is sized and aligned to `T` via `std::mem::size_of` / `align_of`.
    // Hint: use `self.alloc(size_of::<T>(), align_of::<T>())` then
    // cast the returned `&mut [u8]` pointer to `&mut T` with unsafe.
    pub fn alloc_typed<T: Copy>(&mut self) -> &mut T {
        todo!()
    }

    /// Allocate and copy a string into the arena, returning an `ArenaString`.
    ///
    /// The string bytes live inside the arena; `ArenaString` holds a raw pointer
    /// and length back into that memory.
    pub fn alloc_string(&mut self, s: &str) -> ArenaString {
        todo!()
    }

    /// Reset the arena, reusing existing blocks without deallocating.
    ///
    /// After reset, all previous allocations are invalidated. The backing
    /// `Vec<u8>` blocks are retained so future allocations avoid re-allocating.
    pub fn reset(&mut self) {
        self.current_offset = 0;
        self.total_allocated = 0;
    }

    /// Total bytes allocated from this arena.
    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    /// Number of blocks currently in the arena.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}

// Default trait delegates to `Arena::new()`.
impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

/// A string allocated within an Arena. Provides a view into arena memory.
///
/// This is `Copy` because it is just a pointer + length (no ownership).
/// The caller must ensure the backing `Arena` outlives any `ArenaString`.
#[derive(Clone, Copy)]
pub struct ArenaString {
    // Raw pointer into arena-owned memory.
    ptr: *const u8,
    len: usize,
}

impl ArenaString {
    /// Get the string as a `&str` slice.
    ///
    // Hint: use `unsafe { std::slice::from_raw_parts(self.ptr, self.len) }`
    // then `std::str::from_utf8_unchecked` (the bytes came from a valid &str).
    pub fn as_str(&self) -> &str {
        todo!()
    }

    /// Get the length of the string in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// A scoped arena that can be used for temporary allocations.
///
/// Records the parent arena's position on creation, and rewinds back to
/// that position on `reset` (or drop), freeing only the scoped allocations.
// Hint: the lifetime `'a` ties this scope to the parent `Arena`,
// ensuring the parent cannot be used directly while a scope is active.
pub struct ScopedArena<'a> {
    parent: &'a mut Arena,
    start_block: usize,
    start_offset: usize,
}

impl<'a> ScopedArena<'a> {
    /// Create a new scoped arena from a parent arena.
    ///
    /// Captures the parent's current allocation position so it can be
    /// restored later via `reset`.
    pub fn new(parent: &'a mut Arena) -> Self {
        let start_block = parent.blocks.len().saturating_sub(1);
        let start_offset = parent.current_offset;
        Self {
            parent,
            start_block,
            start_offset,
        }
    }

    /// Allocate bytes within this scope. Delegates to the parent arena.
    pub fn alloc(&mut self, size: usize, align: usize) -> &mut [u8] {
        self.parent.alloc(size, align)
    }

    /// Allocate and copy a string into the scoped arena.
    pub fn alloc_string(&mut self, s: &str) -> ArenaString {
        self.parent.alloc_string(s)
    }

    /// Reset the scope, freeing all allocations made within it.
    ///
    /// Restores the parent arena's offset to where it was when this scope
    /// was created.
    pub fn reset(&mut self) {
        // Truncate any blocks allocated after the scope started
        self.parent.blocks.truncate(self.start_block + 1);
        self.parent.current_offset = self.start_offset;
    }
}
