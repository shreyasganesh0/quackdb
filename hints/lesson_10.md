# Lesson 10: Buffer Pool

## What You're Building
A buffer pool manager that caches a fixed number of pages in memory and evicts the least-recently-used page when space runs out. It sits between the execution engine and disk, providing `fetch_page`, `unpin_page`, `new_page`, and `flush_page` operations. The disk backend is abstracted behind a `DiskManager` trait, allowing an in-memory implementation for testing and a file-backed one for production.

> **Unified Concept:** This is ONE concept with three faces: **fetch** (load a page from disk into memory), **pin/unpin** (reference counting so in-use pages are not evicted), and **evict** (make room by writing a dirty page back and freeing its frame). They are phases of the same lifecycle: a page is fetched, pinned while in use, unpinned when done, and eventually evicted to make room for another. The DiskManager, LRU list, and page table are just the bookkeeping that makes this lifecycle work.

## Concept Recap
Building on Lesson 09: You built `Page` with serialization, checksums, and the `PageBuilder`. Now you need a manager that caches pages in memory, loads them from disk on demand, and writes dirty pages back. The buffer pool uses `Page::to_bytes()` and `Page::from_bytes()` for disk persistence, and page IDs from your header to track which pages are cached.

## Rust Concepts You'll Need
- [Traits and Derive](../concepts/traits_and_derive.md) -- defining the `DiskManager` trait with read/write/allocate methods
- [Generics](../concepts/generics.md) -- `BufferPool<D: DiskManager>` is parameterized over any disk backend
- [Trait Bounds](../concepts/trait_bounds.md) -- the `D: DiskManager` bound lets the pool call trait methods on its disk manager
- [Collections](../concepts/collections.md) -- `HashMap` for the page table mapping page IDs to frame indices
- [IO and Serialization](../concepts/io_and_serialization.md) -- converting pages to/from bytes for disk persistence

## Key Patterns

### Trait for Disk Abstraction
Defining a trait lets you swap implementations without changing the buffer pool logic.
Think of a universal charger -- it works with any phone as long as the phone has a USB-C
port (the trait). The buffer pool is the charger; the disk manager is the phone.

```rust
trait Cache {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: String);
}

struct InMemoryCache {
    data: HashMap<String, String>,
}

impl Cache for InMemoryCache {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
    fn set(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }
}
```

Your `InMemoryDiskManager` stores pages as `HashMap<PageId, Vec<u8>>`. When `write_page` is called, serialize the page to bytes and insert. When `read_page` is called, look up the bytes and deserialize.

### Generic Struct with Trait Bound
The buffer pool does not know which disk manager it uses at compile time -- only that it satisfies the trait. Think of a vending machine that accepts any coin implementing the "valid currency" interface.

```rust
struct Pool<S: Storage> {
    storage: S,
    slots: Vec<Option<Item>>,
    index: HashMap<u64, usize>,
}

impl<S: Storage> Pool<S> {
    fn fetch(&mut self, id: u64) -> Result<&Item, String> {
        // Check index, or load from storage and place into a slot
    }
}
```

### LRU Eviction with HashMap + Vec
A simple LRU can be implemented with a `Vec<usize>` tracking frame indices from least to most recently used, and a `HashMap<PageId, usize>` mapping page IDs to frame indices.
Think of a restaurant wait list -- the person who has been waiting longest is next to be seated (evicted from the waiting area).

```rust
// On access, move frame to the back (most recent):
fn touch(lru: &mut Vec<usize>, frame: usize) {
    if let Some(pos) = lru.iter().position(|&f| f == frame) {
        lru.remove(pos);
    }
    lru.push(frame);
}

// On eviction, pick from the front (least recent):
fn evict(lru: &mut Vec<usize>, pin_counts: &[u32]) -> Option<usize> {
    for i in 0..lru.len() {
        if pin_counts[lru[i]] == 0 {
            return Some(lru.remove(i));
        }
    }
    None // all frames are pinned
}
```

The key insight: only unpinned frames (pin_count == 0) are eligible for eviction.

### Pin/Unpin Protocol
Fetching a page increments its pin count (like checking out a library book). Unpinning
decrements it (returning the book). A pinned page cannot be evicted -- it is in active use.

```rust
// Analogy: library checkout system
struct BookCheckout {
    checkouts: HashMap<String, u32>,
}

impl BookCheckout {
    fn checkout(&mut self, book: &str) { *self.checkouts.entry(book.into()).or_default() += 1; }
    fn return_book(&mut self, book: &str) { if let Some(c) = self.checkouts.get_mut(book) { *c -= 1; } }
    fn can_archive(&self, book: &str) -> bool { self.checkouts.get(book).copied().unwrap_or(0) == 0 }
}
```

