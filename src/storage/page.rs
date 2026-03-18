//! Lesson 09: Pages & Page Layout
//!
//! Fixed-size pages are the unit of I/O between memory and disk. Each page
//! has a header (type, ID, checksum, free space) followed by a data region.
//! Pages are typically 8 KB and are read/written atomically.
//!
//! Key Rust concepts: `byteorder` crate for endian-aware serialization,
//! `#[repr(u8)]` for enum-to-integer mapping, `Cursor` for in-memory I/O,
//! and CRC32 checksums for data integrity.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

/// Default page size: 8 KB.
pub const DEFAULT_PAGE_SIZE: usize = 8192;

/// Large page size: 64 KB.
pub const LARGE_PAGE_SIZE: usize = 65536;

/// Page types -- each page serves a specific purpose.
// The `#[repr(u8)]` attribute ensures the enum can be safely cast to/from u8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PageType {
    /// Regular data page holding column values.
    Data = 1,
    /// Index page (e.g., B-tree node).
    Index = 2,
    /// Overflow page for values that exceed the main page.
    Overflow = 3,
    /// Metadata page (e.g., file header, schema info).
    Meta = 4,
}

impl PageType {
    /// Convert a raw `u8` to a `PageType`, returning `None` for unknown values.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Data),
            2 => Some(Self::Index),
            3 => Some(Self::Overflow),
            4 => Some(Self::Meta),
            _ => None,
        }
    }
}

/// Header at the start of every page (fixed 16 bytes).
///
/// The header is always serialized in little-endian byte order.
#[derive(Debug, Clone)]
pub struct PageHeader {
    pub page_type: PageType,
    pub page_id: u32,
    /// CRC32 checksum of the page data (excluding the header).
    pub checksum: u32,
    /// Remaining free space in the page (bytes).
    pub free_space: u16,
    /// Number of records stored in this page.
    pub num_records: u16,
}

impl PageHeader {
    /// Serialized size of the header in bytes.
    pub const SIZE: usize = 16;

    /// Serialize the header to bytes (little-endian).
    // Hint: use `WriteBytesExt` methods (write_u8, write_u32::<LittleEndian>, etc.)
    // to write each field into a Vec<u8>.
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    /// Deserialize a header from bytes.
    // Hint: use `Cursor` + `ReadBytesExt` to read fields in the same order
    // they were written. Convert PageType with `from_u8`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        todo!()
    }
}

/// A fixed-size page of data.
///
/// Contains a `PageHeader` and a data region. The total size (header + data)
/// equals `page_size`.
pub struct Page {
    pub header: PageHeader,
    pub data: Vec<u8>,
    page_size: usize,
}

impl Page {
    /// Create a new empty page with the given page size.
    // Hint: allocate `page_size - PageHeader::SIZE` bytes for the data region.
    pub fn new(page_id: u32, page_type: PageType, page_size: usize) -> Self {
        todo!()
    }

    /// Create a new page with the default size (8 KB).
    pub fn new_default(page_id: u32, page_type: PageType) -> Self {
        Self::new(page_id, page_type, DEFAULT_PAGE_SIZE)
    }

    /// Write data into the page at the given byte offset within the data region.
    ///
    /// Returns an error if the write would exceed the data region.
    pub fn write_data(&mut self, offset: usize, data: &[u8]) -> Result<(), String> {
        todo!()
    }

    /// Read data from the page at the given byte offset.
    ///
    /// Returns a slice of `length` bytes, or an error if out of bounds.
    pub fn read_data(&self, offset: usize, length: usize) -> Result<&[u8], String> {
        todo!()
    }

    /// Get the amount of free space in the page's data region.
    pub fn free_space(&self) -> usize {
        todo!()
    }

    /// Compute the CRC32 checksum of the page data region.
    // Hint: use a CRC32 library or a simple implementation over `self.data`.
    pub fn compute_checksum(&self) -> u32 {
        todo!()
    }

    /// Recompute and store the checksum in the header.
    pub fn update_checksum(&mut self) {
        todo!()
    }

    /// Verify the stored checksum matches a fresh computation.
    pub fn verify_checksum(&self) -> bool {
        todo!()
    }

    /// Serialize the entire page (header + data) to bytes.
    // Hint: concatenate `header.to_bytes()` and `self.data`, padded to `page_size`.
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    /// Deserialize a page from bytes.
    // Hint: split bytes into header portion and data portion, then parse each.
    pub fn from_bytes(bytes: &[u8], page_size: usize) -> Result<Self, String> {
        todo!()
    }

    /// The total page size (header + data).
    pub fn page_size(&self) -> usize {
        self.page_size
    }
}

/// A page builder for incrementally constructing pages.
///
/// Tracks a write cursor so callers can `append` data without managing offsets.
pub struct PageBuilder {
    page: Page,
    write_offset: usize,
}

impl PageBuilder {
    /// Create a new page builder with an empty page.
    pub fn new(page_id: u32, page_type: PageType, page_size: usize) -> Self {
        todo!()
    }

    /// Append data to the page. Returns the offset where data was written.
    ///
    /// Returns an error if there is not enough remaining space.
    pub fn append(&mut self, data: &[u8]) -> Result<usize, String> {
        todo!()
    }

    /// Remaining writable space in the page.
    pub fn remaining(&self) -> usize {
        todo!()
    }

    /// Finalize the page, computing its checksum and returning the `Page`.
    pub fn finish(self) -> Page {
        todo!()
    }
}
