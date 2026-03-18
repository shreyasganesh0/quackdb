# Lesson 22: Logical Plan

## What You're Building
A tree of logical plan nodes that represents a SQL query in a structured, algebraic form. Each node (Scan, Filter, Projection, Join, etc.) carries enough information to describe *what* to compute without specifying *how*. The `schema()` method propagates output column metadata through the tree so later stages know the shape of data at every point in the plan.

## Concept Recap
Building on Lessons 20-21: The parser produces an AST from SQL text. The logical plan is a more structured representation -- while the AST mirrors SQL syntax, the logical plan mirrors relational algebra. The binder (L23) will translate AST nodes into these plan nodes. Later, the physical planner (L24) will convert each logical node into the `PhysicalOperator` types (Filter, Projection, Sort) you built in Lessons 15-19.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- the LogicalPlan and LogicalExpr enums have many variants; you will `match` on them throughout
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- plan nodes reference children via `Box<LogicalPlan>`, making the tree heap-allocated
- [Traits and Derive](../concepts/traits_and_derive.md) -- `#[derive(Debug, Clone)]` on all plan types, plus `Display` for pretty printing

## Key Patterns

### Recursive Enum Trees with Box
When a data structure refers to itself, Rust needs `Box` so the compiler knows the size. Think of it like an organizational chart -- each manager node points to subordinate nodes. The pointer (Box) has a fixed size even though the subtree can be arbitrarily large.

```rust
// Analogy: an arithmetic expression evaluator (NOT the QuackDB solution)
enum MathExpr {
    Number(f64),
    Add(Box<MathExpr>, Box<MathExpr>),
    Mul(Box<MathExpr>, Box<MathExpr>),
}

fn eval(expr: &MathExpr) -> f64 {
    match expr {
        MathExpr::Number(n) => *n,
        MathExpr::Add(l, r) => eval(l) + eval(r),
        MathExpr::Mul(l, r) => eval(l) * eval(r),
    }
}
```

### Schema Propagation
Each node derives its output schema from its children. Think of it like a plumbing system -- each pipe section determines what flows out based on what flows in. A filter does not change the schema (same pipes, fewer drops). A join widens the schema (connecting two pipe systems). A projection reshapes it (selecting and rearranging pipes).

```rust
// Analogy: a pipeline of image transformations tracking output dimensions
struct Dimensions { width: u32, height: u32 }

enum ImageOp {
    Load { dims: Dimensions },
    Crop { input: Box<ImageOp>, new_dims: Dimensions },
    Stack { left: Box<ImageOp>, right: Box<ImageOp> }, // side by side
}

fn output_dims(op: &ImageOp) -> Dimensions {
    match op {
        ImageOp::Load { dims } => dims.clone(),
        ImageOp::Crop { new_dims, .. } => new_dims.clone(),
        ImageOp::Stack { left, right } => Dimensions {
            width: output_dims(left).width + output_dims(right).width,
            height: output_dims(left).height,
        },
    }
}
```

### Indented Tree Printing
Pretty-printing a tree means recursing with an increasing indentation level and prefixing each node's label. Think of it like printing a file system -- each subdirectory is indented one level deeper than its parent, visually showing the hierarchy.

```rust
// Analogy: printing a file-system tree
fn print_tree(name: &str, children: &[(&str, Vec<(&str, Vec<()>)>)], depth: usize) -> String {
    let indent = "  ".repeat(depth);
    format!("{}{}\n", indent, name)
}
```

## Common Mistakes
- Not applying projection indices in `Scan::schema()`. When `projection: Some(vec![0, 2])` is set, the schema should return only columns 0 and 2, not the full table schema.
- Making `Filter` or `Sort` change the schema. These operators only affect which rows appear or their order -- the columns (schema) remain identical to their input child's schema.
- Forgetting that `Join` schema is the concatenation of left and right schemas. The output has all left columns followed by all right columns.

## Step-by-Step Implementation Order
1. Implement `Schema::new()`, `column_count()`, `find_column()`, `types()`, and `merge()` -- these are foundational helpers used everywhere
2. Implement `schema()` -- match on every `LogicalPlan` variant. For `Scan`, return the stored schema (applying projection indices if present). For `Filter`, `Sort`, and `Limit`, delegate to `input.schema()`. For `Join`, merge left and right schemas. For `Aggregate`, build a schema from group expressions plus aggregate expressions.
3. Implement `pretty_print()` -- use a helper like `fn fmt_node(&self, indent: usize) -> String` that recursively formats each node with its name and children indented one level deeper
4. Implement `Display for LogicalPlan` -- delegate to `pretty_print()`
5. Implement `from_select()` if needed -- build a Scan from the FROM clause, layer a Filter for WHERE, then a Projection for the SELECT list. Handle ORDER BY with Sort, LIMIT with Limit, and GROUP BY with Aggregate.
6. Watch out for the `Projection` schema -- you need to infer a column name and type for each projected expression
7. Watch out for the `Aggregate` schema -- it combines the group-by columns with the aggregate result columns
8. Make sure every plan variant is covered in the `schema()` match; a missing arm will not compile but an incorrect implementation will produce subtle bugs

## Reading the Tests
- **`test_schema`** constructs a Schema with columns ["id" Int32, "name" Varchar] and checks `column_count()`, `find_column()`, and `types()`. This validates the basic Schema struct operations.
- **`test_schema_merge`** merges two single-column schemas and checks the result has 2 columns with the correct names. This is the foundation for join schema computation.
- **`test_scan_plan_schema`** creates a Scan with no projection and checks `schema().column_count() == 2`. This is the simplest schema propagation test.
- **`test_scan_with_projection`** creates a 3-column scan with `projection: Some(vec![0, 2])` and asserts the output schema has 2 columns. This confirms that `schema()` must filter columns by the projection indices, not just return the full table schema.
- **`test_filter_plan_schema`** wraps a Scan in a Filter and checks the schema is unchanged (still 1 column). This validates that Filter passes through its child's schema.
- **`test_join_plan_schema`** creates a Join of two single-column scans and checks `schema().column_count() == 2`. This confirms join schema concatenation.
- **`test_limit_plan_schema`** wraps a Scan in a Limit and checks the schema is unchanged. Like Filter, Limit does not alter the column set.
- **`test_pretty_print`** builds a `Filter(Scan)` tree and checks that the printed string contains both "Filter" and "Scan"/"users". This tells you the printer must recursively include child nodes.
- **`test_aggregate_plan_schema`** creates an Aggregate with one group expr and one agg expr, and checks `schema().column_count() == 2`. This validates that the aggregate schema combines group and aggregate columns.
- **`test_display`** checks that `format!("{}", plan)` produces a non-empty string. This is a basic smoke test for your Display implementation.
