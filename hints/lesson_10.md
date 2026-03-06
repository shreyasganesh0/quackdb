# Lesson 10: Buffer Pool

## What You're Building
A buffer pool manager that caches a fixed number of pages in memory and evicts the least-recently-used page when space runs out. It sits between the execution engine and disk, providing `fetch_page`, `unpin_page`, `new_page`, and `flush_page` operations. The disk backend is abstracted behind a `DiskManager` trait, allowing an in-memory implementation for testing and a file-backed one for production.

## Rust Concepts You'll Need
- [Traits and Derive](../concepts/traits_and_derive.md) -- defining the `DiskManager` trait with read/write/allocate methods
- [Generics](../concepts/generics.md) -- `BufferPool<D: DiskManager>` is parameterized over any disk backend
- [Trait Bounds](../concepts/trait_bounds.md) -- the `D: DiskManager` bound lets the pool call trait methods on its disk manager
- [Collections](../concepts/collections.md) -- `HashMap` for the page table mapping page IDs to frame indices
- [IO and Serialization](../concepts/io_and_serialization.md) -- converting pages to/from bytes for disk persistence

## Key Patterns

### Trait for Disk Abstraction
Defining a trait lets you swap implementations without changing the buffer pool logic:

```rust
trait Cache {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: String);
}

struct RedisCache { /* ... */ }
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
The buffer pool does not know which disk manager it uses at compile time -- only that it satisfies the trait:

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
A simple LRU can be implemented with a `Vec<usize>` tracking frame indices from least to most recently used, and a `HashMap<PageId, usize>` mapping page IDs to frame indices:

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

## Step-by-Step Implementation Order
1. Start with `InMemoryDiskManager::new()` -- initialize an empty `HashMap`, set `next_page_id` to 0, store `page_size`.
2. Implement the `DiskManager` trait for `InMemoryDiskManager` -- `write_page` serializes via `page.to_bytes()` and inserts; `read_page` looks up and calls `Page::from_bytes`; `allocate_page` increments and returns the ID.
3. Implement `BufferPool::new()` -- allocate `frames` and `metadata` as `Vec<Option<_>>` with `capacity` entries all set to `None`; create empty `page_table` and `lru_list`.
4. Implement `fetch_page` -- first check `page_table` for a cache hit (increment pin count, touch LRU, return reference). On a miss, find a free frame or evict an LRU victim, flush if dirty, load from disk, insert into frame slot and page table.
5. Implement `unpin_page` -- decrement pin count, mark dirty if requested. Add frame to LRU list if pin count reaches 0.
6. Implement `new_page` -- allocate a page ID from the disk manager, find a free frame (or evict), place the new page.
7. Implement `flush_page` -- write the page to disk if dirty, clear the dirty flag.
8. Watch out for the eviction path: you must flush dirty victims before overwriting their frame slot. Also ensure you never evict a pinned page.

## Reading the Tests
- Look for a test that creates a `BufferPool` with a small capacity (e.g., 3 frames), fetches more pages than fit, and checks that eviction works correctly. The assertions will verify that previously unpinned pages are evicted while pinned ones remain.
- Look for a test that fetches a page, modifies it via `fetch_page_mut`, unpins it as dirty, then flushes and re-fetches to verify persistence. This confirms the dirty-tracking and flush path.
