//! Lesson 09: Pages & Page Layout
//!
//! Fixed-size pages with headers, checksums, and serialization.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

/// Default page size: 8KB.
pub const DEFAULT_PAGE_SIZE: usize = 8192;

/// Large page size: 64KB.
pub const LARGE_PAGE_SIZE: usize = 65536;

/// Page types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PageType {
    Data = 1,
    Index = 2,
    Overflow = 3,
    Meta = 4,
}

impl PageType {
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

/// Header at the start of every page.
#[derive(Debug, Clone)]
pub struct PageHeader {
    pub page_type: PageType,
    pub page_id: u32,
    pub checksum: u32,
    pub free_space: u16,
    pub num_records: u16,
}

impl PageHeader {
    pub const SIZE: usize = 16;

    /// Serialize the header to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    /// Deserialize a header from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        todo!()
    }
}

/// A fixed-size page of data.
pub struct Page {
    pub header: PageHeader,
    pub data: Vec<u8>,
    page_size: usize,
}

impl Page {
    /// Create a new empty page with the given page size.
    pub fn new(page_id: u32, page_type: PageType, page_size: usize) -> Self {
        todo!()
    }

    /// Create a new page with the default size.
    pub fn new_default(page_id: u32, page_type: PageType) -> Self {
        Self::new(page_id, page_type, DEFAULT_PAGE_SIZE)
    }

    /// Write data into the page at the given offset.
    pub fn write_data(&mut self, offset: usize, data: &[u8]) -> Result<(), String> {
        todo!()
    }

    /// Read data from the page at the given offset.
    pub fn read_data(&self, offset: usize, length: usize) -> Result<&[u8], String> {
        todo!()
    }

    /// Get the amount of free space in the page.
    pub fn free_space(&self) -> usize {
        todo!()
    }

    /// Compute the CRC32 checksum of the page data.
    pub fn compute_checksum(&self) -> u32 {
        todo!()
    }

    /// Update the checksum in the header.
    pub fn update_checksum(&mut self) {
        todo!()
    }

    /// Verify the checksum is correct.
    pub fn verify_checksum(&self) -> bool {
        todo!()
    }

    /// Serialize the entire page (header + data) to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    /// Deserialize a page from bytes.
    pub fn from_bytes(bytes: &[u8], page_size: usize) -> Result<Self, String> {
        todo!()
    }

    /// Page size.
    pub fn page_size(&self) -> usize {
        self.page_size
    }
}

/// A page builder for incrementally constructing pages.
pub struct PageBuilder {
    page: Page,
    write_offset: usize,
}

impl PageBuilder {
    /// Create a new page builder.
    pub fn new(page_id: u32, page_type: PageType, page_size: usize) -> Self {
        todo!()
    }

    /// Append data to the page. Returns the offset where data was written.
    pub fn append(&mut self, data: &[u8]) -> Result<usize, String> {
        todo!()
    }

    /// Remaining space in the page.
    pub fn remaining(&self) -> usize {
        todo!()
    }

    /// Finalize the page, computing the checksum.
    pub fn finish(self) -> Page {
        todo!()
    }
}
