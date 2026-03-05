//! Lesson 06: Dictionary Encoding Tests

use quackdb::compression::dictionary;
use quackdb::compression::dictionary::Dictionary;

#[test]
fn test_dictionary_basic() {
    let mut dict = Dictionary::<String>::new();
    let c1 = dict.insert("hello".to_string());
    let c2 = dict.insert("world".to_string());
    let c3 = dict.insert("hello".to_string());

    assert_eq!(c1, c3, "Same value should get same code");
    assert_ne!(c1, c2);
    assert_eq!(dict.cardinality(), 2);
}

#[test]
fn test_dictionary_lookup() {
    let mut dict = Dictionary::<i32>::new();
    dict.insert(10);
    dict.insert(20);
    dict.insert(30);

    assert_eq!(dict.get_code(&10), Some(0));
    assert_eq!(dict.get_code(&20), Some(1));
    assert_eq!(dict.get_code(&30), Some(2));
    assert_eq!(dict.get_code(&40), None);

    assert_eq!(dict.get_value(0), Some(&10));
    assert_eq!(dict.get_value(1), Some(&20));
    assert_eq!(dict.get_value(2), Some(&30));
    assert_eq!(dict.get_value(3), None);
}

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

    assert_eq!(encoded.dictionary.cardinality(), 3);
    assert_eq!(encoded.codes.len(), 5);
    // Same values should have same codes
    assert_eq!(encoded.codes[0], encoded.codes[2]); // apple
    assert_eq!(encoded.codes[1], encoded.codes[4]); // banana
}

#[test]
fn test_dictionary_roundtrip() {
    let data: Vec<Option<i32>> = vec![
        Some(10), Some(20), Some(10), Some(30), Some(20), Some(10),
    ];
    let encoded = dictionary::encode(&data);
    let decoded = dictionary::decode(&encoded);
    assert_eq!(decoded, data);
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
    let encoded = dictionary::encode(&data);
    let decoded = dictionary::decode(&encoded);
    assert_eq!(decoded, data);
}

#[test]
fn test_dictionary_all_nulls() {
    let data: Vec<Option<i32>> = vec![None, None, None];
    let encoded = dictionary::encode(&data);
    let decoded = dictionary::decode(&encoded);
    assert_eq!(decoded, data);
}

#[test]
fn test_dictionary_high_cardinality() {
    let data: Vec<i32> = (0..1000).collect();
    // With unique values, dictionary encoding is not beneficial
    assert!(!dictionary::should_dictionary_encode(&data, 0.5));
}

#[test]
fn test_dictionary_low_cardinality() {
    let data: Vec<i32> = (0..1000).map(|i| i % 5).collect();
    // 5 distinct values out of 1000 — very low cardinality
    assert!(dictionary::should_dictionary_encode(&data, 0.1));
}

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

#[test]
fn test_dictionary_empty() {
    let data: Vec<Option<i32>> = vec![];
    let encoded = dictionary::encode(&data);
    assert_eq!(encoded.dictionary.cardinality(), 0);
    assert_eq!(encoded.codes.len(), 0);
    let decoded = dictionary::decode(&encoded);
    assert!(decoded.is_empty());
}
