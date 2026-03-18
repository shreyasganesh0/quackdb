//! Lesson 06: Dictionary Encoding
//!
//! Compress low-cardinality columns by replacing repeated values with small
//! integer codes that index into a dictionary of unique values. Effective when
//! the number of distinct values is much smaller than the total row count
//! (e.g., country names, status codes).
//!
//! Key Rust concepts: `HashMap` for value-to-code lookup, generic bounds
//! (`Clone + Eq + Hash`), and `Option<T>` for nullable values.

use std::collections::HashMap;
use std::hash::Hash;

/// A dictionary mapping unique values to integer codes.
///
/// Values are stored in insertion order in `code_to_value` (a `Vec`), while
/// `value_to_code` provides O(1) reverse lookup.
// Hint: the generic bounds `Clone + Eq + Hash` are needed because values
// are stored in a HashMap (requires Eq + Hash) and cloned into the Vec.
#[derive(Debug, Clone)]
pub struct Dictionary<T: Clone + Eq + Hash> {
    /// Maps values to their dictionary codes.
    value_to_code: HashMap<T, u32>,
    /// Maps codes back to values (index = code).
    code_to_value: Vec<T>,
}

impl<T: Clone + Eq + Hash> Dictionary<T> {
    /// Create a new empty dictionary.
    pub fn new() -> Self {
        todo!()
    }

    /// Insert a value into the dictionary, returning its code.
    ///
    /// If the value already exists, returns the existing code without
    /// adding a duplicate.
    pub fn insert(&mut self, value: T) -> u32 {
        todo!()
    }

    /// Look up the code for a value. Returns `None` if not present.
    pub fn get_code(&self, value: &T) -> Option<u32> {
        todo!()
    }

    /// Look up the value for a code. Returns `None` if out of range.
    pub fn get_value(&self, code: u32) -> Option<&T> {
        todo!()
    }

    /// Number of distinct values in the dictionary.
    pub fn cardinality(&self) -> usize {
        todo!()
    }
}

/// Dictionary-encoded column data.
///
/// The `codes` vector has one entry per row; each entry is an index into
/// `dictionary`. The `nulls` vector tracks which positions are null.
#[derive(Debug, Clone)]
pub struct DictionaryEncoded<T: Clone + Eq + Hash> {
    pub dictionary: Dictionary<T>,
    pub codes: Vec<u32>,
    /// Null bitmap: `true` = null at that position.
    pub nulls: Vec<bool>,
}

/// Encode a slice of optional values using dictionary encoding.
///
/// `None` entries become null; `Some(v)` entries are added to the dictionary.
// Hint: iterate through data, inserting non-null values into the dictionary
// and recording their codes. Use a sentinel code (e.g., u32::MAX) for nulls.
pub fn encode<T: Clone + Eq + Hash>(data: &[Option<T>]) -> DictionaryEncoded<T> {
    todo!()
}

/// Decode dictionary-encoded data back to optional values.
// Hint: iterate through codes and nulls; for non-null entries, look up the
// value in the dictionary via `get_value(code)`.
pub fn decode<T: Clone + Eq + Hash>(encoded: &DictionaryEncoded<T>) -> Vec<Option<T>> {
    todo!()
}

/// Check if dictionary encoding is beneficial for the given data.
///
/// Returns `true` if the distinct-value ratio (cardinality / total) is below
/// the given `threshold` (e.g., 0.5 means fewer than 50% unique values).
pub fn should_dictionary_encode<T: Clone + Eq + Hash>(data: &[T], threshold: f64) -> bool {
    todo!()
}

/// Calculate the compression ratio for dictionary encoding.
///
/// Compares the original size (`original_count * original_value_size`) to the
/// encoded size (dictionary entries + code array).
pub fn compression_ratio<T: Clone + Eq + Hash>(original_count: usize, original_value_size: usize, encoded: &DictionaryEncoded<T>) -> f64 {
    todo!()
}
