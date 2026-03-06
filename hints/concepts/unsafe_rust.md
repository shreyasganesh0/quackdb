# Unsafe Rust

> **Prerequisites:** [ownership_and_borrowing](./ownership_and_borrowing.md)

## What This Is

Rust's safety guarantees come from strict compile-time checks: the borrow checker, type system, and lifetime analysis. But some operations are fundamentally impossible to verify at compile time -- interfacing with hardware, calling C libraries, or manipulating raw memory layouts for performance. For these cases, Rust provides `unsafe` blocks and `unsafe` functions.

An `unsafe` block does not turn off all safety checks. The borrow checker, type checker, and lifetime rules still apply inside `unsafe`. What `unsafe` does is unlock exactly five additional capabilities: dereferencing raw pointers, calling unsafe functions, accessing mutable statics, implementing unsafe traits, and accessing fields of unions. Everything else -- bounds checking on arrays, ownership rules, type safety -- still works normally. Think of `unsafe` as the programmer saying "I have manually verified that this specific operation is correct; trust me here."

If you come from C or C++, `unsafe` Rust is close to what you are used to -- direct pointer manipulation with no safety net. The key difference is that in Rust, these dangerous operations are explicitly marked and isolated into small blocks. The convention is to wrap unsafe operations inside a safe API, so callers never need to write `unsafe` themselves. This pattern is sometimes called "safe abstraction over unsafe code." The standard library itself is full of this: `Vec`, `String`, `HashMap` all use unsafe internally but expose a safe interface.

## Syntax

```rust
fn main() {
    // --- Raw pointers ---
    let mut value: i32 = 42;

    // Creating raw pointers is safe -- dereferencing them is not
    let ptr: *const i32 = &value;        // immutable raw pointer
    let mut_ptr: *mut i32 = &mut value;  // mutable raw pointer

    // Dereferencing requires an unsafe block
    unsafe {
        println!("Read via raw pointer: {}", *ptr);
        *mut_ptr = 100;
        println!("After mutation: {}", *mut_ptr);
    }

    // --- Calling an unsafe function ---
    unsafe {
        dangerous_operation();
    }
}

// Declaring a function as unsafe -- callers must use an unsafe block
unsafe fn dangerous_operation() {
    // This function's contract cannot be verified by the compiler
    println!("Doing something the compiler can't check");
}
```

### Raw pointer types

| Pointer type   | Meaning                    | Comparable to (C/C++)  |
|----------------|----------------------------|------------------------|
| `*const T`     | Immutable raw pointer to T | `const T*`             |
| `*mut T`       | Mutable raw pointer to T   | `T*`                   |

Raw pointers differ from references (`&T` / `&mut T`) in several ways:
- They can be null
- They can dangle (point to freed memory)
- They can alias (multiple `*mut T` to the same location)
- They are not tracked by the borrow checker

## Common Patterns

### Pattern 1: Building a safe API over raw memory (slice from pointer)

This is extremely common when interfacing with C code or building data structures that manage their own memory.

```rust
/// Safely wraps a raw pointer + length into a Rust slice.
/// The caller guarantees that `ptr` points to `len` valid, initialized `f64` values.
///
/// # Safety
/// - `ptr` must be non-null and properly aligned
/// - `ptr` must point to `len` contiguous, initialized `f64` values
/// - The memory must not be mutated through other pointers for the lifetime `'a`
unsafe fn floats_from_raw<'a>(ptr: *const f64, len: usize) -> &'a [f64] {
    assert!(!ptr.is_null(), "null pointer passed to floats_from_raw");
    std::slice::from_raw_parts(ptr, len)
}

fn main() {
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let ptr = data.as_ptr();
    let len = data.len();

    // SAFETY: `ptr` and `len` come directly from a live Vec, so all requirements are met.
    let slice = unsafe { floats_from_raw(ptr, len) };
    println!("Sum: {}", slice.iter().sum::<f64>());
}
```

### Pattern 2: Calling a C function via FFI

```rust
// Declare an external C function
extern "C" {
    fn abs(input: i32) -> i32;
    fn strlen(s: *const std::ffi::c_char) -> usize;
}

