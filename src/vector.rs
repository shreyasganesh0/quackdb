//! Lesson 03: Columnar Vectors
//!
//! Column-oriented data vectors with validity masks for null handling.
//! Vectors are the fundamental unit of columnar storage: a contiguous array
//! of values of a single type, plus a bitmask tracking which entries are null.
//!
//! Key Rust concepts: bit manipulation, `unsafe` pointer casting for typed
//! access to a raw byte buffer, generic methods with `Copy` bounds, and
//! enum-driven dispatch (Flat vs. Constant vs. Dictionary).

use crate::types::{LogicalType, PhysicalType, ScalarValue};

/// A bitmask tracking which values in a vector are valid (non-null).
///
/// Stores one bit per value in a `Vec<u64>` (64 values per word).
/// Bit = 1 means valid, bit = 0 means null.
#[derive(Debug, Clone)]
pub struct ValidityMask {
    bits: Vec<u64>,
    count: usize,
}

impl ValidityMask {
    /// Create a new validity mask with all values marked valid (all bits set).
    // Hint: compute `ceil(count / 64)` words, fill with `u64::MAX`.
    pub fn new_all_valid(count: usize) -> Self {
        let num_words = (count + 63) / 64;
        Self {
            bits: vec![u64::MAX; num_words],
            count,
        }
    }

    /// Create a new validity mask with all values marked invalid/null (all bits cleared).
    pub fn new_all_invalid(count: usize) -> Self {
        let num_words = (count + 63) / 64;
        Self {
            bits: vec![0u64; num_words],
            count,
        }
    }

    /// Check if the value at the given index is valid (non-null).
    // Hint: word index = index / 64, bit position = index % 64.
    pub fn is_valid(&self, index: usize) -> bool {
        let word = index / 64;
        let bit = index % 64;
        (self.bits[word] >> bit) & 1 == 1
    }

    /// Set the validity of the value at the given index.
    // Hint: use bitwise OR to set, AND with NOT to clear.
    pub fn set_valid(&mut self, index: usize, valid: bool) {
        let word = index / 64;
        let bit = index % 64;
        if valid {
            self.bits[word] |= 1u64 << bit;
        } else {
            self.bits[word] &= !(1u64 << bit);
        }
    }

    /// Set a range of values as valid.
    pub fn set_valid_range(&mut self, start: usize, count: usize) {
        for i in start..start + count {
            self.set_valid(i, true);
        }
    }

    /// Check if all values are valid (no nulls).
    // Hint: verify every bit is set; watch out for trailing bits in the last word.
    pub fn all_valid(&self) -> bool {
        if self.count == 0 {
            return true;
        }
        let full_words = self.count / 64;
        for i in 0..full_words {
            if self.bits[i] != u64::MAX {
                return false;
            }
        }
        let remaining = self.count % 64;
        if remaining > 0 {
            let mask = (1u64 << remaining) - 1;
            if self.bits[full_words] & mask != mask {
                return false;
            }
        }
        true
    }

    /// Count the number of valid (non-null) values.
    // Hint: sum `count_ones()` across all words, adjusting the last word.
    pub fn count_valid(&self) -> usize {
        if self.count == 0 {
            return 0;
        }
        let full_words = self.count / 64;
        let mut total: usize = 0;
        for i in 0..full_words {
            total += self.bits[i].count_ones() as usize;
        }
        let remaining = self.count % 64;
        if remaining > 0 {
            let mask = (1u64 << remaining) - 1;
            total += (self.bits[full_words] & mask).count_ones() as usize;
        }
        total
    }

    /// Count of entries tracked by this mask.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Resize the mask to hold `new_count` entries; new entries default to valid.
    pub fn resize(&mut self, new_count: usize) {
        let new_num_words = (new_count + 63) / 64;
        self.bits.resize(new_num_words, u64::MAX);
        self.count = new_count;
    }
}

/// Describes how a vector stores its data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorType {
    /// Standard flat vector with one value per row.
    Flat,
    /// Constant vector: all rows share one value (space-efficient for repeated values).
    Constant,
    /// Dictionary vector: stores indices into a separate dictionary vector.
    Dictionary,
}

/// A selection vector: indices into another vector, used for filtered operations.
///
/// Instead of physically deleting rows, a selection vector picks which rows
/// to process, enabling zero-copy filtering.
#[derive(Debug, Clone)]
pub struct SelectionVector {
    indices: Vec<u32>,
}

impl SelectionVector {
    /// Create a new selection vector from the given indices.
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

