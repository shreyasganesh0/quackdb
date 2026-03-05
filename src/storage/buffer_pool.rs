//! Lesson 10: Buffer Pool Manager
//!
//! LRU-based buffer pool with pin/unpin, dirty tracking, and disk I/O.

use super::page::{Page, PageType, DEFAULT_PAGE_SIZE};
use std::collections::HashMap;

/// Unique identifier for a page.
pub type PageId = u32;

/// Trait for disk I/O operations.
pub trait DiskManager {
    /// Read a page from disk.
    fn read_page(&self, page_id: PageId) -> Result<Page, String>;
    /// Write a page to disk.
    fn write_page(&self, page: &Page) -> Result<(), String>;
    /// Allocate a new page ID.
    fn allocate_page(&mut self) -> PageId;
}

/// A file-based disk manager.
pub struct FileDiskManager {
    path: std::path::PathBuf,
    page_size: usize,
    next_page_id: PageId,
}

impl FileDiskManager {
    /// Create or open a file-backed disk manager.
    pub fn new(path: impl Into<std::path::PathBuf>, page_size: usize) -> Self {
        todo!()
    }
}

impl DiskManager for FileDiskManager {
    fn read_page(&self, page_id: PageId) -> Result<Page, String> {
        todo!()
    }

    fn write_page(&self, page: &Page) -> Result<(), String> {
        todo!()
    }

    fn allocate_page(&mut self) -> PageId {
        todo!()
    }
}

/// An in-memory disk manager for testing.
pub struct InMemoryDiskManager {
    pages: HashMap<PageId, Vec<u8>>,
    next_page_id: PageId,
    page_size: usize,
}

impl InMemoryDiskManager {
    pub fn new(page_size: usize) -> Self {
        todo!()
    }
}

impl DiskManager for InMemoryDiskManager {
    fn read_page(&self, page_id: PageId) -> Result<Page, String> {
        todo!()
    }

    fn write_page(&self, page: &Page) -> Result<(), String> {
        todo!()
    }

    fn allocate_page(&mut self) -> PageId {
        todo!()
    }
}

/// Metadata for a buffer frame.
struct FrameMetadata {
    page_id: PageId,
    pin_count: u32,
    dirty: bool,
}

/// LRU-based buffer pool manager.
pub struct BufferPool<D: DiskManager> {
    disk_manager: D,
    frames: Vec<Option<Page>>,
    metadata: Vec<Option<FrameMetadata>>,
    page_table: HashMap<PageId, usize>,
    capacity: usize,
    /// LRU tracking: list of frame indices from least to most recently used.
    lru_list: Vec<usize>,
}

impl<D: DiskManager> BufferPool<D> {
    /// Create a new buffer pool with the given capacity (number of frames).
    pub fn new(disk_manager: D, capacity: usize) -> Self {
        todo!()
    }

    /// Fetch a page, loading it from disk if necessary.
    /// The page is pinned (pin_count incremented).
    pub fn fetch_page(&mut self, page_id: PageId) -> Result<&Page, String> {
        todo!()
    }

    /// Get a mutable reference to a fetched page.
    pub fn fetch_page_mut(&mut self, page_id: PageId) -> Result<&mut Page, String> {
        todo!()
    }

    /// Unpin a page, allowing it to be evicted.
    pub fn unpin_page(&mut self, page_id: PageId, dirty: bool) -> Result<(), String> {
        todo!()
    }

    /// Create a new page in the buffer pool.
    pub fn new_page(&mut self, page_type: PageType) -> Result<PageId, String> {
        todo!()
    }

    /// Flush a specific page to disk.
    pub fn flush_page(&mut self, page_id: PageId) -> Result<(), String> {
        todo!()
    }

    /// Flush all dirty pages to disk.
    pub fn flush_all(&mut self) -> Result<(), String> {
        todo!()
    }

    /// Number of pages currently in the pool.
    pub fn size(&self) -> usize {
        self.page_table.len()
    }
}
