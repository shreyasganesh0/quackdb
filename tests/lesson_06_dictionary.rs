//! # Lesson 06: Dictionary Encoding — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Basic dictionary construction (`test_dictionary_basic`)
//! 2. Dictionary lookup by code and value (`test_dictionary_lookup`)
//! 3. Edge cases (empty input, all nulls, single element)
//! 4. Encode/decode roundtrips (`test_dictionary_roundtrip`, `test_dictionary_with_nulls`)
//! 5. Cardinality analysis (`test_dictionary_high_cardinality`, `test_dictionary_low_cardinality`)
//! 6. Compression metrics (`test_dictionary_compression_ratio`)

use quackdb::compression::dictionary;
use quackdb::compression::dictionary::Dictionary;

/// Helper: dictionary-encode data and verify the roundtrip is lossless.
fn assert_dict_roundtrip<T>(data: &[Option<T>])
where
    T: Clone + Eq + std::fmt::Debug + std::hash::Hash,
{
    let encoded = dictionary::encode(data);
    let decoded = dictionary::decode(&encoded);
    assert_eq!(decoded, data, "dictionary encode/decode roundtrip must be lossless");
}

// ── 1. Basic dictionary construction ────────────────────────────────

#[test]
fn test_dictionary_basic() {
    let mut dict = Dictionary::<String>::new();
    let c1 = dict.insert("hello".to_string());
    let c2 = dict.insert("world".to_string());
    let c3 = dict.insert("hello".to_string());

    assert_eq!(c1, c3, "Same value should get same code");
    assert_ne!(c1, c2, "distinct values must receive different codes");
    assert_eq!(dict.cardinality(), 2, "cardinality tracks the number of unique entries");
}

// ── 2. Dictionary lookup ────────────────────────────────────────────

#[test]
fn test_dictionary_lookup() {
    let mut dict = Dictionary::<i32>::new();
    dict.insert(10);
    dict.insert(20);
    dict.insert(30);

    assert_eq!(dict.get_code(&10), Some(0), "codes are assigned in insertion order starting at 0");
    assert_eq!(dict.get_code(&20), Some(1));
    assert_eq!(dict.get_code(&30), Some(2));
    assert_eq!(dict.get_code(&40), None, "absent values must return None, not a sentinel");

    assert_eq!(dict.get_value(0), Some(&10), "reverse lookup: code -> value must invert the mapping");
    assert_eq!(dict.get_value(1), Some(&20));
    assert_eq!(dict.get_value(2), Some(&30));
    assert_eq!(dict.get_value(3), None, "out-of-range code must return None");
}

// ── 3. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_dictionary_empty() {
    let data: Vec<Option<i32>> = vec![];
    let encoded = dictionary::encode(&data);
    assert_eq!(encoded.dictionary.cardinality(), 0, "empty input produces an empty dictionary");
    assert_eq!(encoded.codes.len(), 0);
    let decoded = dictionary::decode(&encoded);
    assert!(decoded.is_empty());
}

#[test]
fn test_dictionary_all_nulls() {
    let data: Vec<Option<i32>> = vec![None, None, None];
    assert_dict_roundtrip(&data);
}

#[test]
fn test_dictionary_single_element() {
    // Edge case: a single non-null value
    let data: Vec<Option<i32>> = vec![Some(42)];
    let encoded = dictionary::encode(&data);
    assert_eq!(encoded.dictionary.cardinality(), 1, "single element must produce a dictionary with cardinality 1");
    assert_eq!(encoded.codes.len(), 1);
    assert_dict_roundtrip(&data);
}

#[test]
fn test_dictionary_single_null() {
    // Edge case: a single null value
    let data: Vec<Option<i32>> = vec![None];
    assert_dict_roundtrip(&data);
}

// ── 4. Encode/decode roundtrips ─────────────────────────────────────

#[test]
fn test_dictionary_encode_strings() {
    let data: Vec<Option<String>> = vec![
        Some("apple".into()),
        Some("banana".into()),
        Some("apple".into()),
        Some("cherry".into()),
        Some("banana".into()),
    ];
    let encoded = dictionary::encode(&data);

    assert_eq!(encoded.dictionary.cardinality(), 3, "three distinct fruits yield cardinality 3");
    assert_eq!(encoded.codes.len(), 5, "one code per input element regardless of duplicates");
    // Same values should have same codes
    assert_eq!(encoded.codes[0], encoded.codes[2], "duplicate 'apple' entries must share the same code");
    assert_eq!(encoded.codes[1], encoded.codes[4]); // banana
}

#[test]
fn test_dictionary_roundtrip() {
    let data: Vec<Option<i32>> = vec![
        Some(10), Some(20), Some(10), Some(30), Some(20), Some(10),
    ];
    assert_dict_roundtrip(&data);
}

#[test]
fn test_dictionary_with_nulls() {
    let data: Vec<Option<String>> = vec![
        Some("a".into()),
        None,
        Some("b".into()),
        None,
        Some("a".into()),
    ];
    assert_dict_roundtrip(&data);
}

// ── 5. Cardinality analysis ─────────────────────────────────────────

#[test]
fn test_dictionary_high_cardinality() {
    let data: Vec<i32> = (0..1000).collect();
    // With unique values, dictionary encoding is not beneficial
    assert!(!dictionary::should_dictionary_encode(&data, 0.5), "dictionary encoding hurts when cardinality equals row count");
}

#[test]
fn test_dictionary_low_cardinality() {
    let data: Vec<i32> = (0..1000).map(|i| i % 5).collect();
    // 5 distinct values out of 1000 — very low cardinality
    assert!(dictionary::should_dictionary_encode(&data, 0.1), "5 distinct values in 1000 rows is ideal for dictionary encoding");
}

// ── 6. Compression metrics ──────────────────────────────────────────

#[test]
fn test_dictionary_compression_ratio() {
    let data: Vec<Option<String>> = (0..10000)
        .map(|i| Some(format!("category_{}", i % 10)))
        .collect();
    let encoded = dictionary::encode(&data);
    let original_size = data.len() * std::mem::size_of::<String>(); // approximate
    let ratio = dictionary::compression_ratio(data.len(), std::mem::size_of::<String>(), &encoded);
    assert!(ratio > 1.0, "Dictionary encoding should compress low-cardinality data");
}
