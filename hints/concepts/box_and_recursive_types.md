# Box and Recursive Types

> **Prerequisites:** [ownership_and_borrowing](./ownership_and_borrowing.md), [enums_and_matching](./enums_and_matching.md)

## Quick Reference
- `Box::new(value)` allocates `value` on the heap, returns a fixed-size pointer
- `*my_box` dereferences to access the inner value
- Recursive types need `Box` to break infinite size: `enum List { Cons(i32, Box<List>), Nil }`
- `Box<dyn Trait>` = heap-allocated trait object for dynamic dispatch
- `Box<T>` acts like `std::unique_ptr<T>` in C++ -- single owner, freed on drop

## Common Compiler Errors

**`error[E0072]: recursive type 'Node' has infinite size`**
Your struct or enum contains itself directly without indirection.
Fix: wrap the recursive field in `Box<>`: `children: Vec<Box<Node>>` or `next: Option<Box<Node>>`.

**`error[E0308]: mismatched types -- expected 'Box<T>', found 'T'`**
You forgot to wrap a value in `Box::new()`.
Fix: `let b: Box<i32> = Box::new(42);` -- Rust does not implicitly heap-allocate.

**`error[E0507]: cannot move out of 'Box<T>' which is behind a shared reference`**
You tried to move data out of a borrowed Box.
Fix: use `*my_box` only on an owned Box, or clone the inner value.

## When You'll Use This
- **Lesson 13 (Expressions):** `BinaryOp` contains `Box<Expression>` children
- **Lesson 19 (External Sort):** `Box<dyn Fn>` for runtime-polymorphic comparison closures
- **Lesson 21 (SQL Parser):** `Expr` contains `Box<Expr>` for binary ops; `TableRef::Join` has `Box<TableRef>`
- **Lesson 22 (Logical Plan):** plan nodes reference children via `Box<LogicalPlan>`
- **Lesson 27 (MVCC):** `prev_version: Option<Box<VersionedRow>>` forms a version chain
- **Lesson 32 (Distributed Plan):** recursive plan tree with `Box<LogicalPlan>` children

## What This Is

In Python and JavaScript, every object lives on the heap. You never think about it. In C++, you
choose between stack allocation (`MyStruct s;`) and heap allocation (`new MyStruct()`). Rust
defaults to the stack, which is fast but has a key limitation: the compiler must know the exact
size of every type at compile time.

`Box<T>` is Rust's simplest heap-allocation smart pointer. It puts a value of type `T` on the
heap and stores a fixed-size pointer on the stack. Think of it as C++'s `std::unique_ptr<T>`:
it owns the value, and when the `Box` is dropped, the heap memory is freed. There is no garbage
collector involved.

The most important use case for `Box` is **recursive types**. Consider a tree node that contains
children of the same type. Without `Box`, the struct would be infinitely large (a node contains
nodes which contain nodes...). By wrapping the recursive field in `Box`, each child is a
fixed-size pointer, and the compiler is satisfied. This pattern is essential for expression
trees, B-trees, query plan nodes, and any hierarchical data structure.

## Syntax

```rust
fn main() {
    // Basic Box usage
    let x: Box<i32> = Box::new(42);      // allocate 42 on the heap
    println!("{}", *x);                    // dereference to get the value: 42

    // Box owns the value; moving the Box moves ownership
    let y = x;                            // x is now invalid
    // println!("{}", *x);                // compile error: use of moved value

    // A recursive enum WITHOUT Box would not compile:
    // enum BadList { Cons(i32, BadList), Nil }  // ERROR: infinite size

    // Fixed with Box:
    enum List {
        Cons(i32, Box<List>),              // Box<List> is pointer-sized
        Nil,
    }

    let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));
}
```

## Common Patterns

### Pattern 1: Expression Trees

Compilers, query engines, and calculators all represent expressions as trees. Each node can
contain child expressions recursively.