## Step-by-Step Implementation Order
1. Start with `InMemoryDiskManager::new()` -- initialize an empty `HashMap`, set `next_page_id` to 0, store `page_size`.
2. Implement the `DiskManager` trait for `InMemoryDiskManager` -- `write_page` serializes via `page.to_bytes()` and inserts; `read_page` looks up and calls `Page::from_bytes`; `allocate_page` increments and returns the ID.
3. Implement `BufferPool::new()` -- allocate `frames` and `metadata` as `Vec<Option<_>>` with `capacity` entries all set to `None`; create empty `page_table` and `lru_list`.
4. Implement `fetch_page` -- first check `page_table` for a cache hit (increment pin count, touch LRU, return reference). On a miss, find a free frame or evict an LRU victim, flush if dirty, load from disk, insert into frame slot and page table.
5. Implement `unpin_page` -- decrement pin count, mark dirty if requested. Add frame to LRU list if pin count reaches 0.
6. Implement `new_page` -- allocate a page ID from the disk manager, find a free frame (or evict), place the new page.
7. Implement `flush_page` and `flush_all` -- write the page to disk if dirty, clear the dirty flag.
8. Implement `fetch_page_mut` -- same as fetch_page but returns a mutable reference.

## Common Mistakes
- **Not flushing dirty pages before eviction**: When evicting a dirty page, you must write it to disk first. Otherwise the modifications are lost and re-fetching the page will return stale data.
- **Evicting pinned pages**: Only pages with pin_count == 0 are eviction candidates. If all frames are pinned and a new page is needed, you must return an error rather than evicting a pinned page.
- **Forgetting to update the page_table on eviction**: When you evict page A to make room for page B, you must remove A's entry from the page_table HashMap and insert B's. If you forget to remove A, stale lookups will point to the wrong frame.

## Reading the Tests
- **`test_buffer_pool_new_page`** creates a pool with 10 frames, creates one new page, checks `size() == 1`, and fetches it back verifying the page type. This is the basic create-and-retrieve path.
- **`test_buffer_pool_cache_hit`** creates a page, writes "cached data" via `fetch_page_mut`, unpins as dirty, fetches again, and reads the data back. The second fetch should be a cache hit returning the same data without disk I/O.
- **`test_buffer_pool_lru_eviction`** creates a pool with capacity 3, creates 3 pages and unpins all, then creates a 4th. The pool must evict the LRU page (p1) to stay at capacity 3. This tests your eviction logic.
- **`test_buffer_pool_pin_prevents_eviction`** creates a pool with capacity 2, creates 2 pages but only unpins p2. When creating p3, p2 (unpinned) must be evicted, NOT p1 (still pinned). Fetching p1 after must succeed.
- **`test_buffer_pool_dirty_flush`** writes data to a page, unpins as dirty, flushes, forces eviction, then re-fetches and checks the data persisted. This tests the full dirty-write-flush-reload cycle.
- **`test_buffer_pool_capacity_limit`** creates 5 pages in a pool of 5, unpins all, and checks size is 5. The pool should be at exactly its capacity.
- **`test_buffer_pool_flush_all`** creates 3 dirty pages and calls `flush_all()`. This must succeed without errors.
- **`test_inmemory_disk_manager`** directly tests the disk manager: allocate a page ID, create a page, write it, read it back, and verify the data. This confirms your disk abstraction works independently of the buffer pool.

## Rust Sidebar: Generic Trait Bounds
If you hit `the trait DiskManager is not implemented for D` or `no method named read_page found`, here's what's happening: `BufferPool<D>` is generic over `D`, but Rust does not know `D` has disk operations unless you add a trait bound. Without `D: DiskManager`, the compiler cannot resolve any method calls on the `storage` field.
The fix: declare the struct as `struct BufferPool<D: DiskManager>` and repeat the bound on every `impl<D: DiskManager> BufferPool<D>` block. This tells the compiler "D could be any type, but it must implement DiskManager, so I can call `.read_page()`, `.write_page()`, and `.allocate_page()` on it."

## What Comes Next
With pages cached in the buffer pool, Lesson 11 builds the **columnar file writer** --
a streaming writer that produces Parquet-like files with magic bytes, row groups, and
a footer. The writer will use your compression framework (L08) to compress column data
and write the results as pages or raw byte streams managed by the storage layer.
