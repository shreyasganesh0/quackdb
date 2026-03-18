# Lesson 15: Scan, Filter, Projection

## What You're Building
Three fundamental execution operators that form the backbone of query processing. TableScanOperator reads data from any source with optional column pruning. FilterOperator evaluates a predicate expression and keeps only matching rows. ProjectionOperator computes a list of output expressions (column references, arithmetic, etc.) to reshape each chunk. Together they form the classic Scan-Filter-Project pipeline that most SQL queries reduce to.

## Concept Recap
Building on Lessons 13-14: You'll use `Expression` and `ExpressionExecutor` from Lesson 13 to evaluate predicates and projection expressions. These operators plug into the `Pipeline` framework from Lesson 14 as `Box<dyn PhysicalOperator>`, using `InMemorySource` and `PipelineExecutor` to drive execution.

## Rust Concepts You'll Need
- [Traits and Derive](../concepts/traits_and_derive.md) -- all three operators implement PhysicalOperator, each providing its own `execute()` and `output_schema()`
- [Trait Objects](../concepts/trait_objects.md) -- TableScanOperator holds `Box<dyn DataSource>`, and the pipeline stores operators as `Box<dyn PhysicalOperator>`

## Key Patterns

### Implementing a Trait for Multiple Struct Types
When several structs share a common interface, define a trait and implement it for each. Think of it like different kitchen appliances -- a blender, toaster, and microwave all plug into the same outlet (trait), but each does something different inside.

```rust
// Analogy: different shape renderers (NOT the QuackDB solution)
trait Renderer {
    fn render(&self, canvas: &mut Canvas) -> Result<(), String>;
    fn name(&self) -> &str;
}

struct CircleRenderer { radius: f64 }
struct RectRenderer { width: f64, height: f64 }

impl Renderer for CircleRenderer {
    fn render(&self, canvas: &mut Canvas) -> Result<(), String> {
        canvas.draw_circle(self.radius)
    }
    fn name(&self) -> &str { "Circle" }
}

impl Renderer for RectRenderer {
    fn render(&self, canvas: &mut Canvas) -> Result<(), String> {
        canvas.draw_rect(self.width, self.height)
    }
    fn name(&self) -> &str { "Rect" }
}

// Caller uses trait objects:
let pipeline: Vec<Box<dyn Renderer>> = vec![
    Box::new(CircleRenderer { radius: 5.0 }),
    Box::new(RectRenderer { width: 10.0, height: 3.0 }),
];
```

### Operator Chaining via Trait Objects
A pipeline feeds the output of one operator into the next. Think of it like a water treatment plant -- raw water flows through filtration, then chemical treatment, then UV disinfection, each stage cleaning it further.

```rust
// Analogy: an image processing pipeline (NOT the QuackDB solution)
trait ImageFilter {
    fn apply(&mut self, pixels: &[u8]) -> Result<Vec<u8>, String>;
}

fn run_pipeline(input: &[u8], filters: &mut [Box<dyn ImageFilter>]) -> Result<Vec<u8>, String> {
    let mut data = input.to_vec();
    for filter in filters.iter_mut() {
        data = filter.apply(&data)?;
    }
    Ok(data)
}
```

### Predicate Evaluation as Row Selection
A filter evaluates a boolean expression for each row. Rows where the predicate is true (or not NULL) are kept; all others are discarded. Think of it like a bouncer at a club -- each person is checked against the criteria, and only those who pass get through.

## Common Mistakes
- Treating NULL predicate results as true. In SQL, `NULL > 5` is NULL (falsy), so those rows must be filtered out. Only rows where the predicate evaluates to `Boolean(true)` should pass.
- Returning an error when the filter removes all rows. An empty chunk is a valid result, not an error condition.
- Forgetting that ProjectionOperator can compute arbitrary expressions, not just column references. `SELECT col0 * 10` requires expression evaluation, not just column extraction.

## Step-by-Step Implementation Order
1. Start with `TableScanOperator::new()` -- store the `Box<dyn DataSource>` and projection indices; derive `output_types` from the source schema filtered by projection
2. Implement `TableScanOperator::next_chunk()` -- call the source to get the next chunk, then select only the projected columns if a projection is specified
3. Implement `FilterOperator::new()` -- store the predicate Expression
4. Implement `FilterOperator::execute()` -- evaluate the predicate Expression on the input chunk using `ExpressionExecutor`, collect the indices where the result is `Boolean(true)`, build a new chunk containing only matching rows; treat NULL predicate results as false
5. Implement `ProjectionOperator::new()` -- store the expressions list and output types
6. Implement `ProjectionOperator::execute()` -- evaluate each Expression in `self.expressions` against the input chunk, assemble the resulting vectors into a new DataChunk
7. Watch out for empty results: if the filter removes all rows, return an empty chunk, not an error
8. Verify that filter-then-project chains work: the projection must operate on the already-filtered chunk
9. Test with NULL values in predicate columns to ensure they are filtered out correctly

## Reading the Tests
- **`test_filter_equality`** builds predicate `id == 3` and filters 5 rows. Only 1 row matches, so `total == 1`. This validates basic equality filtering and confirms that the filter keeps exactly the rows that satisfy the predicate.
- **`test_filter_range`** builds a compound predicate `id > 2 AND id < 5` using nested `Expression::BinaryOp` with `Box`. It expects exactly 2 rows out of 5 (rows with id 3 and 4). This shows that your filter must recursively evaluate expressions and treat AND as a logical combinator.
- **`test_filter_no_matches`** filters with `id > 100` on data where max id is 5. Expects 0 rows. This confirms your operator handles the all-filtered-out case gracefully without errors.
- **`test_projection_columns`** projects `[ColumnRef(2), ColumnRef(0)]` and checks the output has 2 columns with the values reordered. This confirms projection can reorder columns, not just select a subset.
- **`test_projection_expression`** projects `col0 * 10` and checks the first row is 10 (1*10) and the fifth is 50 (5*10). This validates that projection evaluates arbitrary expressions, not just column pass-through.
- **`test_filter_then_project`** chains a FilterOperator (`id >= 3`) and a ProjectionOperator (select column 1). It asserts 3 surviving rows and checks the output type is `Int64`. This confirms that chained operators compose correctly in a pipeline.
- **`test_filter_with_nulls`** filters `col > 5` on data `[10, NULL, 30]`. Only rows with 10 and 30 pass (2 rows). The NULL row is excluded because `NULL > 5` evaluates to NULL, which is falsy in SQL.
