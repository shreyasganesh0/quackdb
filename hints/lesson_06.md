# Lesson 06: Dictionary Encoding

## What You're Building
Dictionary encoding replaces repeated values with small integer codes, storing each
unique value only once in a lookup table. This is the compression workhorse for
low-cardinality string columns (e.g., country names, status codes). You will build a
bidirectional Dictionary mapping, encode/decode functions that handle NULLs via
`Option<T>`, and a heuristic to decide when dictionary encoding is worthwhile.

## Concept Recap
Building on Lesson 05: You used RLE to compress sorted runs of identical values. Dictionary encoding complements RLE by handling the *unsorted* case -- when the same values repeat throughout a column but not consecutively. Both are compression strategies that will feed into the compression framework in Lesson 08.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- Option<T> for nullable values in the input data
- [Structs and Impl](../concepts/structs_and_impl.md) -- Dictionary<T> with HashMap and Vec for bidirectional lookup
- [Traits and Derive](../concepts/traits_and_derive.md) -- trait bounds Eq + Hash required for HashMap keys

## Key Patterns

### HashMap for Bidirectional Mapping
A dictionary needs to go both ways: value-to-code (for encoding) and code-to-value
(for decoding). Use a HashMap for the first direction and a Vec for the second. Think
of a phone book: you can look up a person's number (value-to-code) or use a reverse
directory to find who owns a number (code-to-value).

```rust
// Analogy: a color palette in an image editor (NOT the QuackDB solution)
use std::collections::HashMap;

struct Palette {
    color_to_index: HashMap<(u8, u8, u8), u16>,
    index_to_color: Vec<(u8, u8, u8)>,
}

impl Palette {
    fn new() -> Self {
        Palette { color_to_index: HashMap::new(), index_to_color: Vec::new() }
    }

    fn add_color(&mut self, rgb: (u8, u8, u8)) -> u16 {
        if let Some(&idx) = self.color_to_index.get(&rgb) {
            return idx;  // already in palette
        }
        let idx = self.index_to_color.len() as u16;
        self.index_to_color.push(rgb);
        self.color_to_index.insert(rgb, idx);
        idx
    }

    fn lookup(&self, idx: u16) -> Option<&(u8, u8, u8)> {
        self.index_to_color.get(idx as usize)
    }
}
```

### Option<T> for Nullable Data
The encode function takes `&[Option<T>]`. For `None` values, you still need to emit
a code (or a sentinel) and record the position as null in the `nulls` vector. Think
of a survey form where some questions are left blank -- you still number each question,
but mark blank ones as "no response."

```rust
// Analogy: encoding survey responses where some are unanswered
fn encode_responses(responses: &[Option<&str>]) -> (Vec<u32>, Vec<bool>) {
    let mut codes = Vec::new();
    let mut is_null = Vec::new();
    for resp in responses {
        match resp {
            Some(_) => { codes.push(1); is_null.push(false); }
            None    => { codes.push(0); is_null.push(true); }
        }
    }
    (codes, is_null)
}
```

### Cardinality Threshold Heuristic
Dictionary encoding saves space only when the number of distinct values is small
relative to the total count. The `should_dictionary_encode` function computes
`distinct_count / total_count` and compares to a threshold. Think of it like deciding
whether to create abbreviations in a document -- only worthwhile if the same long
words appear frequently.

```rust
// Analogy: should we build an index for a phone book?
fn should_index(entries: &[&str], ratio_threshold: f64) -> bool {
    use std::collections::HashSet;
    let unique: HashSet<_> = entries.iter().collect();
    (unique.len() as f64 / entries.len() as f64) < ratio_threshold
}
```

### Idempotent Insert
Inserting a value that already exists must return the same code, not create a duplicate.
This is like assigning employee IDs -- if "Alice" already has ID 7, hiring Alice again
should return 7, not assign a new ID.

