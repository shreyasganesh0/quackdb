# Ownership and Borrowing

> **Prerequisites:** None - this is a starting concept

## What This Is

Ownership is Rust's core memory management strategy. In Python, JavaScript, and Go, a garbage collector automatically frees memory when no references remain. In C and C++, you manually allocate and free memory (or use smart pointers). Rust takes a third path: every value has exactly one owner, and the value is dropped (freed) the moment that owner goes out of scope. There is no garbage collector, and there is no manual free.

This single-owner rule leads to "move semantics." When you assign a value from one variable to another -- `let b = a;` -- ownership transfers (moves) to `b`, and `a` is no longer valid. This is different from Python where `b = a` merely creates a second reference to the same object, and different from C++ where `b = a` copies by default (unless you use `std::move`). If you try to use `a` after the move, the Rust compiler rejects your code at compile time.

To let multiple parts of your code access a value without transferring ownership, Rust provides borrowing through references. A shared reference `&T` lets you read the value but not modify it, and you can have as many shared references as you want. A mutable reference `&mut T` lets you read and modify the value, but you can only have one mutable reference at a time, and no shared references can coexist with it. The borrow checker enforces these rules at compile time, eliminating data races and use-after-free bugs before your code ever runs.

## Syntax

```rust
fn main() {
    // --- Ownership and moves ---
    let name = String::from("Alice");   // `name` owns the String
    let greeting = name;                // ownership MOVES to `greeting`
    // println!("{}", name);            // COMPILE ERROR: `name` has been moved

    // --- Shared (immutable) borrowing ---
    let data = vec![1, 2, 3];
    let r1 = &data;                     // shared borrow
    let r2 = &data;                     // another shared borrow -- fine
    println!("{:?} {:?}", r1, r2);      // both references are valid
    // data.push(4);                    // COMPILE ERROR: cannot mutate while shared borrows exist

    // --- Mutable borrowing ---
    let mut scores = vec![10, 20];
    let m = &mut scores;                // mutable borrow
    m.push(30);                         // can modify through the mutable reference
    // let r = &scores;                 // COMPILE ERROR: cannot create shared borrow while mutable borrow exists
    println!("{:?}", m);                // last use of `m`
    // After `m` is no longer used, `scores` is free again
    println!("{:?}", scores);           // this works because `m`'s borrow has ended
}
```

### The ownership rules at a glance

```
1. Each value has exactly one owner.
2. When the owner goes out of scope, the value is dropped.
3. You can have EITHER:
       - any number of shared references (&T), OR
       - exactly one mutable reference (&mut T)
   ...but not both at the same time.
```

## Common Patterns

### Pattern 1: Functions that borrow vs. functions that take ownership

```rust
// This function borrows the vector -- caller keeps ownership
fn sum(values: &[i32]) -> i32 {
    values.iter().sum()
}

// This function takes ownership -- caller loses the vector
fn consume_and_report(values: Vec<i32>) -> String {
    let total: i32 = values.iter().sum();
    format!("Consumed {} items, total = {}", values.len(), total)
    // `values` is dropped here
}

fn main() {
    let nums = vec![1, 2, 3, 4, 5];
    let s = sum(&nums);            // borrow: `nums` is still ours
    println!("Sum: {}", s);
    println!("Still have nums: {:?}", nums);

    let report = consume_and_report(nums);  // move: `nums` is gone
    println!("{}", report);
    // println!("{:?}", nums);     // COMPILE ERROR: value moved
}
```

### Pattern 2: Returning owned data from functions

```rust
// Functions can create and return owned values -- ownership transfers to the caller
fn read_config(path: &str) -> String {
    // In real code this would read a file; simplified here
    format!("config from {}", path)
}

fn main() {
    let config = read_config("/etc/app.toml");  // caller now owns the String
    println!("{}", config);
}   // `config` is dropped here automatically
```

### Pattern 3: Clone when you need two independent copies

```rust
fn main() {
    let original = String::from("important data");

    // Clone creates a deep copy -- both variables own independent data
    let backup = original.clone();

    println!("original: {}", original);   // still valid
    println!("backup: {}", backup);       // independent copy
}
```

Note: `clone()` performs a deep copy and can be expensive for large data. Use it deliberately, not as a habit to silence the compiler.

## Gotchas

1. **Primitive types implement `Copy`, heap types do not.** Integers, floats, booleans, and chars are `Copy` -- assigning them creates a bitwise copy, and the original remains valid. But `String`, `Vec<T>`, and most heap-allocated types are moved instead. This is why `let x = 5; let y = x;` works fine, but `let s = String::from("hi"); let t = s;` moves `s`.

2. **Borrows must not outlive the owner.** If you create a reference to a value and then the value goes out of scope, the reference would dangle. Rust prevents this at compile time:
   ```rust
   fn dangling() -> &String {       // COMPILE ERROR
       let s = String::from("oops");
       &s                            // `s` is dropped at end of function
   }
   // Fix: return the owned String instead of a reference
   fn not_dangling() -> String {
       String::from("ok")
   }
   ```

3. **Mutable borrow "locks" the original variable.** A common surprise: while a `&mut` reference is active, you cannot use the original variable at all -- not even to read it. The lock ends at the last point where the mutable reference is used (this is called Non-Lexical Lifetimes, or NLL).

## Quick Reference

| Operation                 | Syntax              | Ownership effect                         |
|---------------------------|----------------------|------------------------------------------|
| Assign / pass by value    | `let b = a;`        | Moves `a` into `b` (for non-Copy types) |
| Shared borrow             | `let r = &a;`       | `a` is temporarily read-only             |
| Mutable borrow            | `let m = &mut a;`   | `a` is locked until `m` is done         |
| Dereference               | `*r`                 | Access the value behind a reference      |
| Clone                     | `a.clone()`          | Deep copy; both `a` and result are valid |
| Copy (implicit)           | `let b = a;`        | Bitwise copy for `Copy` types (i32, etc) |
| Slice borrow              | `&v[1..3]`          | Borrows a window into a Vec or array     |
| Returning owned value     | `fn f() -> String`  | Ownership transfers to caller            |
| Borrowing in function arg | `fn f(x: &str)`     | Caller retains ownership                 |
