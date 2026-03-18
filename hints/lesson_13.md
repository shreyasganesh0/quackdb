# Lesson 13: Expressions

## What You're Building
A recursive expression tree that represents computations like `(column_0 + 10) > column_1`. Each node in the tree is an `Expression` enum variant -- constants, column references, binary operations, unary operations, and casts. The `ExpressionExecutor` walks this tree recursively, evaluating each node against a `DataChunk` to produce a result `Vector`. This is the core of how databases evaluate WHERE clauses and SELECT expressions.

## Concept Recap
Building on Lessons 10-12: You'll use `DataChunk`, `Vector`, `ScalarValue`, and `LogicalType` from the storage engine. Expressions operate on the vectorized data structures you already built -- each expression evaluation takes a `DataChunk` in and produces a `Vector` out.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- the `Expression` enum has five variants, and evaluation requires matching on each
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- `BinaryOp` contains `Box<Expression>` children because a recursive enum needs indirection
- [Error Handling](../concepts/error_handling.md) -- type mismatches and invalid operations should return descriptive errors

## Key Patterns

### Recursive Enum with Box
Rust enums cannot contain themselves directly (infinite size). `Box` provides heap indirection. Think of it like a folder that contains a shortcut to another folder -- the shortcut has a fixed size even though the target folder can be arbitrarily deep.

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
Determining the output type also walks the tree recursively. Think of it like asking "what color paint will I get?" -- you need to know the colors of the ingredients before you can determine the mixture.

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

### Element-Wise Vector Operations
When executing a binary operation, think of it like a spreadsheet formula applied to two columns -- you walk both vectors in lockstep, applying the operation at each row index to produce a new result vector of the same length.

## Common Mistakes
- Forgetting to broadcast constants to match the chunk size. A `Constant(42)` expression must produce a vector with one value per row in the chunk, not a single-element vector.
- Not handling NULL propagation in binary operations. In SQL, `NULL + 5` is `NULL`, not 5. Check validity bits before computing.
- Returning the wrong type for comparison operators. `Int32 > Int32` returns `Boolean`, not `Int32`. Make sure `result_type` and `execute` agree.

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
5. Handle boolean operations (And, Or) as binary operations that work on Boolean vectors.
6. Handle NULL propagation -- if either operand is NULL, the arithmetic result should be NULL. Set the validity bit accordingly.
7. Watch out for vector length mismatches -- both sides of a binary operation must have the same length. Constants should be broadcast to match the chunk size.
8. Watch out for division by zero -- return an error or produce a null, depending on what the tests expect.
9. Test with nested expressions to make sure recursion works correctly end-to-end.

## Reading the Tests
- **`test_constant_expression`** creates a chunk and evaluates `Constant(42)`. It asserts the result value is `Int32(42)`. This confirms that constant expressions produce their literal value regardless of the input data. Your execute must create a vector of the right size.
- **`test_column_ref_expression`** builds a 2-row chunk with columns `[Int32, Int64]` and evaluates `ColumnRef(0)`. It checks both row values match the first column. This verifies that `ColumnRef(i)` extracts the correct column from the chunk.
- **`test_binary_add`** evaluates `ColumnRef(0) + ColumnRef(1)` on a chunk with rows `[10,5]` and `[20,3]`, expecting `[15, 23]`. This confirms element-wise addition across two columns.
- **`test_comparison_equal`** evaluates `ColumnRef(0) == Constant(42)` and expects `[true, false]`. This shows that comparison operators must return Boolean vectors, not numeric ones.
- **`test_boolean_and`** evaluates `ColumnRef(0) AND ColumnRef(1)` on Boolean columns. Row 0 is `(true, true) -> true`, row 1 is `(true, false) -> false`. This confirms AND must be implemented as a binary operation on Booleans.
- **`test_unary_is_null`** checks `IsNull` on a column with `[42, NULL]`, expecting `[false, true]`. This tests that your unary operator inspects the validity bitmap, not just the stored value.
- **`test_nested_expression`** evaluates `(a + b) * 2` and expects `(3+4)*2 = 14`. This validates that recursive execution evaluates inner expressions before outer ones.
- **`test_null_propagation`** adds `10 + NULL` and asserts the result is invalid (NULL). This is a critical SQL semantic: any arithmetic involving NULL must propagate NULL.
- **`test_expression_result_type`** checks that `Int32 + Int64` produces `Int64` (type promotion) and that `Equal` produces `Boolean`. This validates your `result_type` logic independent of execution.
