//! Lesson 10: Buffer Pool Manager
//!
//! An LRU-based buffer pool that caches disk pages in memory. Pages are
//! "pinned" while in use (preventing eviction) and "unpinned" when done.
//! Dirty pages are flushed to disk on eviction or explicit flush.
//!
//! Key Rust concepts: trait objects (`DiskManager`), `HashMap` for the page
//! table, generic structs (`BufferPool<D: DiskManager>`), interior mutability
//! patterns, and LRU eviction policy.

use super::page::{Page, PageType, DEFAULT_PAGE_SIZE};
use std::collections::HashMap;

/// Unique identifier for a page.
pub type PageId = u32;

/// Trait for disk I/O operations.
///
/// Abstractions over this trait allow swapping between file-backed and
/// in-memory storage (useful for testing).
pub trait DiskManager {
    /// Read a page from disk by its ID.
    fn read_page(&self, page_id: PageId) -> Result<Page, String>;
    /// Write a page to disk, overwriting any existing content at that page ID.
    fn write_page(&self, page: &Page) -> Result<(), String>;
    /// Allocate and return a fresh page ID.
    fn allocate_page(&mut self) -> PageId;
}

/// A file-based disk manager that reads/writes pages to a single file.
///
/// Pages are stored at byte offset `page_id * page_size` within the file.
pub struct FileDiskManager {
    path: std::path::PathBuf,
    page_size: usize,
    next_page_id: PageId,
}

impl FileDiskManager {
    /// Create or open a file-backed disk manager at the given path.
    pub fn new(path: impl Into<std::path::PathBuf>, page_size: usize) -> Self {
        todo!()
    }
}

// Trait impl: read/write pages at `page_id * page_size` offsets in the file.
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

/// An in-memory disk manager for testing (no actual file I/O).
///
/// Stores serialized page bytes in a `HashMap` keyed by `PageId`.
pub struct InMemoryDiskManager {
    pages: HashMap<PageId, Vec<u8>>,
    next_page_id: PageId,
    page_size: usize,
}

impl InMemoryDiskManager {
    /// Create a new in-memory disk manager with the given page size.
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

/// Metadata for a single buffer frame (slot in the pool).
struct FrameMetadata {
    page_id: PageId,
    /// Number of active users holding a reference to this page.
    pin_count: u32,
    /// Whether the page has been modified since it was loaded.
    dirty: bool,
}

/// LRU-based buffer pool manager.
///
/// Manages a fixed number of frames (slots). When all frames are occupied
/// and a new page is requested, the least-recently-used unpinned frame is
/// evicted (flushed to disk if dirty).
// The generic parameter `D: DiskManager` allows plugging in different
// storage backends (file-based, in-memory, etc.).
pub struct BufferPool<D: DiskManager> {
    disk_manager: D,
    frames: Vec<Option<Page>>,
    metadata: Vec<Option<FrameMetadata>>,
    /// Maps page IDs to frame indices for O(1) lookup.
    page_table: HashMap<PageId, usize>,
    capacity: usize,
    /// LRU tracking: list of frame indices from least to most recently used.
    lru_list: Vec<usize>,
}

impl<D: DiskManager> BufferPool<D> {
    /// Create a new buffer pool with the given capacity (number of frames).
    // Hint: initialize `frames` and `metadata` as Vec of `None` with length `capacity`.
    pub fn new(disk_manager: D, capacity: usize) -> Self {
        todo!()
    }

    /// Fetch a page by ID, loading it from disk if not already in the pool.
    ///
    /// The page is pinned (pin_count incremented) to prevent eviction while
    /// the caller holds a reference.
    // Hint: check page_table first. On miss, find a free frame or evict
    // the LRU unpinned frame, then load from disk.
    pub fn fetch_page(&mut self, page_id: PageId) -> Result<&Page, String> {
        todo!()
    }

    /// Get a mutable reference to a fetched page, marking it dirty.
    pub fn fetch_page_mut(&mut self, page_id: PageId) -> Result<&mut Page, String> {
        todo!()
    }

    /// Unpin a page, allowing it to be evicted.
    ///
    /// If `dirty` is true, marks the page as dirty so it will be flushed
    /// before eviction.
    pub fn unpin_page(&mut self, page_id: PageId, dirty: bool) -> Result<(), String> {
        todo!()
    }

    /// Create a new page in the buffer pool.
    ///
    /// Allocates a page ID via the disk manager and places it in a frame.
    pub fn new_page(&mut self, page_type: PageType) -> Result<PageId, String> {
        todo!()
    }

    /// Flush a specific page to disk (write if dirty, then clear dirty flag).
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