    /// Returns `true` if the selection vector contains no indices.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Get the underlying indices as a slice.
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Create an incrementing selection vector `[0, 1, 2, ..., count-1]`.
    pub fn incrementing(count: usize) -> Self {
        Self {
            indices: (0..count as u32).collect(),
        }
    }
}

/// A columnar data vector -- the fundamental data container in QuackDB.
///
/// Stores a contiguous byte buffer (`data`) that is interpreted according to
/// `logical_type`. Variable-length types (Varchar) use an additional `offsets`
/// array. A `ValidityMask` tracks which entries are null.
pub struct Vector {
    logical_type: LogicalType,
    vector_type: VectorType,
    data: Vec<u8>,
    validity: ValidityMask,
    count: usize,
    // For variable-length types (Varchar): offsets into the data buffer.
    // offsets[i] .. offsets[i+1] is the byte range for value i.
    offsets: Option<Vec<u32>>,
}

impl Vector {
    /// Create a new flat vector of the given type with capacity for `count` values.
    // Hint: use `LogicalType::byte_width()` to size the data buffer.
    // For variable-length types, initialize an empty offsets vec.
    pub fn new(logical_type: LogicalType, count: usize) -> Self {
        let data_size = match logical_type.byte_width() {
            Some(width) => width * count,
            None => 0, // variable-length: data grows dynamically
        };
        let offsets = if logical_type.byte_width().is_none() {
            Some(vec![0u32]) // initial offset for variable-length types
        } else {
            None
        };
        Self {
            logical_type,
            vector_type: VectorType::Flat,
            data: vec![0u8; data_size],
            validity: ValidityMask::new_all_valid(count),
            count,
            offsets,
        }
    }

    /// Create a constant vector that repeats a single value for `count` rows.
    ///
    /// Only stores the value once; `vector_type` is set to `Constant`.
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

    /// Number of logical values in this vector.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Set the count of logical values.
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

    /// Get the raw data buffer as a byte slice.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable reference to the raw data buffer.
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Get a typed value at the given index, respecting the validity mask.
    ///
    /// Returns `ScalarValue::Null` if the index is marked invalid.
    // Hint: check validity first, then read from the byte buffer at
    // `index * byte_width` using the appropriate ScalarValue variant.
    pub fn get_value(&self, index: usize) -> ScalarValue {
        todo!()
    }

    /// Set a typed value at the given index.
    // Hint: write the value's bytes into `self.data` at the correct offset
    // and mark the index as valid in the validity mask.
    pub fn set_value(&mut self, index: usize, value: ScalarValue) {
        todo!()
    }

    /// Set the value at the given index to null.
    pub fn set_null(&mut self, index: usize) {
        self.validity.set_valid(index, false);
    }

    /// Flatten a constant vector into a flat vector by replicating the value.
    ///
    /// After this call, `vector_type` becomes `Flat`.
    pub fn flatten(&mut self) {
        todo!()
    }

    /// Copy values from this vector according to a selection vector into `target`.
    pub fn copy_with_selection(&self, sel: &SelectionVector, target: &mut Vector) {
        todo!()
    }

    /// Append a string value to a Varchar vector.
    // Hint: push the string bytes to `self.data`, record the new offset
    // in `self.offsets`, and mark the new entry as valid.
    pub fn append_string(&mut self, s: &str) {
        todo!()
    }

    /// Get a string value at `index` for Varchar vectors.
    ///
    /// Returns `None` if the index is null or the vector is not Varchar.
    // Hint: use the offsets array to find the byte range, then
    // `std::str::from_utf8` to convert.
    pub fn get_string(&self, index: usize) -> Option<&str> {
        todo!()
    }

    /// Reinterpret the data buffer as a typed slice of `T`.
    ///
    // Hint: use unsafe `std::slice::from_raw_parts` after casting
    // `self.data.as_ptr()` to `*const T`. Ensure alignment is correct.
    pub fn get_data_slice<T: Copy>(&self) -> &[T] {
        todo!()
    }

    /// Reinterpret the data buffer as a mutable typed slice of `T`.
    // Hint: same as `get_data_slice` but with `from_raw_parts_mut`.
    pub fn get_data_slice_mut<T: Copy>(&mut self) -> &mut [T] {
        todo!()
    }

    /// Reference to the offsets array (for Varchar vectors).
    pub fn offsets(&self) -> Option<&[u32]> {
        self.offsets.as_deref()
    }
}
