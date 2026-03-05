//! Lesson 06: Dictionary Encoding
//!
//! Dictionary compression for low-cardinality columns.

use std::collections::HashMap;
use std::hash::Hash;

/// A dictionary mapping unique values to integer codes.
#[derive(Debug, Clone)]
pub struct Dictionary<T: Clone + Eq + Hash> {
    /// Maps values to their dictionary codes.
    value_to_code: HashMap<T, u32>,
    /// Maps codes back to values.
    code_to_value: Vec<T>,
}

impl<T: Clone + Eq + Hash> Dictionary<T> {
    /// Create a new empty dictionary.
    pub fn new() -> Self {
        todo!()
    }

    /// Insert a value into the dictionary. Returns the code.
    pub fn insert(&mut self, value: T) -> u32 {
        todo!()
    }

    /// Look up the code for a value.
    pub fn get_code(&self, value: &T) -> Option<u32> {
        todo!()
    }

    /// Look up the value for a code.
    pub fn get_value(&self, code: u32) -> Option<&T> {
        todo!()
    }

    /// Number of distinct values in the dictionary.
    pub fn cardinality(&self) -> usize {
        todo!()
    }
}

/// Dictionary-encoded column data.
#[derive(Debug, Clone)]
pub struct DictionaryEncoded<T: Clone + Eq + Hash> {
    pub dictionary: Dictionary<T>,
    pub codes: Vec<u32>,
    /// Null bitmap: true = null at that position.
    pub nulls: Vec<bool>,
}

/// Encode a slice of optional values using dictionary encoding.
pub fn encode<T: Clone + Eq + Hash>(data: &[Option<T>]) -> DictionaryEncoded<T> {
    todo!()
}

/// Decode dictionary-encoded data back to optional values.
pub fn decode<T: Clone + Eq + Hash>(encoded: &DictionaryEncoded<T>) -> Vec<Option<T>> {
    todo!()
}

/// Check if dictionary encoding is beneficial (cardinality below threshold).
pub fn should_dictionary_encode<T: Clone + Eq + Hash>(data: &[T], threshold: f64) -> bool {
    todo!()
}

/// Calculate the compression ratio for dictionary encoding.
pub fn compression_ratio<T: Clone + Eq + Hash>(original_count: usize, original_value_size: usize, encoded: &DictionaryEncoded<T>) -> f64 {
    todo!()
}
