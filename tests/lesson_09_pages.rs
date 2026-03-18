//! # Lesson 09: Pages & Page Layout — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Basic page creation (`test_page_creation`)
//! 2. Page type discriminant (`test_page_type_from_u8`)
//! 3. Write and read operations (`test_page_write_read`, `test_page_free_space`)
//! 4. Edge cases (empty write, boundary writes)
//! 5. Integrity — checksum (`test_page_checksum`)
//! 6. Serialization roundtrip (`test_page_header_roundtrip`, `test_page_serialize_roundtrip`)
//! 7. PageBuilder — append-based construction (`test_page_builder`, `test_page_builder_overflow`)

use quackdb::storage::page::{Page, PageBuilder, PageHeader, PageType, DEFAULT_PAGE_SIZE};

// ── 1. Basic page creation ──────────────────────────────────────────

#[test]
fn test_page_creation() {
    let page = Page::new(0, PageType::Data, DEFAULT_PAGE_SIZE);
    assert_eq!(page.header.page_id, 0, "page ID assigned at creation must be stored in the header");
    assert_eq!(page.header.page_type, PageType::Data);
    assert_eq!(page.page_size(), DEFAULT_PAGE_SIZE, "page size defaults to the system page size constant");
}

// ── 2. Page type discriminant ───────────────────────────────────────

#[test]
fn test_page_type_from_u8() {
    assert_eq!(PageType::from_u8(1), Some(PageType::Data));
    assert_eq!(PageType::from_u8(2), Some(PageType::Index));
    assert_eq!(PageType::from_u8(3), Some(PageType::Overflow));
    assert_eq!(PageType::from_u8(4), Some(PageType::Meta));
    assert_eq!(PageType::from_u8(0), None, "invalid type discriminants must return None to catch corruption");
    assert_eq!(PageType::from_u8(255), None);
}

// ── 3. Write and read operations ────────────────────────────────────

#[test]
fn test_page_write_read() {
    let mut page = Page::new_default(1, PageType::Data);
    let data = b"hello world";
    page.write_data(0, data).unwrap();
    let read_back = page.read_data(0, data.len()).unwrap();
    assert_eq!(read_back, data, "page must faithfully store and return written bytes");
}

#[test]
fn test_page_free_space() {
    let mut page = Page::new_default(0, PageType::Data);
    let initial_free = page.free_space();
    assert!(initial_free > 0, "a fresh page must have usable free space after the header");

    page.write_data(0, &[0u8; 100]).unwrap();
    // Free space tracking depends on implementation
}

// ── 4. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_page_write_empty() {
    // Edge case: writing zero bytes should succeed without error
    let mut page = Page::new_default(0, PageType::Data);
    let result = page.write_data(0, &[]);
    assert!(result.is_ok(), "writing zero bytes must succeed");
}

#[test]
fn test_page_boundary_write() {
    let mut page = Page::new(0, PageType::Data, 128);
    // Write up to the boundary
    let max_data = page.page_size() - PageHeader::SIZE;
    let result = page.write_data(0, &vec![0xAB; max_data]);
    assert!(result.is_ok());
}

#[test]
fn test_page_single_byte_write() {
    // Edge case: writing and reading a single byte
    let mut page = Page::new_default(0, PageType::Data);
    page.write_data(0, &[0xFF]).unwrap();
    let read_back = page.read_data(0, 1).unwrap();
    assert_eq!(read_back, &[0xFF], "single-byte write must be retrievable");
}

// ── 5. Integrity — checksum ────────────────────────────────────────

#[test]
fn test_page_checksum() {
    let mut page = Page::new_default(0, PageType::Data);
    page.write_data(0, b"test data for checksum").unwrap();
    page.update_checksum();

    assert!(page.verify_checksum(), "Checksum should verify after update");

    // Corrupt data
    let _checksum_before = page.header.checksum;
    page.data[0] ^= 0xFF;
    assert!(!page.verify_checksum(), "Checksum should fail after corruption");
}

// ── 6. Serialization roundtrip ──────────────────────────────────────

#[test]
fn test_page_header_roundtrip() {
    let header = PageHeader {
        page_type: PageType::Meta,
        page_id: 123,
        checksum: 0xDEADBEEF,
        free_space: 4096,
        num_records: 10,
    };
    let bytes = header.to_bytes();
    assert_eq!(bytes.len(), PageHeader::SIZE, "header serialization must produce a fixed-size byte array");
    let restored = PageHeader::from_bytes(&bytes).unwrap();
    assert_eq!(restored.page_type, PageType::Meta);
    assert_eq!(restored.page_id, 123);
    assert_eq!(restored.checksum, 0xDEADBEEF, "checksum must be preserved exactly through serialization");
    assert_eq!(restored.free_space, 4096);
    assert_eq!(restored.num_records, 10);
}

#[test]
fn test_page_serialize_roundtrip() {
    let mut page = Page::new_default(42, PageType::Index);
    page.write_data(0, b"serialization test data").unwrap();
    page.update_checksum();

    let bytes = page.to_bytes();
    let restored = Page::from_bytes(&bytes, DEFAULT_PAGE_SIZE).unwrap();

    assert_eq!(restored.header.page_id, 42);
    assert_eq!(restored.header.page_type, PageType::Index);
    assert!(restored.verify_checksum());
    let original_data = page.read_data(0, 23).unwrap();
    let restored_data = restored.read_data(0, 23).unwrap();
    assert_eq!(original_data, restored_data);
}

// ── 7. PageBuilder ──────────────────────────────────────────────────

#[test]
fn test_page_builder() {
    let mut builder = PageBuilder::new(0, PageType::Data, DEFAULT_PAGE_SIZE);
    let offset1 = builder.append(b"first record").unwrap();
    let offset2 = builder.append(b"second record").unwrap();
    assert!(offset2 > offset1, "each append must advance the write offset");
    assert!(builder.remaining() < DEFAULT_PAGE_SIZE, "remaining space must decrease after appending records");

    let page = builder.finish();
    assert!(page.verify_checksum());
}

#[test]
fn test_page_builder_overflow() {
    let mut builder = PageBuilder::new(0, PageType::Data, 64);
    builder.append(&[0u8; 30]).unwrap();
    // This should fail — not enough space
    let result = builder.append(&[0u8; 40]);
    assert!(result.is_err(), "writing beyond page capacity must return an error, not corrupt data");
}

#[test]
fn test_page_builder_empty_record() {
    // Edge case: appending an empty record
    let mut builder = PageBuilder::new(0, PageType::Data, DEFAULT_PAGE_SIZE);
    let result = builder.append(&[]);
    assert!(result.is_ok(), "appending an empty record should succeed");
}
