# Lesson 06: Dictionary Encoding

## What You're Building
Dictionary encoding replaces repeated values with small integer codes, storing each
unique value only once in a lookup table. This is the compression workhorse for
low-cardinality string columns (e.g., country names, status codes). You will build a
bidirectional Dictionary mapping, encode/decode functions that handle NULLs via
`Option<T>`, and a heuristic to decide when dictionary encoding is worthwhile.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- Option<T> for nullable values in the input data
- [Structs and Impl](../concepts/structs_and_impl.md) -- Dictionary<T> with HashMap and Vec for bidirectional lookup
- [Traits and Derive](../concepts/traits_and_derive.md) -- trait bounds Eq + Hash required for HashMap keys

## Key Patterns

### HashMap for Bidirectional Mapping
A dictionary needs to go both ways: value-to-code (for encoding) and code-to-value
(for decoding). Use a HashMap for the first direction and a Vec for the second.

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
a code (or a sentinel) and record the position as null in the `nulls` vector.

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
`distinct_count / total_count` and compares to a threshold.

```rust
// Analogy: should we build an index for a phone book?
fn should_index(entries: &[&str], ratio_threshold: f64) -> bool {
    use std::collections::HashSet;
    let unique: HashSet<_> = entries.iter().collect();
    (unique.len() as f64 / entries.len() as f64) < ratio_threshold
}
```

## Step-by-Step Implementation Order
1. Start with `Dictionary::new()` -- empty HashMap and empty Vec
2. Implement `insert()` -- check if value exists in HashMap; if yes return existing code, if no push to Vec and insert into HashMap
3. Implement `get_code()` and `get_value()` -- straightforward lookups returning Option
4. Implement `cardinality()` -- return the length of code_to_value
5. Implement `encode()` -- iterate `&[Option<T>]`, for each Some(v) insert into dictionary and record the code; for None, push a sentinel code (e.g., 0) and mark null
6. Implement `decode()` -- iterate codes and nulls; if null return None, otherwise look up the code in the dictionary and return Some
7. Implement `should_dictionary_encode()` -- count distinct values with a HashSet, compare ratio to threshold
8. Implement `compression_ratio()` -- estimate original size vs dictionary size + codes array size
9. Watch out for: the null bitmap and codes array must stay aligned (same length); inserting a duplicate value must return the same code, not a new one

## Reading the Tests
- **`test_dictionary_basic`** inserts "hello", "world", "hello" and asserts the first and third insertions return the same code, and cardinality is 2. This confirms your insert must be idempotent for duplicate values.
- **`test_dictionary_with_nulls`** encodes `[Some("a"), None, Some("b"), None, Some("a")]` and expects a perfect roundtrip. Your decode must check the nulls vector to decide whether to return None or look up the code.
- **`test_dictionary_low_cardinality`** has 1000 values with only 5 distinct ones and threshold 0.1. Since 5/1000 = 0.005 < 0.1, `should_dictionary_encode` must return true.
