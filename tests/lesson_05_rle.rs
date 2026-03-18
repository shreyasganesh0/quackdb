//! # Lesson 05: Run-Length Encoding — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Basic encoding of constant data (`test_rle_all_same`)
//! 2. Edge cases (empty input, single element)
//! 3. Core algorithm correctness (`test_rle_sorted`, `test_rle_alternating`)
//! 4. Random access into encoded data (`test_rle_random_access`)
//! 5. Generic type support (`test_rle_roundtrip_strings`)
//! 6. Compression metrics (`test_rle_compression_ratio`)
//! 7. Byte-level RLE (`test_rle_bytes`, `test_rle_bytes_mixed`)

use quackdb::compression::rle;

/// Helper: create a Vec<i32> of repeated groups, e.g. `make_repeated_groups(3, 50)`
/// produces [0,0,...(50 times), 1,1,...(50 times), 2,2,...(50 times)].
/// Useful for constructing sorted/grouped data patterns for RLE tests.
fn make_repeated_groups(num_groups: i32, elements_per_group: usize) -> Vec<i32> {
    let mut data = Vec::new();
    for i in 0..num_groups {
        for _ in 0..elements_per_group {
            data.push(i);
        }
    }
    data
}

/// Helper: encode then decode data and assert the roundtrip is lossless.
fn assert_rle_roundtrip<T: Clone + PartialEq + std::fmt::Debug>(data: &[T]) {
    let encoded = rle::encode(data);
    let decoded = rle::decode(&encoded);
    assert_eq!(decoded, data, "RLE roundtrip must be lossless");
}

// ── 1. Basic encoding of constant data ──────────────────────────────

#[test]
fn test_rle_all_same() {
    let data = vec![42i32; 1000];
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 1, "all-same input should compress to a single run");
    assert_eq!(encoded.runs[0].value, 42);
    assert_eq!(encoded.runs[0].count, 1000, "the single run must capture the total element count");
    assert_eq!(encoded.total_count, 1000);

    let decoded = rle::decode(&encoded);
    assert_eq!(decoded, data);
}

// ── 2. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_rle_empty() {
    let data: Vec<i32> = vec![];
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 0, "empty input must produce zero runs");
    assert_eq!(encoded.total_count, 0);
    let decoded = rle::decode(&encoded);
    assert!(decoded.is_empty());
}

#[test]
fn test_rle_single_element() {
    let data = vec![99i32];
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 1, "single element should produce exactly one run");
    assert_eq!(encoded.runs[0].count, 1, "single-element run must have count 1");
    let decoded = rle::decode(&encoded);
    assert_eq!(decoded, data);
}

#[test]
fn test_rle_two_distinct_elements() {
    // Edge case: two elements that differ — the minimum input to produce two runs
    let data = vec![1i32, 2];
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 2, "two distinct adjacent values must produce two runs");
    assert_rle_roundtrip(&data);
}

// ── 3. Core algorithm correctness ───────────────────────────────────

#[test]
fn test_rle_sorted() {
    let data = make_repeated_groups(10, 50);
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 10, "sorted data groups into one run per distinct value");
    assert_rle_roundtrip(&data);
}

#[test]
fn test_rle_alternating() {
    let data: Vec<i32> = (0..100).map(|i| i % 2).collect();
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 100, "alternating values produce no consecutive duplicates, so every element is its own run");
    let decoded = rle::decode(&encoded);
    assert_eq!(decoded, data, "RLE must be lossless even in the worst case");
}

// ── 4. Random access ────────────────────────────────────────────────

#[test]
fn test_rle_random_access() {
    let data = make_repeated_groups(5, 100);
    let encoded = rle::encode(&data);

    // Check random access at various positions
    assert_eq!(rle::get_at_index(&encoded, 0), 0, "index 0 falls in the first run");
    assert_eq!(rle::get_at_index(&encoded, 99), 0, "last element of the first run");
    assert_eq!(rle::get_at_index(&encoded, 100), 1, "run boundary: index 100 starts a new run");
    assert_eq!(rle::get_at_index(&encoded, 250), 2);
    assert_eq!(rle::get_at_index(&encoded, 499), 4, "last valid index should map to the final run");
}

// ── 5. Generic type support ─────────────────────────────────────────

#[test]
fn test_rle_roundtrip_strings() {
    let data = vec!["hello", "hello", "hello", "world", "world", "foo"];
    let encoded = rle::encode(&data);
    assert_eq!(encoded.runs.len(), 3, "RLE should be generic over string-like types too");
    assert_rle_roundtrip(&data);
}

// ── 6. Compression metrics ──────────────────────────────────────────

#[test]
fn test_rle_compression_ratio() {
    let data = vec![1i32; 10000];
    let encoded = rle::encode(&data);
    let ratio = rle::compression_ratio(data.len(), &encoded);
    assert!(ratio > 10.0, "Expected compression ratio > 10x, got {}", ratio);
}

// ── 7. Byte-level RLE ───────────────────────────────────────────────

#[test]
fn test_rle_bytes() {
    let data = vec![0xAA_u8; 500];
    let encoded = rle::encode_bytes(&data);
    assert!(encoded.len() < data.len(), "byte-level RLE of constant data must shrink the payload");
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

#[test]
fn test_rle_bytes_empty() {
    // Edge case: empty byte input
    let data: Vec<u8> = vec![];
    let encoded = rle::encode_bytes(&data);
    let decoded = rle::decode_bytes(&encoded);
    assert!(decoded.is_empty(), "empty byte-level RLE must round-trip to empty");
}
