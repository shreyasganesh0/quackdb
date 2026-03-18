# Lesson 24: Physical Plan

## What You're Building
A translation layer that converts the logical plan tree into executable physical pipelines. The `PhysicalPlanBuilder` maps each logical node to a physical operator (or sequence of operators), while `PipelineBuilder` splits the plan at "pipeline breakers" -- operators like joins, aggregates, and sorts that need to consume all input before producing output. The `execute_plan()` function ties everything together for end-to-end SQL execution.

## Rust Concepts You'll Need
- [Trait Objects](../concepts/trait_objects.md) -- `Box<dyn DataSource>` and `Box<dyn PhysicalOperator>` allow heterogeneous operator types in a pipeline
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- the logical plan tree is traversed recursively to build flat pipeline sequences
- [Lifetimes](../concepts/lifetimes.md) -- `PhysicalPlanBuilder<'a>` borrows the catalog to look up table data during scan construction

## Key Patterns

### Converting a Tree to a Flat Pipeline
A logical plan is a tree, but execution often works as a chain of operators that pull data through. You walk the tree bottom-up, creating physical operators and deciding when a pipeline must be broken.

```rust
// Analogy: compiling a recipe tree into sequential cooking steps (NOT the QuackDB solution)
enum Recipe {
    RawIngredient(String),
    Chop(Box<Recipe>),
    Mix(Box<Recipe>, Box<Recipe>), // pipeline breaker: need both inputs ready
}

fn compile_steps(recipe: &Recipe, steps: &mut Vec<String>) {
    match recipe {
        Recipe::RawIngredient(name) => steps.push(format!("Fetch {}", name)),
        Recipe::Chop(inner) => {
            compile_steps(inner, steps);
            steps.push("Chop".to_string());
        }
        Recipe::Mix(left, right) => {
            // Pipeline breaker: must finish both sub-recipes first
            compile_steps(left, steps);
            compile_steps(right, steps);
            steps.push("Mix together".to_string());
        }
    }
}
```

### Pipeline Breaker Detection
Some operators are "blocking" -- they need all input before producing any output (e.g., sort needs all rows to find the minimum, aggregate needs all groups). When you encounter such a node, you end the current pipeline and start a new one.

```rust
// Analogy: a factory assembly line that must pause for quality inspection
fn is_blocking(step: &str) -> bool {
    matches!(step, "sort" | "aggregate" | "hash_build")
}

fn split_into_stages(steps: Vec<String>) -> Vec<Vec<String>> {
    let mut stages = vec![vec![]];
    for step in steps {
        if is_blocking(&step) {
            stages.push(vec![step]);
        } else {
            stages.last_mut().unwrap().push(step);
        }
    }
    stages
}
```

### Building a DataSource from Catalog Data
For a table scan, you fetch the stored `Vec<DataChunk>` from the catalog and wrap it in a struct that implements `DataSource`. If a projection is specified, you trim columns before returning.

## Step-by-Step Implementation Order
1. Start with `PhysicalPlanBuilder::build_scan()` -- look up the table in the catalog with `get_table_data()`, clone the chunks, apply column projection if present, and wrap in a `Box<dyn DataSource>`.
2. Implement `PhysicalPlanBuilder::build()` -- match on the logical plan. For `Scan`, call `build_scan`. For `Filter`, build the child pipeline and add a filter operator. For `Projection`, add a project operator. For `Sort`, `Aggregate`, `Join`, mark them as pipeline breakers.
3. Implement `PipelineBuilder::build()` -- walk the logical plan tree recursively. When you hit a pipeline breaker, materialize the child pipeline's output and feed it into the next pipeline as a new data source.
4. Implement `execute_plan()` -- build pipelines from the logical plan, execute each in order, and collect the final result chunks.
5. Watch out for operator ordering -- filters and projections are "streaming" (non-blocking), while sorts and aggregates are blocking. Getting this wrong causes incorrect results or panics.

## Reading the Tests
- **`test_e2e_select_all`** sets up a database with 4 rows and runs `SELECT * FROM users`, checking that 4 rows come back. This validates the full path: parse -> bind -> plan -> physical -> execute.
- **`test_e2e_where`** runs `SELECT * FROM users WHERE age > 28` and expects 2 rows (alice at 30 and charlie at 35). This confirms that filter operators correctly evaluate predicates against actual data values.
- **`test_e2e_join`** creates two tables and runs an INNER JOIN, expecting only matching rows. This tests that the pipeline breaker logic correctly materializes both sides before joining.

## What Comes Next
You now have a working end-to-end SQL database: parsing, binding, planning, and
executing queries. But the plans are naive — they execute whatever the parser
produces without optimization. Part VI introduces the **query optimizer**, which
rewrites logical plans into more efficient equivalents. Lesson 25 applies rule-based
transformations (predicate pushdown, constant folding), and Lesson 26 adds cost-based
optimization with join ordering. The `LogicalPlan` tree from L22 is exactly what the
optimizer transforms.
