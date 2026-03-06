# Trait Objects

> **Prerequisites:** [traits_and_derive](./traits_and_derive.md), [box_and_recursive_types](./box_and_recursive_types.md)

## What This Is

When you write `fn process(item: &impl Summarizable)` in Rust, the compiler generates a
separate copy of the function for every concrete type that gets passed in. This is called
**static dispatch** (or monomorphization) -- it is fast because there is no runtime overhead,
but it means the concrete type must be known at compile time.

Sometimes you need to store or return values of *different* types that all implement the same
trait. For example, a query engine might have a `Vec` of operators where each operator is a
different struct. You cannot put a `Filter` and a `Projection` in the same `Vec<impl Operator>`
because each element would be a different type and therefore a different size. This is where
**trait objects** come in: `dyn Trait` tells Rust "I do not know the concrete type, but I know
it implements this trait." Combined with `Box`, you get `Box<dyn Trait>` -- a heap-allocated,
dynamically dispatched value, much like a virtual-method call in C++ or an interface reference
in Java.

If you come from Python or JavaScript, *all* method calls are dynamically dispatched (duck
typing). Rust makes you opt in explicitly with `dyn`, which gives you the performance of static
dispatch by default and dynamic dispatch only where you ask for it.

## Syntax

```rust
trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> &str;
}

struct Circle { radius: f64 }
struct Rect   { w: f64, h: f64 }

impl Shape for Circle {
    fn area(&self) -> f64 { std::f64::consts::PI * self.radius * self.radius }
    fn name(&self) -> &str { "circle" }
}
impl Shape for Rect {
    fn area(&self) -> f64 { self.w * self.h }
    fn name(&self) -> &str { "rect" }
}

// Static dispatch: compiler generates two versions of this function
fn print_area_static(s: &impl Shape) {
    println!("{}: {}", s.name(), s.area());
}

// Dynamic dispatch: one function, dispatches at runtime via vtable
fn print_area_dynamic(s: &dyn Shape) {
    println!("{}: {}", s.name(), s.area());
}

// Storing mixed types in a Vec
let shapes: Vec<Box<dyn Shape>> = vec![
    Box::new(Circle { radius: 3.0 }),
    Box::new(Rect { w: 4.0, h: 5.0 }),
];
for s in &shapes {
    println!("{} has area {:.2}", s.name(), s.area());
}
```

## Common Patterns

### Pattern 1: Heterogeneous Collections

A pipeline of processing stages where each stage is a different concrete type.

```rust
trait Stage {
    fn process(&self, input: &[u8]) -> Vec<u8>;
}

struct Compress;
struct Encrypt { key: u8 }

impl Stage for Compress {
    fn process(&self, input: &[u8]) -> Vec<u8> {
        // simplified: just remove zeros
        input.iter().copied().filter(|&b| b != 0).collect()
    }
}

impl Stage for Encrypt {
    fn process(&self, input: &[u8]) -> Vec<u8> {
        input.iter().map(|b| b ^ self.key).collect()
    }
}

fn run_pipeline(stages: &[Box<dyn Stage>], mut data: Vec<u8>) -> Vec<u8> {
    for stage in stages {
        data = stage.process(&data);
    }
    data
}

let pipeline: Vec<Box<dyn Stage>> = vec![
    Box::new(Compress),
    Box::new(Encrypt { key: 0xAB }),
];
let result = run_pipeline(&pipeline, vec![72, 0, 101, 0, 108]);
```

### Pattern 2: Returning Different Types from a Function

```rust
trait Logger {
    fn log(&self, msg: &str);
}

struct ConsoleLogger;
struct FileLogger { path: String }

impl Logger for ConsoleLogger {
    fn log(&self, msg: &str) { println!("[console] {}", msg); }
}
impl Logger for FileLogger {
    fn log(&self, msg: &str) { println!("[file:{}] {}", self.path, msg); }
}

fn create_logger(use_file: bool) -> Box<dyn Logger> {
    if use_file {
        Box::new(FileLogger { path: "/var/log/app.log".into() })
    } else {
        Box::new(ConsoleLogger)
    }
}
```

### Pattern 3: Trait Objects as Struct Fields

```rust
struct EventBus {
    listeners: Vec<Box<dyn Fn(&str)>>,   // closures as trait objects
}

impl EventBus {
    fn new() -> Self { EventBus { listeners: Vec::new() } }

    fn subscribe(&mut self, handler: Box<dyn Fn(&str)>) {
        self.listeners.push(handler);
    }

    fn emit(&self, event: &str) {
        for listener in &self.listeners {
            listener(event);
        }
    }
}
```

## Gotchas

1. **Object safety**: Not every trait can be used as `dyn Trait`. A trait is *object-safe* only
   if its methods do not use `Self` as a return type and do not have generic type parameters.
   For example, `Clone` is not object-safe because `fn clone(&self) -> Self` would require
   knowing the concrete type at the call site. If the compiler says "the trait `X` cannot be
   made into an object," check for `Self` in return position or generic methods.

2. **Performance cost**: Each method call on a `dyn Trait` goes through a vtable (a pointer
   lookup), similar to virtual calls in C++. This is usually negligible, but in a tight inner
   loop over millions of rows, it can matter. Profile before switching to static dispatch
   everywhere -- clarity often outweighs the nanoseconds.

3. **Cannot downcast without `Any`**: Once you erase the type behind `dyn Trait`, you cannot
   get the concrete type back unless the trait extends `std::any::Any`. This is different from
   Java where you can always cast down with `instanceof`. If you need downcasting, add
   `as_any()` helper methods or use the `downcast-rs` crate.

## Quick Reference

| Syntax                          | Meaning                                          |
|---------------------------------|--------------------------------------------------|
| `&dyn Trait`                    | Reference to a trait object (borrowed)           |
| `Box<dyn Trait>`                | Owned, heap-allocated trait object               |
| `Vec<Box<dyn Trait>>`           | Collection of mixed concrete types               |
| `&impl Trait`                   | Static dispatch (monomorphized) -- the alternative|
| `dyn Trait + Send`              | Trait object that is also `Send`                 |
| `dyn Trait + 'static`           | Trait object with no borrowed references inside  |

**Static vs Dynamic Dispatch:**

| Aspect          | `impl Trait` (static)               | `dyn Trait` (dynamic)                |
|-----------------|--------------------------------------|--------------------------------------|
| Type known at   | Compile time                         | Runtime                              |
| Binary size     | Larger (duplicated code)             | Smaller (single function)            |
| Call speed      | Direct call (fast)                   | Vtable lookup (tiny overhead)        |
| Heterogeneous   | No (one type per slot)               | Yes (mix types freely)               |
| C++ equivalent  | Templates                            | Virtual methods / `std::function`    |
| Python equiv.   | (not applicable)                     | Normal method call (duck typing)     |
