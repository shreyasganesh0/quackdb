# Closures

> **Prerequisites:** [traits_and_derive](./traits_and_derive.md)

## What This Is

A closure is an anonymous function that can capture variables from its surrounding scope. If you
know Python's `lambda x: x + 1`, JavaScript's `(x) => x + 1`, or C++'s `[](int x){ return x + 1; }`,
you already understand the basic idea. Rust closures look like `|x| x + 1` -- vertical bars
around the parameters, then the body.

What makes Rust closures unique is how they interact with ownership. When a closure captures a
variable, the compiler decides whether it borrows the variable immutably, borrows it mutably,
or takes ownership of it. This decision is encoded in three traits: `Fn` (immutable borrow),
`FnMut` (mutable borrow), and `FnOnce` (takes ownership, can only be called once). Every
closure automatically implements the most permissive trait its captures allow. In Python and
JavaScript, closures always capture by reference with garbage collection handling cleanup; in
Rust, the borrow checker ensures safety at compile time.

Closures are used extensively in Rust: iterators (`.map()`, `.filter()`), thread spawning,
callbacks, custom sort comparators, and anywhere you want to pass behavior as a parameter. In a
database engine, closures appear in predicate evaluation, custom aggregation functions, and
visitor patterns over query plans.

## Syntax

```rust
// Basic closure
let add_one = |x: i32| x + 1;
assert_eq!(add_one(5), 6);

// Type annotations are optional when the compiler can infer types
let multiply = |a, b| a * b;
assert_eq!(multiply(3, 4), 12);

// Multi-line closure with a block body
let classify = |score: f64| {
    if score >= 90.0 { "A" }
    else if score >= 80.0 { "B" }
    else { "C" }
};

// Capturing from the environment
let threshold = 50;
let is_passing = |score: i32| score >= threshold;  // borrows `threshold`

// Mutable capture
let mut count = 0;
let mut increment = || {
    count += 1;    // mutably borrows `count`
    count
};
assert_eq!(increment(), 1);
assert_eq!(increment(), 2);

// Move capture: closure takes ownership
let name = String::from("Alice");
let greet = move || {
    println!("Hello, {}!", name);  // `name` is moved into the closure
};
greet();
// println!("{}", name);  // compile error: `name` was moved
```

## Common Patterns

### Pattern 1: Iterator Chains

Closures are the backbone of Rust's iterator combinators. This is the most common place
you will write closures.

```rust
let scores = vec![85, 92, 41, 73, 98, 55];

// Filter, transform, collect
let passing_grades: Vec<String> = scores
    .iter()
    .filter(|&&s| s >= 60)                    // Fn: immutable borrow
    .map(|&s| format!("Score: {}", s))        // Fn: immutable borrow
    .collect();

// fold (like Python's reduce)
let total: i32 = scores.iter().sum();

// sort_by with a closure
let mut items = vec![(1, "banana"), (3, "apple"), (2, "cherry")];
items.sort_by(|a, b| a.0.cmp(&b.0));   // sort by the numeric field
```

### Pattern 2: Closures as Function Parameters

Use `Fn`/`FnMut`/`FnOnce` trait bounds to accept closures as parameters.

```rust
// Accept any closure that takes an i32 and returns a bool
fn find_first<F>(data: &[i32], predicate: F) -> Option<i32>
where
    F: Fn(i32) -> bool,        // Fn: closure borrows its captures immutably
{
    for &item in data {
        if predicate(item) {
            return Some(item);
        }
    }
    None
}

let numbers = vec![10, 25, 30, 45, 50];
let threshold = 28;

// Pass a closure that captures `threshold`
let result = find_first(&numbers, |n| n > threshold);
assert_eq!(result, Some(30));

// FnMut example: closure that accumulates state
fn apply_to_each<F>(data: &[i32], mut action: F)
where
    F: FnMut(i32),             // FnMut: closure may mutate its captures
{
    for &item in data {
        action(item);
    }
}

let mut sum = 0;
apply_to_each(&[1, 2, 3], |x| sum += x);
assert_eq!(sum, 6);
```

