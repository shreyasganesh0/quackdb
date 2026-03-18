# Concurrency

> **Prerequisites:** [ownership_and_borrowing](./ownership_and_borrowing.md), [trait_objects](./trait_objects.md)

## Quick Reference
- `thread::spawn(move || { ... })` spawns an OS thread; closure must be `'static + Send`
- `Arc<T>` = thread-safe shared ownership (atomic reference counting)
- `Mutex<T>` = mutual exclusion; wraps the data, not code sections
- `Arc::clone(&x)` clones the pointer, not the data
- `AtomicU64` / `AtomicBool` = lock-free atomic operations for counters and flags

## Common Compiler Errors

**`error[E0277]: 'Rc<T>' cannot be sent between threads safely`**
You used `Rc` instead of `Arc` for cross-thread sharing.
Fix: replace `Rc<T>` with `Arc<T>`. `Rc` uses non-atomic reference counts and is not `Send`.

**`error[E0373]: closure may outlive the current function`**
The closure borrows local variables but the thread may outlive the function.
Fix: add `move` before the closure to take ownership of captured variables.

**`error[E0277]: 'MutexGuard' cannot be sent between threads safely`**
You tried to hold a `MutexGuard` across a `.await` or send it to another thread.
Fix: drop the guard before the await point or thread boundary. Use a shorter critical section.

## When You'll Use This
- **Lesson 27 (MVCC):** `AtomicU64` with `Ordering::SeqCst` for thread-safe transaction ID generation
- **Lesson 29 (Morsel-Parallel):** `Arc`, `Mutex`, `thread::spawn`, `Send`/`Sync` bounds for parallel execution
- **Lesson 33 (Shuffle Exchange):** `mpsc::channel` for message passing between threads

## What This Is

Concurrency in Rust is built on a simple but powerful idea: the ownership system that prevents
data races at compile time. If you come from Python, you know the GIL (Global Interpreter Lock)
effectively makes threads useless for CPU-bound work. In Java and C++, threads can freely share
mutable state, and data races are your problem to debug at 3 AM. Rust takes a middle path:
threads are real OS threads with true parallelism (no GIL), but the compiler enforces that
shared data is either immutable or protected by synchronization primitives.

The key building blocks are `Arc<T>` (atomic reference counting for shared ownership across
threads), `Mutex<T>` (mutual exclusion lock that wraps the data it protects), and atomic types
like `AtomicU64` (lock-free shared counters). Two marker traits make this work: `Send` means a
value can be transferred to another thread, and `Sync` means a value can be referenced from
multiple threads simultaneously. Most types are `Send + Sync` automatically; the compiler will
stop you if you try to share something that is not safe.

For message-passing concurrency (like Go channels or Erlang mailboxes), Rust provides
`std::sync::mpsc` (multiple-producer, single-consumer channels). This lets threads communicate
by sending values rather than sharing memory. In a database engine, concurrency appears
everywhere: parallel query execution, concurrent buffer pool access, background flushing, and
write-ahead log appenders.

## Syntax

```rust
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    // Spawning a thread -- the closure must be 'static + Send
    let handle = thread::spawn(|| {
        println!("Hello from a thread!");
        42  // return value
    });
    let result = handle.join().unwrap();  // wait for thread, get return value
    assert_eq!(result, 42);

    // Sharing data with Arc + Mutex
    let counter = Arc::new(Mutex::new(0));

    let mut handles = vec![];
    for _ in 0..4 {
        let counter = Arc::clone(&counter);   // clone the Arc, not the data
        handles.push(thread::spawn(move || {
            let mut num = counter.lock().unwrap();  // lock returns MutexGuard
            *num += 1;
            // lock is automatically released when MutexGuard is dropped
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(*counter.lock().unwrap(), 4);
}
```

## Common Patterns

### Pattern 1: Parallel Aggregation with Atomics

When you only need to increment a counter or accumulate a sum, atomics are faster than a Mutex
because they avoid kernel-level locking.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

