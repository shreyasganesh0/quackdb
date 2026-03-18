# Lesson 24: Physical Plan

## What You're Building
A translation layer that converts the logical plan tree into executable physical pipelines. The `PhysicalPlanBuilder` maps each logical node to a physical operator (or sequence of operators), while `PipelineBuilder` splits the plan at "pipeline breakers" -- operators like joins, aggregates, and sorts that need to consume all input before producing output. The `execute_plan()` function ties everything together for end-to-end SQL execution.

## Concept Recap
Building on Lessons 13-23: This is the capstone lesson that connects the entire stack. The logical plan from L22 (produced by the binder in L23 from the parsed AST in L21) is converted into the physical operators you built in Lessons 13-19: `FilterOperator`, `ProjectionOperator`, `ExternalSortOperator`, `HashAggregateOperator`, `HashJoinOperator`, all running inside the `Pipeline` framework from L14. The catalog provides table data for scan operators.

## Rust Concepts You'll Need
- [Trait Objects](../concepts/trait_objects.md) -- `Box<dyn DataSource>` and `Box<dyn PhysicalOperator>` allow heterogeneous operator types in a pipeline
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- the logical plan tree is traversed recursively to build flat pipeline sequences
- [Lifetimes](../concepts/lifetimes.md) -- `PhysicalPlanBuilder<'a>` borrows the catalog to look up table data during scan construction

## Key Patterns

### Converting a Tree to a Flat Pipeline
A logical plan is a tree, but execution often works as a chain of operators that pull data through. Think of it like turning a recipe (a tree of dependencies) into a step-by-step cooking instruction list -- you need to figure out the right order and identify where you must wait for something to finish before continuing.

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
Some operators are "blocking" -- they need all input before producing any output (e.g., sort needs all rows to find the minimum, aggregate needs all groups). Think of it like a traffic signal on a highway -- streaming operators are the open road, but blocking operators are red lights where traffic must accumulate before being released in a batch.

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
For a table scan, you fetch the stored `Vec<DataChunk>` from the catalog and wrap it in a struct that implements `DataSource`. Think of it like fetching a filing cabinet's contents and placing them on the conveyor belt -- the physical scan operator translates the catalog's stored data into the pipeline's streaming data format.

## Common Mistakes
- Not handling the pipeline breaker correctly for joins. A join has two child plans (left and right), and both must be fully materialized before the join can begin. This means executing one or two sub-pipelines to completion before starting the join pipeline.
- Getting the operator ordering wrong when converting a plan tree to a pipeline. Filters and projections are streaming (added to the current pipeline), while sorts and aggregates break the pipeline. If you add a sort as a streaming operator, it will try to sort one chunk at a time instead of all data.
- Forgetting to convert `LogicalExpr` to `Expression`. The logical plan uses `LogicalExpr` (with column names), but the physical operators use `Expression` (with column indices). The physical plan builder must translate between the two.

## Step-by-Step Implementation Order
1. Start with `PhysicalPlanBuilder::build_scan()` -- look up the table in the catalog with `get_table_data()`, clone the chunks, apply column projection if present, and wrap in a `Box<dyn DataSource>`.
2. Implement `PhysicalPlanBuilder::build()` -- match on the logical plan. For `Scan`, call `build_scan`. For `Filter`, build the child pipeline and add a filter operator. For `Projection`, add a project operator. For `Sort`, `Aggregate`, `Join`, mark them as pipeline breakers.
3. Implement logical-to-physical expression conversion -- translate `LogicalExpr::ColumnRef { index, .. }` to `Expression::ColumnRef(index)`, `LogicalExpr::Literal` to `Expression::Constant`, and recursively convert binary/unary operations.
4. Implement `PipelineBuilder::build()` -- walk the logical plan tree recursively. When you hit a pipeline breaker, materialize the child pipeline's output and feed it into the next pipeline as a new data source.
5. Implement `execute_plan()` -- build pipelines from the logical plan, execute each in order, and collect the final result chunks.
6. Implement the `Database` struct that ties everything together: parse SQL, bind against catalog, build physical plan, execute.
7. Handle DDL statements (CREATE TABLE, INSERT) separately from queries -- they modify the catalog rather than producing query results.
8. Watch out for operator ordering -- filters and projections are "streaming" (non-blocking), while sorts and aggregates are blocking. Getting this wrong causes incorrect results or panics.

## Reading the Tests
- **`test_e2e_select_all`** sets up a database with 4 rows and runs `SELECT * FROM users`, checking that 4 rows come back. This validates the full path: parse, bind, plan, physical plan, execute. It is the simplest end-to-end test.
- **`test_e2e_select_columns`** runs `SELECT name, age FROM users` and checks 4 rows with 2 columns. This confirms that column projection is correctly translated from SQL to physical operators.
- **`test_e2e_where`** runs `SELECT * FROM users WHERE age > 28` and expects 2 rows (alice at 30 and charlie at 35). This confirms that filter operators correctly evaluate predicates against actual data values from the catalog.
- **`test_e2e_order_by`** runs `SELECT name FROM users ORDER BY age ASC` and checks that "bob" (age 25, youngest) is first. This validates that the sort operator is correctly wired as a pipeline breaker and produces ordered output.
- **`test_e2e_limit`** runs `SELECT * FROM users LIMIT 2` and expects 2 rows. This confirms that LIMIT correctly caps the output row count.
- **`test_e2e_group_by`** creates a sales table and runs `SELECT product, SUM(amount) FROM sales GROUP BY product`. It expects 2 groups. This validates the hash aggregate pipeline breaker and GROUP BY handling end-to-end.
- **`test_e2e_join`** creates two tables and runs an INNER JOIN, expecting 1 matching row. This tests that the pipeline breaker logic correctly materializes both sides before joining, and that the join condition is evaluated correctly.
- **`test_e2e_create_and_insert`** runs CREATE TABLE, INSERT, then SELECT and checks the inserted value comes back. This validates the DDL/DML path: catalog modification followed by query execution.
- **`test_e2e_expression_in_select`** runs `SELECT age * 2 FROM users WHERE id = 1` and expects 60 (30*2). This validates that computed expressions in the SELECT list are correctly translated to physical projection operators.
- **`test_database_default`** checks that `Database::default()` has an empty catalog. This is a simple constructor test.

## What Comes Next
You now have a working end-to-end SQL database: parsing, binding, planning, and
executing queries. But the plans are naive -- they execute whatever the parser
produces without optimization. Part VI introduces the **query optimizer**, which
rewrites logical plans into more efficient equivalents. Lesson 25 applies rule-based
transformations (predicate pushdown, constant folding), and Lesson 26 adds cost-based
optimization with join ordering. The `LogicalPlan` tree from L22 is exactly what the
optimizer transforms.
