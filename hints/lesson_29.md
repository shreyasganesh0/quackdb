# Lesson 29: Morsel Parallelism

## What You're Building
A morsel-driven parallel execution framework that divides work into small chunks (morsels) and distributes them across worker threads. This is the approach used by modern analytical databases to achieve intra-query parallelism -- rather than partitioning data statically, workers dynamically steal morsels from a shared queue, naturally balancing load across cores.

**Core concept count: 2** — the morsel queue (thread-safe work distribution) and the worker loop (take-process-push). Everything else (Arc/Mutex wrappers, factory closures, collector) is scaffolding that supports these two.

> **Unified Concept:** The morsel queue and the scheduler are ONE concept: parallel execution via dynamic work distribution. The queue is the shared work pool, the scheduler spawns workers that drain it. Think of it as a single pattern -- "take work, process it, store result" -- running on multiple threads simultaneously.

## Concept Recap
Building on Lesson 14 (Pipeline Execution): The `Pipeline` and `PhysicalOperator` trait you built earlier execute operators sequentially on DataChunks. Morsel parallelism wraps that same pattern -- each worker thread gets its own operator instance and processes DataChunks (morsels) independently. The `DataChunk` is the morsel. The key difference is that now multiple threads run the same pipeline logic concurrently.

## Rust Concepts You'll Need
- [Concurrency](../concepts/closures.md) -- Arc, Mutex, std::thread::spawn, Send and Sync bounds govern how data crosses thread boundaries
- [Closures](../concepts/closures.md) -- the operator factory pattern uses `impl Fn() -> Box<dyn Trait + Send>` to create per-thread operator instances
- [Trait Objects](../concepts/trait_objects.md) -- PhysicalOperator is used through `Box<dyn PhysicalOperator + Send>` to allow heterogeneous operators

## Key Patterns

### Arc<Mutex<Vec<T>>> for Thread-Safe Shared Data
When multiple threads need to read/write the same collection, wrap it in Arc (shared ownership) and Mutex (exclusive access). Think of it like a shared refrigerator in an office -- the Arc is giving everyone a key to the room, and the Mutex is the "one person at a time" rule for opening the fridge.

```rust
// Analogy: a shared task queue for a web crawler (NOT the QuackDB solution)
use std::sync::{Arc, Mutex};

struct TaskQueue {
    urls: Mutex<Vec<String>>,
}

impl TaskQueue {
    fn new(urls: Vec<String>) -> Self {
        Self { urls: Mutex::new(urls) }
    }

    fn take_next(&self) -> Option<String> {
        let mut guard = self.urls.lock().unwrap();
        guard.pop()
    }
}

let queue = Arc::new(TaskQueue::new(vec!["http://a.com".into(), "http://b.com".into()]));
let q2 = Arc::clone(&queue);
std::thread::spawn(move || {
    while let Some(url) = q2.take_next() {
        println!("Crawling: {}", url);
    }
});
```

### Factory Closures for Per-Thread State
When each thread needs its own mutable operator instance, pass a factory closure that creates a fresh one. This avoids sharing mutable state across threads entirely. It is like a cookie cutter -- you pass around the cutter (factory), and each thread stamps out its own cookie (operator instance).

```rust
// Analogy: per-thread loggers (NOT the QuackDB solution)
fn run_workers(n: usize, factory: impl Fn() -> Box<dyn std::io::Write + Send> + Send + Sync) {
    let factory = Arc::new(factory);
    let mut handles = Vec::new();
    for _ in 0..n {
        let f = Arc::clone(&factory);
        handles.push(std::thread::spawn(move || {
            let mut writer = f();
            writeln!(writer, "Worker reporting").unwrap();
        }));
    }
    for h in handles { h.join().unwrap(); }
}
```

### Worker Loop: Take-Process-Push
Each worker runs a simple loop: take a morsel from the shared queue, run the operator on it, push the result to the shared collector. When the queue returns None, the worker exits. This is the fundamental pattern for dynamic work distribution.

