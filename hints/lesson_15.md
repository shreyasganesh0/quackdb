# Lesson 15: Scan, Filter, Projection

## What You're Building
Three fundamental execution operators that form the backbone of query processing. TableScanOperator reads data from any source with optional column pruning. FilterOperator evaluates a predicate expression and keeps only matching rows. ProjectionOperator computes a list of output expressions (column references, arithmetic, etc.) to reshape each chunk. Together they form the classic Scan-Filter-Project pipeline that most SQL queries reduce to.

## Rust Concepts You'll Need
- [Traits and Derive](../concepts/traits_and_derive.md) -- all three operators implement PhysicalOperator, each providing its own `execute()` and `output_schema()`
- [Trait Objects](../concepts/trait_objects.md) -- TableScanOperator holds `Box<dyn DataSource>`, and the pipeline stores operators as `Box<dyn PhysicalOperator>`

## Key Patterns

### Implementing a Trait for Multiple Struct Types
When several structs share a common interface, define a trait and implement it for each. The caller works through the trait, not the concrete type.

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
A pipeline feeds the output of one operator into the next. Each operator transforms data and returns a result indicating whether more input is needed.

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

## Step-by-Step Implementation Order
1. Start with `TableScanOperator::new()` -- store the `Box<dyn DataSource>` and projection indices; derive `output_types` from the source schema filtered by projection
2. Implement `TableScanOperator::next_chunk()` -- call the source to get the next chunk, then select only the projected columns if a projection is specified
3. Implement `FilterOperator::evaluate()` -- evaluate the predicate Expression on each row of the chunk, collect the indices where the result is true into a SelectionVector; treat NULL predicate results as false
4. Implement `FilterOperator::execute()` -- call evaluate, then apply the SelectionVector to produce a filtered chunk
5. Implement `ProjectionOperator::execute()` -- evaluate each Expression in `self.expressions` against the input chunk, assemble the resulting vectors into a new DataChunk
6. Watch out for empty results: if the filter removes all rows, return an empty chunk, not an error

## Reading the Tests
- **`test_filter_range`** builds a compound predicate `id > 2 AND id < 5` using nested `Expression::BinaryOp` with `Box`. It expects exactly 2 rows out of 5. This shows that your filter must recursively evaluate expressions and treat AND as a logical combinator.
- **`test_filter_then_project`** chains a FilterOperator and a ProjectionOperator in the same pipeline. It filters `id >= 3` (keeping 3 rows), then projects only column 1. The assertions check both row count and the output column type, confirming that chained operators compose correctly.
