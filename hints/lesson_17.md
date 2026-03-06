# Lesson 17: Hash Join

## What You're Building
A hash join operator that combines rows from two data sources based on matching key columns. The JoinHashTable stores build-side rows indexed by serialized key bytes, then probes against incoming chunks to find matches. The HashJoinOperator orchestrates the two-phase process: first it consumes all build-side data, then it processes probe-side chunks. It supports six join types -- Inner, Left, Right, Full, Semi, and Anti -- each with distinct behavior for matched and unmatched rows. Hash join is the workhorse join algorithm in most analytical databases.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- JoinType enum selects the join behavior; match arms in the probe logic handle each variant differently
- [Collections](../concepts/collections.md) -- `HashMap<Vec<u8>, Vec<usize>>` maps serialized key bytes to lists of build-side row indices
- [Error Handling](../concepts/error_handling.md) -- probe returns `Result<DataChunk, String>`, propagating errors from key serialization or type mismatches

## Key Patterns

### Build/Probe Hash Join
The build phase inserts all rows from the smaller side into a hash table. The probe phase streams through the larger side, looking up each row's key in the hash table.

```rust
// Analogy: matching students to courses by student_id (NOT the QuackDB solution)
use std::collections::HashMap;

struct Student { id: u32, name: String }
struct Enrollment { student_id: u32, course: String }

fn build_index(students: &[Student]) -> HashMap<u32, Vec<usize>> {
    let mut index: HashMap<u32, Vec<usize>> = HashMap::new();
    for (i, s) in students.iter().enumerate() {
        index.entry(s.id).or_default().push(i);
    }
    index
}

fn probe(enrollments: &[Enrollment], index: &HashMap<u32, Vec<usize>>, students: &[Student]) {
    for e in enrollments {
        if let Some(matches) = index.get(&e.student_id) {
            for &idx in matches {
                println!("{} enrolled in {}", students[idx].name, e.course);
            }
        }
    }
}
```

### Handling Multiple Join Types with Match
Different join types share the same build logic but diverge in the probe phase. Use a match expression to handle each case: Inner emits only matches, Left keeps all probe rows, Semi returns probe-only columns, Anti returns probe rows with no match, and so on.

```rust
// Analogy: a file sync tool with different merge modes (NOT the QuackDB solution)
enum MergeMode { KeepBoth, KeepLeft, KeepNew, ExcludeMatched }

fn merge_files(local: &[&str], remote: &[&str], mode: MergeMode) -> Vec<String> {
    match mode {
        MergeMode::KeepBoth => { /* emit both sides */ todo!() }
        MergeMode::KeepLeft => { /* emit all local, fill missing remote with None */ todo!() }
        MergeMode::KeepNew => { /* emit local only if it matches remote */ todo!() }
        MergeMode::ExcludeMatched => { /* emit local only if NO remote match */ todo!() }
    }
}
```

## Step-by-Step Implementation Order
1. Start with `JoinHashTable::new()` -- initialize the HashMap, store build_keys and build_types, prepare an empty build_chunks vector
2. Implement `build()` -- for each row in the chunk, serialize the key columns into `Vec<u8>`, insert the row index into the HashMap entry; store the chunk in build_chunks
3. Implement `build_row_count()` -- sum the row counts across all stored build chunks
4. Implement `probe()` for Inner join first -- for each probe row, serialize its key columns, look up in the HashMap, and combine matched build and probe rows into the output chunk
5. Extend `probe()` to handle Left join -- same as Inner, but also emit probe rows that have no match (fill build columns with NULLs)
6. Add Right and Full joins -- track which build rows were matched; Right emits unmatched build rows with NULL probe columns; Full combines Left and Right logic
7. Add Semi and Anti joins -- Semi returns only probe columns for matched rows; Anti returns only probe columns for unmatched rows
8. Watch out for duplicate keys: a single probe row may match multiple build rows (the test `test_hash_join_duplicates` verifies this)

## Reading the Tests
- **`test_hash_join_inner`** builds with 3 rows (ids 1,2,3) and probes with 3 rows (ids 1,2,4). It asserts `result.count() == 2` because only ids 1 and 2 match. This confirms Inner join drops unmatched rows from both sides.
- **`test_hash_join_semi`** asserts `result.count() == 2` (matched probe rows) AND `result.column_count() == 2` (probe-side columns only). This reveals that Semi join must not include build-side columns in the output.