### Pattern 3: Storing Closures with `Box<dyn Fn()>`

Because every closure has a unique anonymous type, you cannot name it. To store closures
in structs or collections, use trait objects.

```rust
type Callback = Box<dyn Fn(f64) -> f64>;

struct Transform {
    name: String,
    func: Callback,
}

let transforms: Vec<Transform> = vec![
    Transform {
        name: "double".into(),
        func: Box::new(|x| x * 2.0),
    },
    Transform {
        name: "square".into(),
        func: Box::new(|x| x * x),
    },
    Transform {
        name: "negate".into(),
        func: Box::new(|x| -x),
    },
];

let input = 3.0;
for t in &transforms {
    println!("{}: {} -> {}", t.name, input, (t.func)(input));
}
// double: 3 -> 6
// square: 3 -> 9
// negate: 3 -> -3
```

## Gotchas

1. **`Fn` vs `FnMut` vs `FnOnce` confusion**: If your closure mutates a captured variable, it
   implements `FnMut` but NOT `Fn`. If it moves a captured value out (consuming it), it
   implements only `FnOnce`. The hierarchy is: every `Fn` is also `FnMut`, and every `FnMut` is
   also `FnOnce`. When writing function signatures, use the *least restrictive* bound you can:
   prefer `Fn` over `FnMut` over `FnOnce`.

   ```rust
   // This WON'T compile because the closure mutates `count`:
   // fn run(f: impl Fn()) { f(); }
   // let mut count = 0;
   // run(|| count += 1);  // ERROR: cannot borrow `count` as mutable

   // Fix: use FnMut
   fn run(mut f: impl FnMut()) { f(); }
   ```

2. **`move` does not mean `FnOnce`**: The `move` keyword forces the closure to take ownership
   of captured variables, but this does NOT automatically make it `FnOnce`. A `move` closure
   that only reads its captured data is still `Fn`. `FnOnce` applies when the closure body
   *consumes* a captured value (e.g., drops it, returns it, passes ownership away).

3. **Returning closures from functions**: Because closures have anonymous types, you cannot
   write the return type directly. Use `impl Fn(...)` for static dispatch or `Box<dyn Fn(...)>`
   for dynamic dispatch:

   ```rust
   fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
       move |x| x + n   // must use `move` so the closure owns `n`
   }
   let add5 = make_adder(5);
   assert_eq!(add5(10), 15);
   ```

## Quick Reference

| Syntax                        | Meaning                                         |
|-------------------------------|-------------------------------------------------|
| `\|x\| x + 1`                | Closure taking one argument                     |
| `\|x, y\| x + y`             | Closure taking two arguments                    |
| `\|\| println!("hi")`        | Closure taking no arguments                     |
| `move \|\| { ... }`          | Force ownership of all captured variables       |
| `impl Fn(i32) -> i32`        | Accept any closure with this signature (static) |
| `Box<dyn Fn(i32) -> i32>`    | Heap-allocated closure (dynamic dispatch)       |

**The three closure traits:**

| Trait    | Captures           | Can Call   | Example Use               |
|----------|--------------------|------------|---------------------------|
| `Fn`     | `&self` (immutable)| Many times | `.filter()`, `.map()`     |
| `FnMut`  | `&mut self`        | Many times | `.for_each()`, accumulators|
| `FnOnce` | `self` (owned)     | Once only  | `thread::spawn()`, consumers|

**Language comparison:**

| Rust                       | Python                  | JavaScript              | C++                          |
|----------------------------|-------------------------|-------------------------|------------------------------|
| `\|x\| x + 1`             | `lambda x: x + 1`      | `(x) => x + 1`         | `[](int x){ return x+1; }`  |
| `move \|\| { ... }`       | (always by reference)   | (always by reference)   | `[=]{ ... }` (capture by copy)|
| `Fn` / `FnMut` / `FnOnce` | (no distinction)        | (no distinction)        | (no distinction)             |
