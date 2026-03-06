# Lesson 13: Expressions

## What You're Building
A recursive expression tree that represents computations like `(column_0 + 10) > column_1`. Each node in the tree is an `Expression` enum variant -- constants, column references, binary operations, unary operations, and casts. The `ExpressionExecutor` walks this tree recursively, evaluating each node against a `DataChunk` to produce a result `Vector`. This is the core of how databases evaluate WHERE clauses and SELECT expressions.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- the `Expression` enum has five variants, and evaluation requires matching on each
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- `BinaryOp` contains `Box<Expression>` children because a recursive enum needs indirection
- [Error Handling](../concepts/error_handling.md) -- type mismatches and invalid operations should return descriptive errors

## Key Patterns

### Recursive Enum with Box
Rust enums cannot contain themselves directly (infinite size). `Box` provides heap indirection:

```rust
enum Calc {
    Number(f64),
    Add(Box<Calc>, Box<Calc>),
    Negate(Box<Calc>),
}

fn eval(node: &Calc) -> f64 {
    match node {
        Calc::Number(n) => *n,
        Calc::Add(a, b) => eval(a) + eval(b),
        Calc::Negate(inner) => -eval(inner),
    }
}

// Build: -(3 + 4)
let tree = Calc::Negate(Box::new(
    Calc::Add(Box::new(Calc::Number(3.0)), Box::new(Calc::Number(4.0)))
));
assert_eq!(eval(&tree), -7.0);
```

Your `Expression` follows the same structure. `BinaryOp { left, right, .. }` and `UnaryOp { expr, .. }` hold `Box<Expression>` children. Evaluation recurses into children before combining results.

### Recursive Type Inference
Determining the output type also walks the tree recursively:

```rust
enum CalcType { Int, Float }

fn result_type(node: &Calc) -> CalcType {
    match node {
        Calc::Number(n) => if n.fract() == 0.0 { CalcType::Int } else { CalcType::Float },
        Calc::Add(a, b) => {
            // If either side is Float, result is Float
            match (result_type(a), result_type(b)) {
                (CalcType::Float, _) | (_, CalcType::Float) => CalcType::Float,
                _ => CalcType::Int,
            }
        }
        Calc::Negate(inner) => result_type(inner),
    }
}
```

Your `Expression::result_type` needs `input_types: &[LogicalType]` so that `ColumnRef(i)` can look up its type. Binary operations should follow numeric promotion rules (e.g., Int + Float = Float). Comparison operators always return Boolean.

## Step-by-Step Implementation Order
1. Start with `Expression::result_type()` -- match on each variant:
   - `Constant(v)` returns the scalar value's type
   - `ColumnRef(i)` indexes into `input_types`
   - `BinaryOp` recursively gets left/right types and determines the result (arithmetic yields the wider numeric type, comparisons yield Boolean)
   - `UnaryOp` depends on the operator (Negate preserves type, Not/IsNull/IsNotNull yield Boolean)
   - `Cast` simply returns the `target_type`
2. Implement `ExpressionExecutor::execute()` -- match on the expression variant:
   - `Constant` creates a vector filled with the constant value, sized to `chunk.len()`
   - `ColumnRef(i)` clones the column from the chunk
   - `BinaryOp` recursively executes left and right, then calls `execute_binary`
   - `UnaryOp` recursively executes the inner expression, then calls `execute_unary`
   - `Cast` recursively executes, then converts the vector to the target type
3. Implement `execute_binary()` -- iterate element-wise over the two vectors and apply the operation. Handle type combinations (Int+Int, Float+Float, etc.) and produce the correct output vector.
4. Implement `execute_unary()` -- iterate over the input vector and apply the operation (negate values, logical not, null checks).
5. Watch out for vector length mismatches -- both sides of a binary operation must have the same length. Constants should be broadcast to match the chunk size.
6. Watch out for division by zero -- return an error or produce a null, depending on what the tests expect.

## Reading the Tests
- Look for a test that builds a `BinaryOp` expression like `ColumnRef(0) + Constant(10)`, executes it against a chunk, and asserts the result vector contains each value incremented by 10. This confirms the recursive execution flow.
- Look for a test that nests expressions multiple levels deep (e.g., `(col0 + col1) > Constant(5)`) and checks that the result is a Boolean vector. This validates that `result_type` and `execute` handle deep recursion correctly.
