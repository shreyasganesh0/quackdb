# Lesson 14: Pipelines

## What You're Building
A push-based execution pipeline that connects a data source, a chain of physical operators, and a data sink. The source produces chunks, each operator transforms them, and the sink collects the results. All components are accessed through trait objects (`Box<dyn DataSource>`, `Box<dyn PhysicalOperator>`, `&mut dyn DataSink`), enabling the engine to compose arbitrary query plans at runtime. This is the backbone of vectorized query execution.

## Concept Recap
Building on Lessons 10-13: You'll use `DataChunk` as the unit of data flowing through the pipeline, and `LogicalType` / `ScalarValue` for schema and value representation. The expressions from Lesson 13 will later be plugged into operators that use this pipeline infrastructure.

## Rust Concepts You'll Need
- [Trait Objects](../concepts/trait_objects.md) -- `Box<dyn DataSource>` and `Box<dyn PhysicalOperator>` provide runtime polymorphism
- [Traits and Derive](../concepts/traits_and_derive.md) -- defining the `PhysicalOperator`, `DataSource`, and `DataSink` traits
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- `Box<dyn Trait>` is needed because trait objects are unsized

## Key Patterns

### Trait Objects for Polymorphism
When you need a collection of different types that share behavior, trait objects are the tool. Think of it like a power strip -- each device plugged in is different, but they all conform to the same plug interface.

```rust
trait Filter {
    fn apply(&mut self, text: &str) -> String;
}

struct Uppercase;
impl Filter for Uppercase {
    fn apply(&mut self, text: &str) -> String {
        text.to_uppercase()
    }
}

struct Replace { from: String, to: String }
impl Filter for Replace {
    fn apply(&mut self, text: &str) -> String {
        text.replace(&self.from, &self.to)
    }
}

// A pipeline of heterogeneous filters:
let mut filters: Vec<Box<dyn Filter>> = vec![
    Box::new(Replace { from: "foo".into(), to: "bar".into() }),
    Box::new(Uppercase),
];

let mut result = "hello foo world".to_string();
for f in &mut filters {
    result = f.apply(&result);
}
assert_eq!(result, "HELLO BAR WORLD");
```

Your pipeline stores `Vec<Box<dyn PhysicalOperator>>` and chains them the same way -- each operator's output becomes the next operator's input.

### Push-Based Pipeline Model
In a push-based model, data flows forward from the source through each operator to the sink. Think of it like an assembly line in a factory -- each station receives a part, does its work, and passes it forward.

```
Source.next_chunk() --> Operator[0].execute(chunk) --> Operator[1].execute(chunk) --> Sink.consume(chunk)
```

The driver loop looks like:

```rust
fn run(
    source: &mut dyn Iterator<Item = Packet>,
    transforms: &mut [Box<dyn Transform>],
    output: &mut dyn Collector,
) {
    while let Some(packet) = source.next() {
        let mut current = packet;
        for t in transforms.iter_mut() {
            current = match t.process(current) {
                Some(p) => p,
                None => continue, // filtered out
            };
        }
        output.collect(current);
    }
}
```

Note the subtlety: an operator might return `NeedMoreInput` (it buffered the chunk but has no output yet) or `Output(chunk)` (it produced a result). Your loop must handle both cases. After the source is exhausted, call `finalize()` on each operator to flush any buffered output.

### Finalization Phase
Some operators (like aggregations) only produce output after all input is consumed. Think of it like a blender -- you add all the ingredients first, then blend once. After the source returns `None`, iterate through operators calling `finalize()` and push any resulting chunks through the remaining operators to the sink.

## Common Mistakes
- Forgetting to handle `NeedMoreInput` correctly. When an operator returns `NeedMoreInput`, you must NOT pass data to the next operator for that iteration -- just go back to the source for the next chunk.
- Not processing all chunks from the source. The pipeline loop must continue until `next_chunk()` returns `None`, not stop after the first chunk.
- Skipping the finalization cascade. When operator `i` finalizes and produces output, that output must still flow through operators `i+1..n` before reaching the sink.

## Step-by-Step Implementation Order
1. Start with `InMemorySource::new()` -- store the chunks, set position to 0, store the types.
2. Implement `DataSource for InMemorySource` -- `next_chunk` returns `self.chunks[self.position]` and increments position, or `None` when exhausted. Clone the chunk since the trait returns owned data.
3. Implement `DataSink for CollectSink` -- `consume` pushes the chunk into `self.chunks`.
4. Implement `Pipeline::new()` and `add_operator()` -- store the source and build up a `Vec<Box<dyn PhysicalOperator>>`.
5. Implement `Pipeline::execute()` -- this is the main driver:
   - Loop: pull a chunk from the source. If `None`, break.
   - Pass the chunk through each operator in sequence. Match on `OperatorResult`: if `Output(chunk)`, pass it to the next operator; if `NeedMoreInput`, stop the chain for this iteration; if `Finished`, stop.
   - When an operator produces output and it is the last operator, send the chunk to the sink.
   - After the source is exhausted, call `finalize()` on each operator. If finalize returns `Some(chunk)`, pass it through the remaining operators and into the sink.
   - Call `finalize()` on the sink.
6. Implement `PipelineExecutor::execute()` -- create a `CollectSink`, call `pipeline.execute(&mut sink)`, return the collected results.
7. Watch out for the operator chain semantics: if operator 0 returns `NeedMoreInput`, you should NOT call operator 1 for that iteration. Only pass data forward when you get `Output`.
8. Watch out for the finalization cascade: when operator `i` finalizes and produces output, that output must still pass through operators `i+1..n` before reaching the sink.
9. Verify that an empty source (no chunks) produces no output and does not panic.

## Reading the Tests
- **`test_pipeline_empty_source`** creates a source with zero chunks, builds a pipeline, and asserts the results are empty. This confirms your pipeline handles the degenerate case without errors -- the loop should just never execute.
- **`test_pipeline_passthrough`** adds a `PassthroughOperator` that returns `Output(input.slice(...))`. It asserts 3 rows come through. This validates the basic source-to-operator-to-sink data flow.
- **`test_pipeline_single_operator`** uses a `DoubleOperator` that multiplies each Int32 by 2. It checks values [2, 4, 6] from input [1, 2, 3]. This confirms that operator transformations are applied to every row in the chunk.
- **`test_pipeline_chain`** stacks two `DoubleOperator` instances. Input [5, 10] becomes [20, 40] (*2 *2). This validates that operators compose: the output of operator 0 feeds into operator 1.
- **`test_pipeline_multiple_chunks`** creates 3 chunks with 2 rows each and asserts all 6 rows reach the sink. This confirms the pipeline loop processes every chunk, not just the first.
- **`test_collect_sink`** directly calls `sink.consume(chunk)` and checks `results()` returns the chunk. This is a unit test for the sink in isolation -- it just accumulates chunks.
- **`test_inmemory_source`** calls `next_chunk()` three times on a 2-chunk source. First two calls return chunks, third returns `None`. This verifies your source correctly signals exhaustion.