fn safe_abs(x: i32) -> i32 {
    // SAFETY: abs() is a pure function with no preconditions
    unsafe { abs(x) }
}

fn main() {
    println!("|-42| = {}", safe_abs(-42));

    let c_string = std::ffi::CString::new("hello").unwrap();
    // SAFETY: c_string is a valid null-terminated C string
    let len = unsafe { strlen(c_string.as_ptr()) };
    println!("strlen: {}", len);
}
```

### Pattern 3: Reinterpreting bytes as a typed value

```rust
/// Reads a little-endian u32 from a byte slice without bounds-checking overhead.
///
/// # Safety
/// `bytes` must have at least 4 elements.
unsafe fn read_u32_unchecked(bytes: &[u8]) -> u32 {
    let ptr = bytes.as_ptr() as *const u32;
    // SAFETY: caller guarantees at least 4 bytes; we assume little-endian platform
    ptr.read_unaligned()
}

fn read_u32_safe(bytes: &[u8]) -> Option<u32> {
    if bytes.len() < 4 {
        return None;
    }
    // SAFETY: we just checked that there are at least 4 bytes
    Some(unsafe { read_u32_unchecked(bytes) })
}

fn main() {
    let buffer: &[u8] = &[0x01, 0x00, 0x00, 0x00, 0xFF];
    match read_u32_safe(buffer) {
        Some(val) => println!("Read value: {}", val),
        None => println!("Buffer too short"),
    }
}
```

## Gotchas

1. **`unsafe` does not disable the borrow checker.** A common misconception is that `unsafe` lets you bypass all Rust rules. It does not. Ownership, lifetimes, and type checking all still apply. If you want to have two mutable references to the same data, you need raw pointers, not just an `unsafe` block around normal references.

2. **The `// SAFETY:` comment is a social contract, not a compiler check.** When you write `unsafe { ... }`, you are making a promise that certain invariants hold. If those invariants are wrong, you get undefined behavior -- not a compiler error, not a runtime panic, but silent corruption, crashes, or worse. Always document your safety argument with a `// SAFETY:` comment. Reviewers (and your future self) depend on it.

3. **Undefined behavior can "time travel."** Unlike a segfault in C that crashes at the bad line, UB in Rust (just like in C/C++) can cause the compiler to make optimizations that break code far away from the actual bug. For example, creating an invalid `bool` (a byte that is neither 0 nor 1) via unsafe can cause `match` statements elsewhere to take impossible branches. If you trigger UB, the entire program's behavior is unpredictable.

## Quick Reference

| Capability unlocked by `unsafe`           | Example                                      |
|-------------------------------------------|----------------------------------------------|
| Dereference a raw pointer                 | `unsafe { *ptr }`                            |
| Call an `unsafe fn`                       | `unsafe { from_raw_parts(ptr, len) }`        |
| Access a mutable static variable          | `unsafe { GLOBAL_COUNT += 1; }`              |
| Implement an unsafe trait                 | `unsafe impl Send for MyType {}`             |
| Access union fields                       | `unsafe { my_union.int_field }`              |

| Safe operation (no `unsafe` needed)       | Example                                      |
|-------------------------------------------|----------------------------------------------|
| Create a raw pointer                      | `let p: *const i32 = &x;`                   |
| Cast between raw pointer types            | `ptr as *const u8`                           |
| Compare raw pointers                      | `ptr1 == ptr2`                               |
| Print a raw pointer                       | `println!("{:p}", ptr)`                      |
| Convert reference to raw pointer          | `&x as *const _`                             |

**Rule of thumb:** Keep `unsafe` blocks as small as possible. Wrap them in safe functions. Document every safety invariant. Test aggressively with Miri (`cargo +nightly miri test`) to catch UB.
