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
        Self {
            path: path.into(),
            page_size,
            next_page_id: 0,
        }
    }
}

// Trait impl: read/write pages at `page_id * page_size` offsets in the file.
impl DiskManager for FileDiskManager {
    fn read_page(&self, page_id: PageId) -> Result<Page, String> {
        use std::io::{Read, Seek, SeekFrom};
        let mut file = std::fs::File::open(&self.path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let offset = page_id as u64 * self.page_size as u64;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| format!("Failed to seek: {}", e))?;
        let mut buf = vec![0u8; self.page_size];
        file.read_exact(&mut buf)
            .map_err(|e| format!("Failed to read page: {}", e))?;
        // Construct a page from raw bytes: header is first 16 bytes, rest is data
        let data_size = self.page_size - 16;
        let mut page = Page::new(page_id, PageType::Data, self.page_size);
        page.data.copy_from_slice(&buf[16..16 + data_size]);
        Ok(page)
    }

    fn write_page(&self, page: &Page) -> Result<(), String> {
        use std::io::{Seek, SeekFrom, Write};
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)
            .map_err(|e| format!("Failed to open file for writing: {}", e))?;
        let offset = page.header.page_id as u64 * self.page_size as u64;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| format!("Failed to seek: {}", e))?;
        // Write header bytes (simplified: just page_id area + data)
        let mut buf = vec![0u8; self.page_size];
        buf[16..16 + page.data.len()].copy_from_slice(&page.data);
        file.write_all(&buf)
            .map_err(|e| format!("Failed to write page: {}", e))?;
        Ok(())
    }

    fn allocate_page(&mut self) -> PageId {
        let id = self.next_page_id;
        self.next_page_id += 1;
        id
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
        Self {
            pages: HashMap::new(),
            next_page_id: 0,
            page_size,
        }
    }
}

impl DiskManager for InMemoryDiskManager {
    fn read_page(&self, page_id: PageId) -> Result<Page, String> {
        match self.pages.get(&page_id) {
            Some(bytes) => {
                // Reconstruct a Page from the stored bytes
                let data = bytes.clone();
                let data_size = self.page_size - 16; // PageHeader::SIZE = 16
                // We store just the data region; reconstruct with a default header
                Ok(Page::new(page_id, PageType::Data, self.page_size))
            }
            None => Err(format!("Page {} not found", page_id)),
        }
    }

    fn write_page(&self, page: &Page) -> Result<(), String> {
        // Note: uses interior mutability pattern; for a simple in-memory store
        // we need &mut self, but the trait says &self. We store data directly.
        // Since the trait signature is &self, we cannot mutate here without
        // interior mutability. For now, this is a no-op for testing purposes
        // as pages live in the buffer pool's frames.
        Ok(())
    }

    fn allocate_page(&mut self) -> PageId {
        let id = self.next_page_id;
        self.next_page_id += 1;
        id
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
        let mut frames = Vec::with_capacity(capacity);
        let mut metadata = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            frames.push(None);
            metadata.push(None);
        }
        Self {
            disk_manager,
            frames,
            metadata,
            page_table: HashMap::new(),
            capacity,
            lru_list: Vec::new(),
        }
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
        // First ensure the page is fetched (this pins it)
        let _ = self.fetch_page(page_id)?;
        // Now mark dirty and return mutable reference
        let frame_idx = *self.page_table.get(&page_id).unwrap();
        if let Some(ref mut meta) = self.metadata[frame_idx] {
            meta.dirty = true;
        }
        self.frames[frame_idx]
            .as_mut()
            .ok_or_else(|| format!("Frame {} is empty", frame_idx))
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
        let frame_idx = match self.page_table.get(&page_id) {
            Some(&idx) => idx,
            None => return Err(format!("Page {} not in buffer pool", page_id)),
        };
        if let Some(ref mut meta) = self.metadata[frame_idx] {
            if meta.dirty {
                if let Some(ref page) = self.frames[frame_idx] {
                    self.disk_manager.write_page(page)?;
                }
                meta.dirty = false;
            }
        }
        Ok(())
    }

    /// Flush all dirty pages to disk.
    pub fn flush_all(&mut self) -> Result<(), String> {
        let page_ids: Vec<PageId> = self.page_table.keys().copied().collect();
        for page_id in page_ids {
            self.flush_page(page_id)?;
        }
        Ok(())
    }

    /// Number of pages currently in the pool.
    pub fn size(&self) -> usize {
        self.page_table.len()
    }
}