fn parallel_count(data: &[Vec<i32>], threshold: i32) -> u64 {
    let count = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    for chunk in data {
        let count = Arc::clone(&count);
        let chunk = chunk.clone();
        handles.push(thread::spawn(move || {
            let local_count = chunk.iter().filter(|&&x| x > threshold).count() as u64;
            count.fetch_add(local_count, Ordering::Relaxed);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
    count.load(Ordering::Relaxed)
}

fn main() {
    let data = vec![vec![1, 5, 10], vec![3, 7, 12], vec![2, 8, 15]];
    assert_eq!(parallel_count(&data, 6), 5);  // 10, 7, 12, 8, 15
}
```

### Pattern 2: Message Passing with `mpsc` Channels

Channels let threads communicate without shared state. This is the Rust equivalent of Go
channels or Python's `queue.Queue`.

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

enum WorkItem {
    Task(String),
    Shutdown,
}

fn worker_pool_example() {
    let (tx, rx) = mpsc::channel();

    // Spawn a worker that processes messages
    let worker = thread::spawn(move || {
        loop {
            match rx.recv().unwrap() {
                WorkItem::Task(name) => {
                    println!("Processing: {}", name);
                    thread::sleep(Duration::from_millis(10));
                }
                WorkItem::Shutdown => {
                    println!("Worker shutting down");
                    break;
                }
            }
        }
    });

    // Send tasks from the main thread
    tx.send(WorkItem::Task("index_page_42".into())).unwrap();
    tx.send(WorkItem::Task("flush_buffer_7".into())).unwrap();
    tx.send(WorkItem::Shutdown).unwrap();

    worker.join().unwrap();
}
```

### Pattern 3: Shared Read-Heavy Data with `Arc` (No Mutex Needed)

If data is immutable after construction, `Arc<T>` alone is sufficient. No Mutex is needed
because `&T` is `Sync` when `T` is `Sync` (which most types are).

```rust
use std::sync::Arc;
use std::thread;
use std::collections::HashMap;

fn parallel_lookup() {
    // Build a lookup table once
    let table: Arc<HashMap<String, f64>> = Arc::new(
        [("pi", 3.14159), ("e", 2.71828), ("phi", 1.61803)]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect()
    );

    let mut handles = vec![];
    let keys = vec!["pi", "e", "phi", "pi"];

    for key in keys {
        let table = Arc::clone(&table);
        let key = key.to_string();
        handles.push(thread::spawn(move || {
            // Multiple threads read concurrently -- no lock needed
            let value = table.get(&key).unwrap();
            println!("{} = {}", key, value);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
```

## Gotchas

1. **`Rc` is not `Send`**: If you are used to `Rc<T>` for reference counting, it will NOT work
   across threads. `Rc` uses non-atomic reference counts for performance, so it is not `Send`.
   You must use `Arc<T>` (atomic reference counting) for shared ownership across threads. The
   compiler will produce a clear error if you try to send an `Rc` to another thread.

2. **Mutex poisoning**: If a thread panics while holding a `Mutex` lock, the mutex becomes
   "poisoned." Subsequent calls to `.lock()` return an `Err`. Most code handles this with
   `.lock().unwrap()`, which will panic if the mutex is poisoned (propagating the failure).
   In production code, consider `.lock().unwrap_or_else(|e| e.into_inner())` to recover the
   data if appropriate.

3. **Deadlocks are not prevented by the compiler**: Rust prevents data races, but NOT deadlocks.
   If thread A locks mutex X then tries to lock mutex Y, while thread B locks Y then tries X,
   you have a classic deadlock. Rust will not catch this. Always acquire locks in a consistent
   order, and keep critical sections short.

## Related Concepts

- [Ownership and Borrowing](./ownership_and_borrowing.md) -- ownership rules prevent data races at compile time
- [Closures](./closures.md) -- `move` closures transfer ownership into spawned threads
- [Trait Objects](./trait_objects.md) -- `Box<dyn Trait + Send>` for thread-safe dynamic dispatch
- [Unsafe Rust](./unsafe_rust.md) -- `unsafe impl Send/Sync` for custom thread-safe types

## Quick Reference

| Type / Trait               | Purpose                                          |
|----------------------------|--------------------------------------------------|
| `thread::spawn(closure)`  | Spawn an OS thread                               |
| `JoinHandle::join()`      | Wait for a thread to finish                      |
| `Arc<T>`                  | Thread-safe shared ownership (atomic ref count)  |
| `Mutex<T>`                | Mutual exclusion; wraps data, not code           |
| `RwLock<T>`               | Multiple readers OR one writer                   |
| `AtomicU64` / `AtomicBool`| Lock-free atomic operations                      |
| `mpsc::channel()`         | Create a multi-producer, single-consumer channel |
| `Send` (trait)            | Type can be transferred to another thread        |
| `Sync` (trait)            | Type can be shared (via `&T`) across threads     |

**Language comparison:**

| Concept            | Rust                   | Python                  | C++                        | Java                  |
|--------------------|------------------------|-------------------------|----------------------------|-----------------------|
| Spawn thread       | `thread::spawn()`      | `threading.Thread()`    | `std::thread t(fn)`       | `new Thread()`        |
| Shared counter     | `Arc<AtomicU64>`       | N/A (GIL)              | `std::atomic<uint64_t>`   | `AtomicLong`          |
| Locked data        | `Arc<Mutex<T>>`        | `threading.Lock()`     | `std::mutex` + separate data | `synchronized`     |
| Channel            | `mpsc::channel()`      | `queue.Queue()`        | (no stdlib equivalent)    | `BlockingQueue`       |
| Data race safety   | Compile-time enforced  | GIL hides it           | Your problem               | Your problem          |
