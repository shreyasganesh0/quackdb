# Lesson 25: Rule Optimizer

## What You're Building
A rule-based query optimizer that transforms a logical plan into a more efficient equivalent. Each rule (predicate pushdown, projection pushdown, constant folding, filter merge, limit pushdown) is a struct implementing the `OptimizerRule` trait. The `optimize()` function applies all rules in a loop until the plan stops changing (fixpoint iteration). This is how real databases make queries faster without changing their meaning.

## Rust Concepts You'll Need
- [Traits and Derive](../concepts/traits_and_derive.md) -- the `OptimizerRule` trait defines `name()` and `apply()` that each rule struct implements
- [Trait Objects](../concepts/trait_objects.md) -- `Vec<Box<dyn OptimizerRule>>` stores heterogeneous rule types in a single collection
- [Iterators](../concepts/iterators.md) -- iterating over rules and applying them sequentially

## Key Patterns

### Trait Objects for Polymorphic Rules
Each optimization rule is a different struct, but they all share the same interface. Using `Box<dyn OptimizerRule>` lets you store them in a Vec and iterate uniformly.

```rust
// Analogy: text transformations applied to a document (NOT the QuackDB solution)
trait TextRule {
    fn name(&self) -> &str;
    fn apply(&self, text: String) -> String;
}

struct Lowercase;
impl TextRule for Lowercase {
    fn name(&self) -> &str { "lowercase" }
    fn apply(&self, text: String) -> String { text.to_lowercase() }
}

struct TrimWhitespace;
impl TextRule for TrimWhitespace {
    fn name(&self) -> &str { "trim" }
    fn apply(&self, text: String) -> String { text.trim().to_string() }
}

fn apply_rules(text: String, rules: &[Box<dyn TextRule>]) -> String {
    rules.iter().fold(text, |t, rule| rule.apply(t))
}
```

### Fixed-Point Iteration
Apply all rules in a loop. If no rule changes the plan during an iteration, you have reached a fixpoint and can stop. Use a maximum iteration count to prevent infinite loops.

```rust
// Analogy: simplifying a fraction repeatedly until it cannot be reduced further
fn simplify(mut num: u64, mut den: u64, max_iter: usize) -> (u64, u64) {
    for _ in 0..max_iter {
        let g = gcd(num, den);
        if g == 1 { break; } // fixpoint reached
        num /= g;
        den /= g;
    }
    (num, den)
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}
```

### Recursive Tree Transformation
Each rule walks the plan tree and rewrites matching patterns. For example, predicate pushdown looks for `Filter(Projection(...))` and swaps them to `Projection(Filter(...))`. You must recursively apply the transformation to children first, then check the current node.

```rust
// Analogy: simplifying nested math expressions (NOT the QuackDB solution)
enum Expr {
    Num(i64),
    Add(Box<Expr>, Box<Expr>),
}

fn fold_constants(expr: Expr) -> Expr {
    match expr {
        Expr::Add(l, r) => {
            let l = fold_constants(*l);
            let r = fold_constants(*r);
            if let (Expr::Num(a), Expr::Num(b)) = (&l, &r) {
                Expr::Num(a + b)
            } else {
                Expr::Add(Box::new(l), Box::new(r))
            }
        }
        other => other,
    }
}
```

## Step-by-Step Implementation Order
1. Start with `default_rules()` -- return a Vec containing one `Box::new(...)` for each of the five rule structs.
2. Implement `optimize()` -- loop up to `max_iterations`. In each iteration, fold over all rules applying each one. Compare the plan before and after (you can use `pretty_print()` or `Debug` format for comparison). If nothing changed, break early.
3. Implement `ConstantFolding` -- the simplest rule. Recursively walk the plan. For any `LogicalExpr::BinaryOp` where both sides are `Literal`, evaluate the operation and replace with a single `Literal`.
4. Implement `FilterMerge` -- look for `Filter { input: Filter { .. } }`. Combine the two predicates with an AND (a new `BinaryOp`) and keep only one Filter node.
5. Implement `PredicatePushdown` -- for `Filter(Projection(...))`, move the filter below the projection. For `Filter(Join(...))`, push the filter to the appropriate side based on which columns it references.
6. Implement `ProjectionPushdown` -- for `Projection(Scan(...))`, set the scan's projection field to include only the referenced column indices.
7. Implement `LimitPushdown` -- propagate limit hints through non-blocking operators when safe.
8. Watch out for column index invalidation -- pushing a filter below a projection may require remapping column indices in the predicate.

## Reading the Tests
- **`test_optimize_fixpoint`** builds a `Filter(Projection(Scan))` plan, applies `default_rules()` with up to 10 iterations, and simply expects it to return `Ok` without hanging. This confirms your fixpoint loop terminates.
- **`test_predicate_pushdown_through_projection`** builds `Filter(Projection(Scan))` and checks that after applying `PredicatePushdown`, the pretty-printed output contains "Projection" (suggesting the filter moved below it, so projection is now the outer node). The test does not check exact structure -- just that the rewrite happened.
- **`test_filter_merge`** stacks two Filter nodes and expects them to be merged into one after applying `FilterMerge`.