```rust
// Analogy: assembly line workers picking items from a conveyor belt (NOT the QuackDB solution)
fn worker_loop(queue: &TaskQueue, results: &Mutex<Vec<String>>) {
    while let Some(task) = queue.take_next() {
        let result = format!("processed: {}", task);
        results.lock().unwrap().push(result);
    }
}
```

## Common Mistakes
- **Sharing a single operator instance across threads instead of using a factory.** Operators are mutable (they maintain state between calls), so sharing one across threads would require Mutex around every call, killing parallelism. The factory pattern gives each thread its own operator.
- **Forgetting to join thread handles.** If you drop the JoinHandle without calling `join()`, the thread detaches and you lose any panic information. Always join all handles and propagate errors.
- **Not wrapping the factory in Arc before cloning into threads.** The factory closure needs to be shared by all spawned threads. Wrap it in `Arc` so each thread gets a reference-counted pointer to the same factory.

## Step-by-Step Implementation Order
1. Start with `MorselQueue::new()` -- convert each DataChunk into a Morsel with an incrementing morsel_id, store in a Mutex<Vec<Morsel>>, track total count.
2. Implement `MorselQueue::take()` -- lock the mutex, pop from the vec; this naturally provides thread-safe work stealing.
3. Implement `MorselQueue::remaining()` and `MorselQueue::total()` -- lock and return the current vec length / stored total.
4. Implement `ParallelCollector::new()` and `ParallelCollector::push()` -- create a Mutex<Vec<DataChunk>>, push locks and appends.
5. Implement `ParallelCollector::into_results()` -- consume self and extract the inner Vec.
6. Implement `ParallelPipelineExecutor::new()` -- store num_workers.
7. Implement `ParallelPipelineExecutor::execute()` -- spawn `num_workers` threads, each with its own operator from the factory; in a loop, take morsels from the queue and execute the operator, pushing results to the collector. Join all handles.
8. Watch out for: the operator_factory must be wrapped in Arc to share across threads; each thread must use `move` closures; join all thread handles and propagate any errors.

## Rust Sidebar: Send + Sync Bounds
If you hit `dyn PhysicalOperator cannot be sent between threads safely` or `the trait Send is not implemented`, here's what's happening: `std::thread::spawn` requires its closure (and everything it captures) to be `Send`. Your `Box<dyn PhysicalOperator>` is not `Send` by default because the compiler does not know whether the concrete type inside is thread-safe.
The fix: change the trait object to `Box<dyn PhysicalOperator + Send>` and make the factory closure return that type. The factory itself must be `Send + Sync` (wrap in `Arc`). This tells the compiler "I guarantee each operator instance is safe to move into a new thread." Since each thread gets its *own* operator from the factory, there is no shared mutable state.

## Reading the Tests
- **`test_morsel_queue_creation`** creates a queue from 4 chunks of 100 rows each and checks that `total()` is 4 and `remaining()` is 4. This validates your constructor correctly counts morsels.
- **`test_morsel_queue_consumption`** takes 3 morsels sequentially, checks each has 10 rows, then verifies a 4th take returns None and remaining is 0. This tests the basic pop-until-empty behavior.
- **`test_morsel_queue_thread_safe`** spawns 4 threads that all drain from a queue of 100 morsels (10 rows each). It asserts the total collected rows equals 1000, verifying that no morsels are lost or double-consumed under contention. This is the critical concurrency test.
- **`test_parallel_collector`** pushes one chunk and verifies `into_results()` returns a vec of length 1. This tests the collector in isolation.
- **`test_parallel_scan_filter`** creates a ParallelPipelineExecutor with 4 workers and a factory closure `|| Box::new(FilterGt50)`. It feeds 10 chunks of 100 rows (values 0..999) and expects exactly 949 rows where value > 50. This confirms that the factory creates independent operators per thread and the collector merges results correctly.
- **`test_parallel_deterministic_results`** runs the same parallel passthrough twice with 8 chunks of 50 rows and verifies both runs produce exactly 400 total rows. This ensures that despite non-deterministic thread scheduling, the total output is always correct -- no rows lost or duplicated.
