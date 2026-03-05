//! Lesson 03: Columnar Vectors
//!
//! Column-oriented data vectors with validity masks for null handling.

use crate::types::{LogicalType, PhysicalType, ScalarValue};

/// A bitmask tracking which values in a vector are valid (non-null).
#[derive(Debug, Clone)]
pub struct ValidityMask {
    bits: Vec<u64>,
    count: usize,
}

impl ValidityMask {
    /// Create a new validity mask with all values valid.
    pub fn new_all_valid(count: usize) -> Self {
        todo!()
    }

    /// Create a new validity mask with all values invalid (null).
    pub fn new_all_invalid(count: usize) -> Self {
        todo!()
    }

    /// Check if the value at the given index is valid (non-null).
    pub fn is_valid(&self, index: usize) -> bool {
        todo!()
    }

    /// Set the validity of the value at the given index.
    pub fn set_valid(&mut self, index: usize, valid: bool) {
        todo!()
    }

    /// Set a range of values as valid.
    pub fn set_valid_range(&mut self, start: usize, count: usize) {
        todo!()
    }

    /// Check if all values are valid (no nulls).
    pub fn all_valid(&self) -> bool {
        todo!()
    }

    /// Count the number of valid (non-null) values.
    pub fn count_valid(&self) -> usize {
        todo!()
    }

    /// Count of entries tracked by this mask.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Resize the mask to hold `new_count` entries, new entries are valid.
    pub fn resize(&mut self, new_count: usize) {
        todo!()
    }
}

/// Describes how a vector stores its data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorType {
    /// Standard flat vector with one value per row.
    Flat,
    /// Constant vector: all rows share one value.
    Constant,
    /// Dictionary vector: indices into a dictionary.
    Dictionary,
}

/// A selection vector: indices into another vector, used for filtered operations.
#[derive(Debug, Clone)]
pub struct SelectionVector {
    indices: Vec<u32>,
}

impl SelectionVector {
    /// Create a new selection vector from indices.
    pub fn new(indices: Vec<u32>) -> Self {
        Self { indices }
    }

    /// Get the index at position `i`.
    pub fn get(&self, i: usize) -> u32 {
        self.indices[i]
    }

    /// Number of selected indices.
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Get the underlying indices.
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Create an incrementing selection vector [0, 1, 2, ..., count-1].
    pub fn incrementing(count: usize) -> Self {
        todo!()
    }
}

/// A columnar data vector — the fundamental data container.
pub struct Vector {
    logical_type: LogicalType,
    vector_type: VectorType,
    data: Vec<u8>,
    validity: ValidityMask,
    count: usize,
    // For variable-length types (Varchar): offsets into the data buffer
    offsets: Option<Vec<u32>>,
}

impl Vector {
    /// Create a new flat vector of the given type with capacity for `count` values.
    pub fn new(logical_type: LogicalType, count: usize) -> Self {
        todo!()
    }

    /// Create a constant vector that repeats a single value.
    pub fn new_constant(value: ScalarValue, count: usize) -> Self {
        todo!()
    }

    /// Get the logical type of this vector.
    pub fn logical_type(&self) -> &LogicalType {
        &self.logical_type
    }

    /// Get the vector type (Flat, Constant, Dictionary).
    pub fn vector_type(&self) -> VectorType {
        self.vector_type
    }

    /// Number of values in this vector.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Set the count of values.
    pub fn set_count(&mut self, count: usize) {
        self.count = count;
    }

    /// Get a reference to the validity mask.
    pub fn validity(&self) -> &ValidityMask {
        &self.validity
    }

    /// Get a mutable reference to the validity mask.
    pub fn validity_mut(&mut self) -> &mut ValidityMask {
        &mut self.validity
    }

    /// Get the raw data buffer.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable reference to the raw data buffer.
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Get a typed value at the given index.
    pub fn get_value(&self, index: usize) -> ScalarValue {
        todo!()
    }

    /// Set a typed value at the given index.
    pub fn set_value(&mut self, index: usize, value: ScalarValue) {
        todo!()
    }

    /// Set a null at the given index.
    pub fn set_null(&mut self, index: usize) {
        todo!()
    }

    /// Flatten a constant vector into a flat vector.
    pub fn flatten(&mut self) {
        todo!()
    }

    /// Copy values from this vector according to a selection vector into a target.
    pub fn copy_with_selection(&self, sel: &SelectionVector, target: &mut Vector) {
        todo!()
    }

    /// Append a string value (for Varchar vectors).
    pub fn append_string(&mut self, s: &str) {
        todo!()
    }

    /// Get a string value at index (for Varchar vectors).
    pub fn get_string(&self, index: usize) -> Option<&str> {
        todo!()
    }

    /// Get a typed slice of the data buffer.
    pub fn get_data_slice<T: Copy>(&self) -> &[T] {
        todo!()
    }

    /// Get a mutable typed slice of the data buffer.
    pub fn get_data_slice_mut<T: Copy>(&mut self) -> &mut [T] {
        todo!()
    }

    /// Reference to offsets (for Varchar).
    pub fn offsets(&self) -> Option<&[u32]> {
        self.offsets.as_deref()
    }
}