```rust
enum Expr {
    Literal(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

fn eval(expr: &Expr) -> f64 {
    match expr {
        Expr::Literal(n) => *n,
        Expr::Add(a, b) => eval(a) + eval(b),
        Expr::Mul(a, b) => eval(a) * eval(b),
        Expr::Neg(e) => -eval(e),
    }
}

fn main() {
    // Build: (3 + 4) * -2
    let tree = Expr::Mul(
        Box::new(Expr::Add(
            Box::new(Expr::Literal(3.0)),
            Box::new(Expr::Literal(4.0)),
        )),
        Box::new(Expr::Neg(Box::new(Expr::Literal(2.0)))),
    );
    assert_eq!(eval(&tree), -14.0);
}
```

### Pattern 2: Recursive Structs for Tree Structures

```rust
struct BTreeNode {
    keys: Vec<i32>,
    // Children are heap-allocated because the struct is recursive
    children: Vec<Box<BTreeNode>>,
    is_leaf: bool,
}

impl BTreeNode {
    fn new_leaf(keys: Vec<i32>) -> Self {
        BTreeNode { keys, children: Vec::new(), is_leaf: true }
    }

    fn depth(&self) -> usize {
        if self.is_leaf {
            1
        } else {
            1 + self.children[0].depth()
        }
    }
}
```

### Pattern 3: Returning Different Types with `Box<dyn Trait>`

Sometimes you need `Box` not for recursion, but to erase a concrete type. This bridges into
trait objects (covered in the next concept).

```rust
use std::io::Read;

fn open_input(path: &str) -> Box<dyn Read> {
    if path == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(std::fs::File::open(path).expect("cannot open file"))
    }
}
```

## Gotchas

1. **Unnecessary Boxing**: Beginners coming from Java or Python sometimes `Box` everything out
   of habit. In Rust, prefer stack allocation. Only use `Box` when you need indirection
   (recursive types), dynamic dispatch (`Box<dyn Trait>`), or when a value is very large and
   you want to avoid expensive stack copies.

2. **Forgetting `Box::new()`**: You cannot assign a value directly to a `Box` variable. Writing
   `let b: Box<i32> = 42;` does not compile. You must write `let b: Box<i32> = Box::new(42);`.
   Rust does not implicitly heap-allocate.

3. **Deref coercion can be confusing**: `Box<T>` implements `Deref<Target = T>`, so you can
   call methods on the inner value directly: `my_box.some_method()`. This is convenient but can
   obscure what is happening. If you see method calls on a `Box` and wonder where the method
   is defined, check the inner type `T`.

## Related Concepts

- [Trait Objects](./trait_objects.md) -- `Box<dyn Trait>` combines boxing with dynamic dispatch
- [Enums and Matching](./enums_and_matching.md) -- recursive enums are the primary use case for Box
- [Ownership and Borrowing](./ownership_and_borrowing.md) -- Box provides single-owner heap allocation
- [Closures](./closures.md) -- `Box<dyn Fn(...)>` is used to store closures in structs

## Quick Reference

| Expression              | What It Does                                       |
|-------------------------|----------------------------------------------------|
| `Box::new(value)`       | Allocate `value` on the heap, return a `Box`       |
| `*my_box`               | Dereference to access the inner value              |
| `Box<List>` in an enum  | Break infinite-size recursion with indirection     |
| `Box<dyn Trait>`        | Heap-allocated trait object (dynamic dispatch)     |
| `let b = *my_box;`      | Move the value out of the Box (consumes the Box)   |

**When to use `Box<T>`:**

- Recursive types (trees, linked lists, ASTs)
- Large values you want on the heap to avoid stack overflow or expensive copies
- Trait objects (`Box<dyn Trait>`) for dynamic dispatch
- Transferring ownership of a value without copying it

**C++ / Python / JS equivalents:**

| Rust                | C++                       | Python / JS          |
|---------------------|---------------------------|----------------------|
| `Box<T>`            | `std::unique_ptr<T>`      | (all objects are heap)|
| `*my_box`           | `*my_ptr`                 | (no equivalent)      |
| `Box::new(val)`     | `std::make_unique<T>(val)`| `val` (already heap) |
