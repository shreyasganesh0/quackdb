//! Lesson 01: Arena Allocator
//!
//! A bump/arena allocator using Vec<Vec<u8>> blocks for fast, bulk allocation.

const DEFAULT_BLOCK_SIZE: usize = 4096;

/// A simple arena (bump) allocator that allocates memory from pre-allocated blocks.
pub struct Arena {
    blocks: Vec<Vec<u8>>,
    current_offset: usize,
    block_size: usize,
    total_allocated: usize,
}

impl Arena {
    /// Create a new arena with the default block size (4096 bytes).
    pub fn new() -> Self {
        todo!()
    }

    /// Create a new arena with a custom block size.
    pub fn with_block_size(block_size: usize) -> Self {
        todo!()
    }

    /// Allocate `size` bytes with the given alignment from the arena.
    /// Returns a mutable slice to the allocated memory.
    pub fn alloc(&mut self, size: usize, align: usize) -> &mut [u8] {
        todo!()
    }

    /// Allocate space for a value of type T and return a mutable reference.
    pub fn alloc_typed<T: Copy>(&mut self) -> &mut T {
        todo!()
    }

    /// Allocate and copy a string into the arena, returning an ArenaString.
    pub fn alloc_string(&mut self, s: &str) -> ArenaString {
        todo!()
    }

    /// Reset the arena, reusing existing blocks without deallocating.
    pub fn reset(&mut self) {
        todo!()
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

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

/// A string allocated within an Arena. Provides a view into arena memory.
#[derive(Clone, Copy)]
pub struct ArenaString {
    ptr: *const u8,
    len: usize,
}

impl ArenaString {
    /// Get the string as a &str slice.
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
/// When dropped or reset, only the allocations made within this scope are freed.
pub struct ScopedArena<'a> {
    parent: &'a mut Arena,
    start_block: usize,
    start_offset: usize,
}

impl<'a> ScopedArena<'a> {
    /// Create a new scoped arena from a parent arena.
    pub fn new(parent: &'a mut Arena) -> Self {
        todo!()
    }

    /// Allocate bytes within this scope.
    pub fn alloc(&mut self, size: usize, align: usize) -> &mut [u8] {
        todo!()
    }

    /// Allocate and copy a string into the scoped arena.
    pub fn alloc_string(&mut self, s: &str) -> ArenaString {
        todo!()
    }

    /// Reset the scope, freeing all allocations made within it.
    pub fn reset(&mut self) {
        todo!()
    }
}
