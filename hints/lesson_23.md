# Lesson 23: Binder & Catalog

## What You're Building
A catalog that stores table definitions and their data, and a binder that resolves column names, checks types, and translates parsed SQL (AST) into a logical plan. The catalog is the database's "phone book" -- it maps table names to their column schemas. The binder walks the AST, looks up names in the catalog, and produces a fully resolved `LogicalPlan` where every column reference has a concrete index and type.

**Core concept count: 2** — the catalog (metadata storage) and name resolution (binding). Everything else (scope tracking, wildcard expansion, error variants) is scaffolding that supports these two.

> **Unified Concept:** Catalog stores metadata, binder resolves names -- together they are ONE concept: name resolution. The catalog is passive data (a phone book), the binder is the active lookup process (dialing the number). They are split into separate files because one is a data structure and the other is an algorithm, but the concept is singular: turning string names into resolved, typed references.

## Concept Recap
Building on Lessons 20-22: The lexer (L20) produces tokens, the parser (L21) produces an AST, and the logical plan (L22) defines the node types. The binder bridges the AST and the logical plan -- it takes the unresolved names from the parser and resolves them against the catalog to produce a fully typed `LogicalPlan` tree that the physical planner (L24) can convert into executable operators.

## Rust Concepts You'll Need
- [Lifetimes](../concepts/lifetimes.md) -- `Binder<'a>` borrows `&'a Catalog`, ensuring the catalog outlives the binder
- [Collections](../concepts/collections.md) -- `HashMap<String, TableInfo>` for the catalog, `Vec` for scope columns
- [Error Handling](../concepts/error_handling.md) -- returning `Result<_, BindError>` or `Result<_, String>` for missing tables, ambiguous columns, etc.

## Key Patterns

### Struct Borrowing with Lifetime Annotations
When a struct needs read access to another struct without owning it, use a lifetime-annotated reference. Think of it like a library card -- the card (Binder) gives you access to books (Catalog data) without you taking the books home. The library must remain open (alive) as long as you hold the card.

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
A scope holds the columns available at a given point in the query. When binding `SELECT users.name`, the scope must find which table provides "name" and at what index. Think of it like a compiler's symbol table -- when you see a variable name, you look through the current scope to find its declaration, its type, and its memory location.

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
When creating a table, check for existence first to return an error on duplicates. Think of it like registering a domain name -- if someone already owns it, the registry rejects your request.

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

## Common Mistakes
- Not handling `SELECT *` wildcard expansion. When the binder sees `SelectItem::Wildcard`, it must expand it into explicit `ColumnRef` entries for every column in the current scope, not pass it through as-is.
- Forgetting to handle qualified vs unqualified column references. `users.id` should resolve against the "users" table specifically, while bare `id` should search all tables in scope and error if ambiguous.
- Not building the scope correctly for JOINs. After a JOIN, the scope must contain columns from both tables, which is what enables qualified references like `users.id` and `orders.user_id` in the same query.

## Step-by-Step Implementation Order
1. Start with `Catalog::new()` -- initialize both `tables` and `table_data` as empty HashMaps.
2. Implement `create_table()` -- check if the table already exists (return `Err` if so), then insert into `tables` and create an empty `Vec<DataChunk>` entry in `table_data`.
3. Implement `get_table()`, `drop_table()`, `insert_data()`, `get_table_data()` -- straightforward HashMap operations. For `get_table_data`, return `Some(slice)` via `.as_deref()` or `.map(|v| v.as_slice())`.
4. Implement `BindScope::resolve()` -- filter the columns list by table qualifier (if given) and column name. Return an error on zero matches ("unknown column") or multiple matches ("ambiguous column").
5. Implement `Binder::bind()` -- match on the AST `Statement` variants and delegate to `bind_select` or handle `CreateTable`/`Insert` directly.
6. Implement `bind_select()` -- look up the table in the catalog via `bind_table_ref`, build a scope, bind WHERE/SELECT/GROUP BY/ORDER BY expressions, and assemble the plan nodes in order: Scan, Filter, Aggregate, Projection, Sort, Limit.
7. Handle wildcard expansion -- `SELECT *` must be expanded into explicit `ColumnRef` entries for every column in the scope.
8. Handle JOIN binding -- build a combined scope from both tables, resolve the ON condition against the combined scope.
9. Test error paths: unknown table, unknown column, and ambiguous column references.

## Reading the Tests
- **`test_catalog_create_get`** creates two tables and retrieves "users", checking it has 3 columns with the first named "id". This validates basic catalog CRUD.
- **`test_catalog_table_not_found`** looks up "nonexistent" and asserts `is_none()`. This confirms your catalog returns None for missing tables rather than panicking.
- **`test_catalog_drop_table`** drops "users" and verifies it is no longer found. This validates the drop operation removes the table from the catalog.
- **`test_catalog_duplicate_create`** calls `create_table` twice with the same name and expects the second call to return `Err`. This confirms you need a containment check before inserting.
- **`test_bind_simple_select`** binds `"SELECT id, name FROM users"` and checks the output schema has 2 columns. This validates end-to-end binding: parsing, scope construction, and column resolution.
- **`test_bind_select_star`** binds `"SELECT * FROM users"` and checks the output schema has 3 columns (all columns in users). This confirms wildcard expansion works correctly.
- **`test_bind_unknown_table`** tries to bind `"SELECT * FROM nonexistent"` and expects an error. This validates the binder checks the catalog for table existence.
- **`test_bind_unknown_column`** tries to bind `"SELECT nonexistent FROM users"` and expects an error. This validates that the binder rejects references to columns not in the table schema.
- **`test_bind_where_clause`** binds a query with WHERE and checks the pretty-printed plan contains "Filter". This confirms the binder creates a Filter node for WHERE clauses.
- **`test_bind_join`** binds `"SELECT users.name, orders.amount FROM users INNER JOIN orders ON users.id = orders.user_id"` and checks the output schema has 2 columns. This validates join binding with qualified column references.
- **`test_bind_alias`** binds `"SELECT id AS user_id FROM users"` and expects success. This confirms alias handling does not break the binding process.
- **`test_bind_aggregate`** binds `"SELECT age, COUNT(*) FROM users GROUP BY age"` and checks 2 output columns. This validates GROUP BY and aggregate function binding.
- **`test_bind_scope_resolution`** directly constructs a `BindScope` with two tables that both have an "id" column. It checks that qualified `resolve(Some("users"), "id")` returns index 0, unqualified `resolve(None, "id")` fails (ambiguous), and `resolve(None, "name")` succeeds (unique). This is the core logic of name resolution.
- **`test_table_info_helpers`** checks `find_column("id")`, `find_column("xyz")`, and `schema_types()`. This validates the TableInfo utility methods.

## Rust Sidebar: Lifetime Annotations
If you hit `missing lifetime specifier` or `borrowed value does not live long enough` on `Binder`, here's what's happening: `Binder` borrows the `Catalog` via `&Catalog`, but Rust needs to know how long that borrow lasts. Without a lifetime annotation, the compiler cannot prove the catalog outlives the binder.
The fix: declare `struct Binder<'a> { catalog: &'a Catalog }` and `impl<'a> Binder<'a>`. The `'a` says "the binder can only live as long as the catalog it references." When constructing: `let binder = Binder::new(&catalog);` -- the compiler infers `'a` from the catalog's scope automatically.
