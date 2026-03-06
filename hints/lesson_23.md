# Lesson 23: Binder & Catalog

## What You're Building
A catalog that stores table definitions and their data, and a binder that resolves column names, checks types, and translates parsed SQL (AST) into a logical plan. The catalog is the database's "phone book" -- it maps table names to their column schemas. The binder walks the AST, looks up names in the catalog, and produces a fully resolved `LogicalPlan` where every column reference has a concrete index and type.

## Rust Concepts You'll Need
- [Lifetimes](../concepts/lifetimes.md) -- `Binder<'a>` borrows `&'a Catalog`, ensuring the catalog outlives the binder
- [Collections](../concepts/collections.md) -- `HashMap<String, TableInfo>` for the catalog, `Vec` for scope columns
- [Error Handling](../concepts/error_handling.md) -- returning `Result<_, BindError>` or `Result<_, String>` for missing tables, ambiguous columns, etc.

## Key Patterns

### Struct Borrowing with Lifetime Annotations
When a struct needs read access to another struct without owning it, use a lifetime-annotated reference. This guarantees the borrowed data stays alive.

```rust
// Analogy: a spell-checker borrowing a dictionary (NOT the QuackDB solution)
struct Dictionary {
    words: std::collections::HashSet<String>,
}

struct SpellChecker<'a> {
    dict: &'a Dictionary,
}

impl<'a> SpellChecker<'a> {
    fn new(dict: &'a Dictionary) -> Self {
        Self { dict }
    }

    fn check(&self, word: &str) -> bool {
        self.dict.words.contains(word)
    }
}
```

### Name Resolution with Scope Tracking
A scope holds the columns available at a given point in the query. When binding `SELECT users.name`, the scope must find which table provides "name" and at what index. Ambiguity arises when two tables define the same column name without qualification.

```rust
// Analogy: resolving variable names in nested scopes (NOT the QuackDB solution)
struct VarScope {
    vars: Vec<(Option<String>, String, usize)>, // (module, name, slot)
}

impl VarScope {
    fn resolve(&self, module: Option<&str>, name: &str) -> Result<usize, String> {
        let matches: Vec<_> = self.vars.iter()
            .filter(|(m, n, _)| {
                n == name && module.map_or(true, |q| m.as_deref() == Some(q))
            })
            .collect();
        match matches.len() {
            0 => Err(format!("unknown variable: {}", name)),
            1 => Ok(matches[0].2),
            _ => Err(format!("ambiguous variable: {}", name)),
        }
    }
}
```

### HashMap Guard for Duplicates
When creating a table, check for existence first to return an error on duplicates. When dropping, check that the table exists before removing.

```rust
// Analogy: a user registry that rejects duplicate usernames
use std::collections::HashMap;

fn register(users: &mut HashMap<String, u64>, name: String, id: u64) -> Result<(), String> {
    if users.contains_key(&name) {
        return Err(format!("user '{}' already exists", name));
    }
    users.insert(name, id);
    Ok(())
}
```

## Step-by-Step Implementation Order
1. Start with `Catalog::new()` -- initialize both `tables` and `table_data` as empty HashMaps.
2. Implement `create_table()` -- check if the table already exists (return `Err` if so), then insert into `tables` and create an empty `Vec<DataChunk>` entry in `table_data`.
3. Implement `get_table()`, `drop_table()`, `insert_data()`, `get_table_data()` -- straightforward HashMap operations. For `get_table_data`, return `Some(slice)` via `.as_deref()` or `.map(|v| v.as_slice())`.
4. Implement `BindScope::resolve()` -- filter the columns list by table qualifier (if given) and column name. Return an error on zero matches ("unknown column") or multiple matches ("ambiguous column").
5. Implement `Binder::bind()` -- match on the AST `Statement` variants and delegate to `bind_select` or handle `CreateTable`/`Insert` directly.
6. Implement `bind_select()` -- look up the table in the catalog via `bind_table_ref`, build a scope, bind WHERE/SELECT/GROUP BY/ORDER BY expressions, and assemble the plan nodes in order: Scan, Filter, Aggregate, Projection, Sort, Limit.
7. Watch out for wildcard expansion -- `SELECT *` must be expanded into explicit `ColumnRef` entries for every column in the scope.

## Reading the Tests
- **`test_bind_scope_resolution`** directly constructs a `BindScope` with two tables that both have an "id" column. It checks that a qualified lookup `resolve(Some("users"), "id")` returns index 0, that an unqualified `resolve(None, "id")` fails (ambiguous), and that `resolve(None, "name")` succeeds (unique). This is the core logic of name resolution.
- **`test_catalog_duplicate_create`** calls `create_table` twice with the same name and expects the second call to return `Err`. This confirms you need a containment check before inserting.
