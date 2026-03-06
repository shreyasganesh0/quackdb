# Lesson 22: Logical Plan

## What You're Building
A tree of logical plan nodes that represents a SQL query in a structured, algebraic form. Each node (Scan, Filter, Projection, Join, etc.) carries enough information to describe *what* to compute without specifying *how*. The `schema()` method propagates output column metadata through the tree so later stages know the shape of data at every point in the plan.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- the LogicalPlan and LogicalExpr enums have many variants; you will `match` on them throughout
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- plan nodes reference children via `Box<LogicalPlan>`, making the tree heap-allocated
- [Traits and Derive](../concepts/traits_and_derive.md) -- `#[derive(Debug, Clone)]` on all plan types, plus `Display` for pretty printing

## Key Patterns

### Recursive Enum Trees with Box
When a data structure refers to itself, Rust needs `Box` so the compiler knows the size. This is the same pattern used in any expression tree.

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
Each node derives its output schema from its children. A Filter passes through its child's schema unchanged. A Join merges the schemas of both children. A Projection builds a new schema from its expression list.

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
Pretty-printing a tree means recursing with an increasing indentation level and prefixing each node's label.

```rust
// Analogy: printing a file-system tree
fn print_tree(name: &str, children: &[(&str, Vec<(&str, Vec<()>)>)], depth: usize) -> String {
    let indent = "  ".repeat(depth);
    format!("{}{}\n", indent, name)
}
```

## Step-by-Step Implementation Order
1. Start with `schema()` -- match on every `LogicalPlan` variant. For `Scan`, return the stored schema (applying projection indices if present). For `Filter`, `Sort`, and `Limit`, delegate to `input.schema()`. For `Join`, merge left and right schemas. For `Aggregate`, build a schema from group expressions plus aggregate expressions.
2. Implement `pretty_print()` -- use a helper like `fn fmt_node(&self, indent: usize) -> String` that recursively formats each node with its name and children indented one level deeper.
3. Implement `from_select()` -- this is the most complex; build a Scan from the FROM clause, layer a Filter for WHERE, then a Projection for the SELECT list. Handle ORDER BY with Sort, LIMIT with Limit, and GROUP BY with Aggregate.
4. Watch out for the `Projection` schema -- you need to infer a column name and type for each projected expression, which can be tricky for aliases and aggregate functions.

## Reading the Tests
- **`test_scan_with_projection`** creates a 3-column scan with `projection: Some(vec![0, 2])` and asserts the output schema has 2 columns. This confirms that `schema()` must filter columns by the projection indices, not just return the full table schema.
- **`test_pretty_print`** builds a `Filter(Scan)` tree and checks that the printed string contains both "Filter" and "Scan"/"users". This tells you the printer must recursively include child nodes and mention the operator name.
