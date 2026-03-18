//! # Lesson 10: Buffer Pool — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Basic page allocation (`test_buffer_pool_new_page`)
//! 2. Disk manager basics (`test_inmemory_disk_manager`)
//! 3. Cache behavior (`test_buffer_pool_cache_hit`)
//! 4. Edge cases (capacity limit, empty pool)
//! 5. Eviction policies (`test_buffer_pool_lru_eviction`, `test_buffer_pool_pin_prevents_eviction`)
//! 6. Dirty page management (`test_buffer_pool_dirty_flush`, `test_buffer_pool_flush_all`)

use quackdb::storage::buffer_pool::{BufferPool, InMemoryDiskManager, DiskManager, PageId};
use quackdb::storage::page::PageType;

// ── 1. Basic page allocation ────────────────────────────────────────

#[test]
fn test_buffer_pool_new_page() {
    let disk = InMemoryDiskManager::new(8192);
    let mut pool = BufferPool::new(disk, 10);

    let page_id = pool.new_page(PageType::Data).unwrap();
    assert_eq!(pool.size(), 1, "creating a page must add exactly one entry to the pool");

    let page = pool.fetch_page(page_id).unwrap();
    assert_eq!(page.header.page_type, PageType::Data, "fetched page must retain the type it was created with");
}

// ── 2. Disk manager basics ──────────────────────────────────────────

#[test]
fn test_inmemory_disk_manager() {
    let mut disk = InMemoryDiskManager::new(8192);
    let id = disk.allocate_page();

    // Write a page
    let mut page = quackdb::storage::page::Page::new_default(id, PageType::Data);
    page.write_data(0, b"test").unwrap();
    page.update_checksum();
    disk.write_page(&page).unwrap();

    // Read it back
    let read_page = disk.read_page(id).unwrap();
    assert_eq!(read_page.read_data(0, 4).unwrap(), b"test", "disk manager must faithfully persist and retrieve page contents");
}

// ── 3. Cache behavior ──────────────────────────────────────────────

#[test]
fn test_buffer_pool_cache_hit() {
    let disk = InMemoryDiskManager::new(8192);
    let mut pool = BufferPool::new(disk, 10);

    let page_id = pool.new_page(PageType::Data).unwrap();
    // Write data
    {
        let page = pool.fetch_page_mut(page_id).unwrap();
        page.write_data(0, b"cached data").unwrap();
    }
    pool.unpin_page(page_id, true).unwrap();

    // Fetch again — should be a cache hit
    let page = pool.fetch_page(page_id).unwrap();
    let data = page.read_data(0, 11).unwrap();
    assert_eq!(data, b"cached data", "buffer pool cache hit must return the same data without disk I/O");
}

// ── 4. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_buffer_pool_capacity_limit() {
    let disk = InMemoryDiskManager::new(8192);
    let mut pool = BufferPool::new(disk, 5);

    let mut page_ids = Vec::new();
    for _ in 0..5 {
        let id = pool.new_page(PageType::Data).unwrap();
        pool.unpin_page(id, false).unwrap();
        page_ids.push(id);
    }
    assert_eq!(pool.size(), 5, "pool size must equal the number of pages when under capacity");
}

#[test]
fn test_buffer_pool_single_page_capacity() {
    // Edge case: pool with capacity of only 1 page
    let disk = InMemoryDiskManager::new(8192);
    let mut pool = BufferPool::new(disk, 1);

    let p1 = pool.new_page(PageType::Data).unwrap();
    pool.unpin_page(p1, false).unwrap();
    assert_eq!(pool.size(), 1, "pool with capacity 1 should hold exactly one page");
}

// ── 5. Eviction policies ───────────────────────────────────────────

#[test]
fn test_buffer_pool_lru_eviction() {
    let disk = InMemoryDiskManager::new(8192);
    let mut pool = BufferPool::new(disk, 3);

    // Create 3 pages filling the pool
    let p1 = pool.new_page(PageType::Data).unwrap();
    let p2 = pool.new_page(PageType::Data).unwrap();
    let p3 = pool.new_page(PageType::Data).unwrap();

    pool.unpin_page(p1, false).unwrap();
    pool.unpin_page(p2, false).unwrap();
    pool.unpin_page(p3, false).unwrap();

    // Creating a 4th page should evict the LRU page (p1)
    let p4 = pool.new_page(PageType::Data).unwrap();
    assert_eq!(pool.size(), 3, "pool must evict before growing beyond its capacity");
}

#[test]
fn test_buffer_pool_pin_prevents_eviction() {
    let disk = InMemoryDiskManager::new(8192);
    let mut pool = BufferPool::new(disk, 2);

    let p1 = pool.new_page(PageType::Data).unwrap();
    let p2 = pool.new_page(PageType::Data).unwrap();
    // Don't unpin p1 — it's still pinned

    pool.unpin_page(p2, false).unwrap();

    // Creating p3 should evict p2 (unpinned), not p1 (pinned)
    let p3 = pool.new_page(PageType::Data).unwrap();
    assert!(pool.fetch_page(p1).is_ok(), "pinned pages must never be evicted, even under memory pressure");
}

// ── 6. Dirty page management ───────────────────────────────────────

#[test]
fn test_buffer_pool_dirty_flush() {
    let disk = InMemoryDiskManager::new(8192);
    let mut pool = BufferPool::new(disk, 2);

    let page_id = pool.new_page(PageType::Data).unwrap();
    {
        let page = pool.fetch_page_mut(page_id).unwrap();
        page.write_data(0, b"dirty data").unwrap();
    }
    pool.unpin_page(page_id, true).unwrap();
    pool.flush_page(page_id).unwrap();

    // Force eviction by filling pool
    let _ = pool.new_page(PageType::Data).unwrap();
    pool.unpin_page(page_id, false).unwrap_or(());
    let _ = pool.new_page(PageType::Data).unwrap();

    // Re-fetch should get the flushed data
    let page = pool.fetch_page(page_id).unwrap();
    let data = page.read_data(0, 10).unwrap();
    assert_eq!(data, b"dirty data", "flushed dirty page must persist to disk and survive eviction");
}

#[test]
fn test_buffer_pool_flush_all() {
    let disk = InMemoryDiskManager::new(8192);
    let mut pool = BufferPool::new(disk, 5);

    for _ in 0..3 {
        let id = pool.new_page(PageType::Data).unwrap();
        {
            let page = pool.fetch_page_mut(id).unwrap();
            page.write_data(0, b"data").unwrap();
        }
        pool.unpin_page(id, true).unwrap();
    }

    pool.flush_all().unwrap();
}
