//! Lesson 05: Run-Length Encoding Tests

use quackdb::compression::rle;

#[test]
fn test_rle_all_same() {
    let data = vec![42i32; 1000];
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 1);
    assert_eq!(encoded.runs[0].value, 42);
    assert_eq!(encoded.runs[0].count, 1000);
    assert_eq!(encoded.total_count, 1000);

    let decoded = rle::decode(&encoded);
    assert_eq!(decoded, data);
}

#[test]
fn test_rle_alternating() {
    let data: Vec<i32> = (0..100).map(|i| i % 2).collect();
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 100);
    let decoded = rle::decode(&encoded);
    assert_eq!(decoded, data);
}

#[test]
fn test_rle_sorted() {
    let mut data = Vec::new();
    for i in 0..10 {
        for _ in 0..50 {
            data.push(i);
        }
    }
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 10);
    let decoded = rle::decode(&encoded);
    assert_eq!(decoded, data);
}

#[test]
fn test_rle_single_element() {
    let data = vec![99i32];
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 1);
    let decoded = rle::decode(&encoded);
    assert_eq!(decoded, data);
}

#[test]
fn test_rle_empty() {
    let data: Vec<i32> = vec![];
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 0);
    assert_eq!(encoded.total_count, 0);
    let decoded = rle::decode(&encoded);
    assert!(decoded.is_empty());
}

#[test]
fn test_rle_random_access() {
    let mut data = Vec::new();
    for i in 0..5 {
        for _ in 0..100 {
            data.push(i as i32);
        }
    }
    let encoded = rle::encode(&data);

    // Check random access at various positions
    assert_eq!(rle::get_at_index(&encoded, 0), 0);
    assert_eq!(rle::get_at_index(&encoded, 99), 0);
    assert_eq!(rle::get_at_index(&encoded, 100), 1);
    assert_eq!(rle::get_at_index(&encoded, 250), 2);
    assert_eq!(rle::get_at_index(&encoded, 499), 4);
}

#[test]
fn test_rle_roundtrip_strings() {
    let data = vec!["hello", "hello", "hello", "world", "world", "foo"];
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 3);
    let decoded = rle::decode(&encoded);
    assert_eq!(decoded, data);
}

#[test]
fn test_rle_compression_ratio() {
    let data = vec![1i32; 10000];
    let encoded = rle::encode(&data);
    let ratio = rle::compression_ratio(data.len(), &encoded);
    assert!(ratio > 10.0, "Expected compression ratio > 10x, got {}", ratio);
}

#[test]
fn test_rle_bytes() {
    let data = vec![0xAA_u8; 500];
    let encoded = rle::encode_bytes(&data);
    assert!(encoded.len() < data.len(), "Encoded should be smaller");
    let decoded = rle::decode_bytes(&encoded);
    assert_eq!(decoded, data);
}

#[test]
fn test_rle_bytes_mixed() {
    let data: Vec<u8> = (0..256).map(|i| i as u8).collect();
    let encoded = rle::encode_bytes(&data);
    let decoded = rle::decode_bytes(&encoded);
    assert_eq!(decoded, data);
}
