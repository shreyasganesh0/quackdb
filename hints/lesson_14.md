# Lesson 14: Pipelines

## What You're Building
A push-based execution pipeline that connects a data source, a chain of physical operators, and a data sink. The source produces chunks, each operator transforms them, and the sink collects the results. All components are accessed through trait objects (`Box<dyn DataSource>`, `Box<dyn PhysicalOperator>`, `&mut dyn DataSink`), enabling the engine to compose arbitrary query plans at runtime. This is the backbone of vectorized query execution.

## Rust Concepts You'll Need
- [Trait Objects](../concepts/trait_objects.md) -- `Box<dyn DataSource>` and `Box<dyn PhysicalOperator>` provide runtime polymorphism
- [Traits and Derive](../concepts/traits_and_derive.md) -- defining the `PhysicalOperator`, `DataSource`, and `DataSink` traits
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- `Box<dyn Trait>` is needed because trait objects are unsized

## Key Patterns

### Trait Objects for Polymorphism
When you need a collection of different types that share behavior, trait objects are the tool:

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
In a push-based model, data flows forward from the source through each operator to the sink:

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
Some operators (like aggregations) only produce output after all input is consumed. After the source returns `None`, iterate through operators calling `finalize()` and push any resulting chunks through the remaining operators to the sink.

## Step-by-Step Implementation Order
1. Start with `InMemorySource::new()` -- store the chunks, set position to 0, store the types.
2. Implement `DataSource for InMemorySource` -- `next_chunk` returns `self.chunks[self.position]` and increments position, or `None` when exhausted. Clone the chunk since the trait returns owned data.
3. Implement `DataSink for CollectSink` -- `consume` pushes the chunk into `self.chunks`.
4. Implement `Pipeline::execute()` -- this is the main driver:
   - Initialize all operators by calling `init()`.
   - Loop: pull a chunk from the source. If `None`, break.
   - Pass the chunk through each operator in sequence. Match on `OperatorResult`: if `Output(chunk)`, pass it to the next operator; if `NeedMoreInput`, stop the chain for this iteration; if `Finished`, stop.
   - When an operator produces output and it is the last operator, send the chunk to the sink.
   - After the source is exhausted, call `finalize()` on each operator. If finalize returns `Some(chunk)`, pass it through the remaining operators and into the sink.
   - Call `finalize()` on the sink.
5. Implement `PipelineExecutor::execute()` -- create a `CollectSink`, call `pipeline.execute(&mut sink)`, return the collected results.
6. Watch out for the operator chain semantics: if operator 0 returns `NeedMoreInput`, you should NOT call operator 1 for that iteration. Only pass data forward when you get `Output`.
7. Watch out for the finalization cascade: when operator `i` finalizes and produces output, that output must still pass through operators `i+1..n` before reaching the sink.

## Reading the Tests
- Look for a test that creates an `InMemorySource` with several chunks, builds a `Pipeline` with no operators, executes it into a `CollectSink`, and asserts the sink collected all original chunks. This validates the basic source-to-sink pass-through.
- Look for a test that adds one or more operators (like a filter or projection) and checks that the output chunks are transformed correctly. Pay attention to how the test constructs `Box<dyn PhysicalOperator>` -- that reveals which concrete operator types you may need to interact with.
