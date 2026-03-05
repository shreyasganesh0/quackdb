//! Lesson 10: Buffer Pool Tests

use quackdb::storage::buffer_pool::{BufferPool, InMemoryDiskManager, DiskManager, PageId};
use quackdb::storage::page::PageType;

#[test]
fn test_buffer_pool_new_page() {
    let disk = InMemoryDiskManager::new(8192);
    let mut pool = BufferPool::new(disk, 10);

    let page_id = pool.new_page(PageType::Data).unwrap();
    assert_eq!(pool.size(), 1);

    let page = pool.fetch_page(page_id).unwrap();
    assert_eq!(page.header.page_type, PageType::Data);
}

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
    assert_eq!(data, b"cached data");
}

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
    assert_eq!(pool.size(), 3); // Still at capacity
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
    assert!(pool.fetch_page(p1).is_ok()); // p1 still in pool
}

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
    assert_eq!(data, b"dirty data");
}

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
    assert_eq!(pool.size(), 5);
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
    assert_eq!(read_page.read_data(0, 4).unwrap(), b"test");
}
