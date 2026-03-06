# Lesson 29: Morsel Parallelism

## What You're Building
A morsel-driven parallel execution framework that divides work into small chunks (morsels) and distributes them across worker threads. This is the approach used by modern analytical databases to achieve intra-query parallelism -- rather than partitioning data statically, workers dynamically steal morsels from a shared queue, naturally balancing load across cores.

## Rust Concepts You'll Need
- [Concurrency](../concepts/closures.md) -- Arc, Mutex, std::thread::spawn, Send and Sync bounds govern how data crosses thread boundaries
- [Closures](../concepts/closures.md) -- the operator factory pattern uses `impl Fn() -> Box<dyn Trait + Send>` to create per-thread operator instances
- [Trait Objects](../concepts/trait_objects.md) -- PhysicalOperator is used through `Box<dyn PhysicalOperator + Send>` to allow heterogeneous operators

## Key Patterns

### Arc<Mutex<Vec<T>>> for Thread-Safe Shared Data
When multiple threads need to read/write the same collection, wrap it in Arc (shared ownership) and Mutex (exclusive access).

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
When each thread needs its own mutable operator instance, pass a factory closure that creates a fresh one. This avoids sharing mutable state across threads entirely.

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

## Step-by-Step Implementation Order
1. Start with `MorselQueue::new()` -- convert each DataChunk into a Morsel with an incrementing morsel_id, store in a Mutex<Vec<Morsel>>, track total count
2. Implement `MorselQueue::take()` -- lock the mutex, pop from the vec; this naturally provides thread-safe work stealing
3. Implement `MorselQueue::remaining()` -- lock and return the current vec length
4. Implement `ParallelCollector::push()` -- lock the mutex and push the chunk
5. Implement `ParallelPipelineExecutor::execute()` -- spawn `num_workers` threads, each with its own operator from the factory; in a loop, take morsels from the queue and execute the operator, pushing results to the collector
6. Implement `PartitionedHashTable::new()` -- create `num_partitions` AggregateHashTable instances
7. Implement `partition_for_hash()` -- use modulo to route hash values to partitions
8. Watch out for: the operator_factory must be wrapped in Arc to share across threads; each thread must use `move` closures; join all thread handles and propagate any errors

## Reading the Tests
- **`test_morsel_queue_thread_safe`** spawns 4 threads that all drain from a queue of 100 morsels (10 rows each). It asserts the total collected rows equals 1000, verifying that no morsels are lost or double-consumed under contention.
- **`test_parallel_scan_filter`** creates a ParallelPipelineExecutor with 4 workers and a factory closure `|| Box::new(FilterGt50)`. It feeds 10 chunks of 100 rows (values 0..999) and expects exactly 949 rows where value > 50. This confirms that the factory creates independent operators per thread and the collector merges results correctly.
