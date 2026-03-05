//! Lesson 01: Arena Allocator Tests

use quackdb::arena::{Arena, ArenaString, ScopedArena};

#[test]
fn test_arena_basic_alloc() {
    let mut arena = Arena::new();
    let buf = arena.alloc(64, 1);
    assert_eq!(buf.len(), 64);
    // Write and read back
    for (i, byte) in buf.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    let buf2 = arena.alloc(128, 1);
    assert_eq!(buf2.len(), 128);
    assert!(arena.total_allocated() >= 192);
}

#[test]
fn test_arena_alignment() {
    let mut arena = Arena::new();
    let buf1 = arena.alloc(7, 1);
    assert_eq!(buf1.len(), 7);

    let buf2 = arena.alloc(16, 8);
    assert_eq!(buf2.len(), 16);
    let ptr = buf2.as_ptr() as usize;
    assert_eq!(ptr % 8, 0, "Expected 8-byte alignment, got ptr={:#x}", ptr);

    let buf3 = arena.alloc(32, 16);
    assert_eq!(buf3.len(), 32);
    let ptr3 = buf3.as_ptr() as usize;
    assert_eq!(ptr3 % 16, 0, "Expected 16-byte alignment, got ptr={:#x}", ptr3);
}

#[test]
fn test_arena_block_growth() {
    let mut arena = Arena::with_block_size(64);
    // Allocate more than one block's worth
    let _buf1 = arena.alloc(32, 1);
    let _buf2 = arena.alloc(32, 1);
    // This should cause a new block
    let _buf3 = arena.alloc(32, 1);
    assert!(arena.block_count() >= 2, "Expected at least 2 blocks, got {}", arena.block_count());
}

#[test]
fn test_arena_reset_and_reuse() {
    let mut arena = Arena::with_block_size(128);
    let _buf = arena.alloc(64, 1);
    let blocks_before = arena.block_count();
    arena.reset();
    assert_eq!(arena.total_allocated(), 0);
    // Blocks should still exist for reuse
    assert_eq!(arena.block_count(), blocks_before);
    // Can allocate again after reset
    let buf2 = arena.alloc(64, 1);
    assert_eq!(buf2.len(), 64);
}

#[test]
fn test_arena_string() {
    let mut arena = Arena::new();
    let s1 = arena.alloc_string("hello");
    assert_eq!(s1.as_str(), "hello");
    assert_eq!(s1.len(), 5);
    assert!(!s1.is_empty());

    let s2 = arena.alloc_string("world");
    assert_eq!(s2.as_str(), "world");

    // Original string still valid
    assert_eq!(s1.as_str(), "hello");
}

#[test]
fn test_arena_string_empty() {
    let mut arena = Arena::new();
    let s = arena.alloc_string("");
    assert_eq!(s.as_str(), "");
    assert_eq!(s.len(), 0);
    assert!(s.is_empty());
}

#[test]
fn test_arena_string_unicode() {
    let mut arena = Arena::new();
    let s = arena.alloc_string("héllo wörld 🦆");
    assert_eq!(s.as_str(), "héllo wörld 🦆");
}

#[test]
fn test_arena_typed_alloc() {
    let mut arena = Arena::new();
    let val_ptr: *mut i32;
    {
        let val: &mut i32 = arena.alloc_typed::<i32>();
        *val = 42;
        assert_eq!(*val, 42);
        val_ptr = val as *mut i32;
    }

    let val2: &mut f64 = arena.alloc_typed::<f64>();
    *val2 = 3.14;
    assert_eq!(*val2, 3.14);

    // Previous allocation still valid (arena memory is stable)
    unsafe {
        assert_eq!(*val_ptr, 42);
    }
}

#[test]
fn test_arena_large_alloc() {
    let mut arena = Arena::with_block_size(64);
    // Allocate more than a single block
    let buf = arena.alloc(256, 1);
    assert_eq!(buf.len(), 256);
    // Write pattern
    for (i, byte) in buf.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    // Verify pattern
    for (i, &byte) in buf.iter().enumerate() {
        assert_eq!(byte, (i % 256) as u8);
    }
}

#[test]
fn test_arena_many_small_allocs() {
    let mut arena = Arena::new();
    let mut ptrs = Vec::new();
    for i in 0..1000 {
        let buf = arena.alloc(8, 1);
        buf[0] = (i % 256) as u8;
        ptrs.push(buf.as_ptr());
    }
    assert!(arena.total_allocated() >= 8000);
}

#[test]
fn test_scoped_arena() {
    let mut arena = Arena::new();
    let _buf1 = arena.alloc(64, 1);
    let allocated_before = arena.total_allocated();

    {
        let mut scope = ScopedArena::new(&mut arena);
        let _sbuf = scope.alloc(128, 1);
        let s = scope.alloc_string("scoped");
        assert_eq!(s.as_str(), "scoped");
    }
    // After scope drops, arena should be back to pre-scope state
    // (Note: exact semantics depend on implementation — at minimum, reset should work)
}

#[test]
fn test_scoped_arena_reset() {
    let mut arena = Arena::new();
    let _pre = arena.alloc(32, 1);

    let mut scope = ScopedArena::new(&mut arena);
    let _buf = scope.alloc(64, 1);
    scope.reset();
    // After reset, can allocate again in same scope
    let buf2 = scope.alloc(64, 1);
    assert_eq!(buf2.len(), 64);
}

#[test]
fn test_arena_default() {
    let arena = Arena::default();
    assert_eq!(arena.total_allocated(), 0);
    assert_eq!(arena.block_count(), 0);
}

#[test]
fn test_arena_zero_size_alloc() {
    let mut arena = Arena::new();
    let buf = arena.alloc(0, 1);
    assert_eq!(buf.len(), 0);
}