```rust
// Analogy: stable ID assignment
fn get_or_assign_id(
    map: &mut HashMap<String, u32>,
    next_id: &mut u32,
    name: &str,
) -> u32 {
    if let Some(&id) = map.get(name) {
        return id;
    }
    let id = *next_id;
    *next_id += 1;
    map.insert(name.to_string(), id);
    id
}
```

## Step-by-Step Implementation Order
1. Start with `Dictionary::new()` -- empty HashMap and empty Vec
2. Implement `insert()` -- check if value exists in HashMap; if yes return existing code, if no push to Vec and insert into HashMap
3. Implement `get_code()` and `get_value()` -- straightforward lookups returning Option
4. Implement `cardinality()` -- return the length of code_to_value Vec
5. Implement `encode()` -- iterate `&[Option<T>]`, for each Some(v) insert into dictionary and record the code; for None, push a sentinel code (e.g., 0) and mark null
6. Implement `decode()` -- iterate codes and nulls; if null return None, otherwise look up the code in the dictionary and return Some
7. Implement `should_dictionary_encode()` -- count distinct values with a HashSet, compare ratio to threshold
8. Implement `compression_ratio()` -- estimate original size vs dictionary size + codes array size
9. Handle empty input: return empty encoded result with cardinality 0

## Common Mistakes
- **Assigning new codes for duplicate values**: The `insert` method must check the HashMap first. If you always push to the Vec, duplicates get multiple codes and decode will produce wrong values.
- **Misaligning codes and nulls arrays**: Both arrays must be the same length -- one entry per input element. If a null entry is missing from the nulls vector or an extra code is inserted, decode will read the wrong positions.
- **Using > instead of < for the threshold check**: `should_dictionary_encode` returns true when the distinct ratio is *below* the threshold (meaning low cardinality is good for dictionaries), not above it.

## Reading the Tests
- **`test_dictionary_basic`** inserts "hello", "world", "hello" and asserts the first and third insertions return the same code, and cardinality is 2. This confirms your insert must be idempotent for duplicate values.
- **`test_dictionary_lookup`** inserts 10, 20, 30 into an i32 dictionary and checks that codes are assigned in order (0, 1, 2). It also checks `get_code(&40) == None` and `get_value(3) == None` -- absent values and out-of-range codes must return None.
- **`test_dictionary_encode_strings`** encodes 5 fruit strings with 3 distinct values. It checks cardinality is 3, codes length is 5, and duplicate "apple" entries share the same code. This is the core encoding behavior.
- **`test_dictionary_roundtrip`** encodes `[Some(10), Some(20), Some(10), Some(30), Some(20), Some(10)]` and verifies decode produces the exact same vector. Full lossless roundtrip.
- **`test_dictionary_with_nulls`** encodes `[Some("a"), None, Some("b"), None, Some("a")]` and expects a perfect roundtrip. Your decode must check the nulls vector to decide whether to return None or look up the code.
- **`test_dictionary_all_nulls`** encodes `[None, None, None]` -- all nulls with no real values. Must roundtrip correctly with cardinality 0 or minimal.
- **`test_dictionary_high_cardinality`** has 1000 unique values and threshold 0.5. Since 1000/1000 = 1.0 > 0.5, `should_dictionary_encode` must return false. Dictionary encoding hurts when every value is unique.
- **`test_dictionary_low_cardinality`** has 1000 values with only 5 distinct ones and threshold 0.1. Since 5/1000 = 0.005 < 0.1, `should_dictionary_encode` must return true.
- **`test_dictionary_compression_ratio`** encodes 10000 values with only 10 distinct categories and checks ratio > 1.0. Low cardinality should compress well.
- **`test_dictionary_empty`** encodes an empty vector and checks cardinality 0, codes length 0, and empty decode. Edge case for the smallest input.

## What Comes Next
You now have two column compression strategies: RLE for sorted runs and dictionary
encoding for scattered repeats. Lesson 07 introduces **bitpacking and delta encoding**
for integer columns, which complete the compression toolkit. Delta encoding is
especially powerful for timestamp columns (sorted, small differences), and bitpacking
shrinks any integer column by using only the minimum bits needed.
