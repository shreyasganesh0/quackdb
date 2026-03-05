//! Lesson 09: Pages & Page Layout Tests

use quackdb::storage::page::{Page, PageBuilder, PageHeader, PageType, DEFAULT_PAGE_SIZE};

#[test]
fn test_page_creation() {
    let page = Page::new(0, PageType::Data, DEFAULT_PAGE_SIZE);
    assert_eq!(page.header.page_id, 0);
    assert_eq!(page.header.page_type, PageType::Data);
    assert_eq!(page.page_size(), DEFAULT_PAGE_SIZE);
}

#[test]
fn test_page_write_read() {
    let mut page = Page::new_default(1, PageType::Data);
    let data = b"hello world";
    page.write_data(0, data).unwrap();
    let read_back = page.read_data(0, data.len()).unwrap();
    assert_eq!(read_back, data);
}

#[test]
fn test_page_free_space() {
    let mut page = Page::new_default(0, PageType::Data);
    let initial_free = page.free_space();
    assert!(initial_free > 0);

    page.write_data(0, &[0u8; 100]).unwrap();
    // Free space tracking depends on implementation
}

#[test]
fn test_page_checksum() {
    let mut page = Page::new_default(0, PageType::Data);
    page.write_data(0, b"test data for checksum").unwrap();
    page.update_checksum();

    assert!(page.verify_checksum(), "Checksum should verify after update");

    // Corrupt data
    let checksum_before = page.header.checksum;
    page.data[0] ^= 0xFF;
    assert!(!page.verify_checksum(), "Checksum should fail after corruption");
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
    assert_eq!(bytes.len(), PageHeader::SIZE);
    let restored = PageHeader::from_bytes(&bytes).unwrap();
    assert_eq!(restored.page_type, PageType::Meta);
    assert_eq!(restored.page_id, 123);
    assert_eq!(restored.checksum, 0xDEADBEEF);
    assert_eq!(restored.free_space, 4096);
    assert_eq!(restored.num_records, 10);
}

#[test]
fn test_page_builder() {
    let mut builder = PageBuilder::new(0, PageType::Data, DEFAULT_PAGE_SIZE);
    let offset1 = builder.append(b"first record").unwrap();
    let offset2 = builder.append(b"second record").unwrap();
    assert!(offset2 > offset1);
    assert!(builder.remaining() < DEFAULT_PAGE_SIZE);

    let page = builder.finish();
    assert!(page.verify_checksum());
}

#[test]
fn test_page_builder_overflow() {
    let mut builder = PageBuilder::new(0, PageType::Data, 64);
    builder.append(&[0u8; 30]).unwrap();
    // This should fail — not enough space
    let result = builder.append(&[0u8; 40]);
    assert!(result.is_err());
}

#[test]
fn test_page_type_from_u8() {
    assert_eq!(PageType::from_u8(1), Some(PageType::Data));
    assert_eq!(PageType::from_u8(2), Some(PageType::Index));
    assert_eq!(PageType::from_u8(3), Some(PageType::Overflow));
    assert_eq!(PageType::from_u8(4), Some(PageType::Meta));
    assert_eq!(PageType::from_u8(0), None);
    assert_eq!(PageType::from_u8(255), None);
}

#[test]
fn test_page_boundary_write() {
    let mut page = Page::new(0, PageType::Data, 128);
    // Write up to the boundary
    let max_data = page.page_size() - PageHeader::SIZE;
    let result = page.write_data(0, &vec![0xAB; max_data]);
    assert!(result.is_ok());
}
