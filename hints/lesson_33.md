# Lesson 33: Shuffle & Exchange

## What You're Building
The data movement operators that physically transfer data chunks between threads (simulating nodes) in a distributed execution. ExchangeChannel provides a typed communication pipe using Rust's mpsc channels. ShuffleOperator routes outgoing data to the correct partition via hashing. GatherOperator merges results from multiple sources. BroadcastOperator replicates data to all receivers. Together, these implement the exchange types planned in Lesson 32.

**Core concept count: 2** — the exchange channel abstraction and hash-based routing. Everything else (gather, broadcast, sentinel termination) is scaffolding that supports these two.

> **Unified Concept:** Shuffle, gather, and broadcast are all ONE concept: data movement between nodes. The channel is the pipe, and the three operators are just different routing strategies (hash-route, merge, or copy-to-all). Learn the channel abstraction first, then each operator is a thin wrapper that decides *where* to send each chunk.

## Concept Recap
Building on Lesson 32 (Distributed Plan) and Lesson 31 (Partitioning): In Lesson 32 you decided where to insert exchanges; now you build the actual data movement machinery. The ShuffleOperator uses the same hash partitioning logic from Lesson 31 to route rows to the correct destination. The GatherOperator collects results the same way Lesson 29's ParallelCollector merged morsel outputs. Think of this as the physical implementation of the logical exchange types.

## Rust Concepts You'll Need
- [Concurrency](../concepts/closures.md) -- mpsc::channel for message passing between threads; Sender and Receiver enforce ownership-based communication
- [Ownership and Borrowing](../concepts/ownership_and_borrowing.md) -- sending a DataChunk through a channel transfers ownership; the sender loses access after `send()`, and the receiver gains it

## Key Patterns

### mpsc Channels for Data Movement
Rust's `mpsc::channel` creates a sender/receiver pair. Sending moves data into the channel; receiving takes ownership on the other end. Closing the sender (dropping it) signals completion. This is like a pneumatic tube system in a bank -- you put the capsule in on one end, and it arrives at the other end. Once sent, you no longer have it.

```rust
// Analogy: a pipeline of document processors (NOT the QuackDB solution)
use std::sync::mpsc;

fn pipeline_example() {
    let (tx, rx) = mpsc::channel::<String>();

    std::thread::spawn(move || {
        tx.send("document_1".to_string()).unwrap();
        tx.send("document_2".to_string()).unwrap();
        // tx is dropped here, signaling completion
    });

    while let Ok(doc) = rx.recv() {
        println!("Processing: {}", doc);
    }
    // recv() returns Err when sender is dropped
}
```

### Sentinel-Based Channel Termination
Instead of relying on channel closure, wrap messages in `Option<T>` and send `None` as a termination signal. This allows explicit close semantics without dropping the sender. It is like a conveyor belt where you put an "END" marker at the end of the items -- the worker on the other side knows to stop when they see it.

```rust
// Analogy: a print job queue with explicit end marker (NOT the QuackDB solution)
use std::sync::mpsc;

fn print_queue() {
    let (tx, rx) = mpsc::channel::<Option<String>>();

    tx.send(Some("page1.pdf".into())).unwrap();
    tx.send(Some("page2.pdf".into())).unwrap();
    tx.send(None).unwrap(); // explicit termination

    loop {
        match rx.recv().unwrap() {
            Some(job) => println!("Printing: {}", job),
            None => break, // sentinel received
        }
    }
}
```

### Gathering from Multiple Sources
Round-robin or sequential polling across multiple receivers until all are exhausted. This is like a conference moderator collecting questions from multiple audience microphones -- you cycle through them, and when a microphone goes silent, you skip it.

```rust
// Analogy: merging feeds from multiple news sources (NOT the QuackDB solution)
fn merge_feeds(receivers: &[mpsc::Receiver<String>]) -> Vec<String> {
    let mut results = Vec::new();
    let mut active: Vec<bool> = vec![true; receivers.len()];
    let mut idx = 0;
    while active.iter().any(|&a| a) {
        if active[idx] {
            match receivers[idx].try_recv() {
                Ok(item) => results.push(item),
                Err(mpsc::TryRecvError::Disconnected) => active[idx] = false,
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        idx = (idx + 1) % receivers.len();
    }
    results
}
```

## Common Mistakes
- **Forgetting to close/drop all senders before expecting receivers to terminate.** If any sender is still alive (not dropped or closed), `recv()` will block forever waiting for more data. Make sure `close()` sends the None sentinel on every channel.
- **Cloning the last DataChunk in broadcast when you could move it.** For N receivers, you need N-1 clones and 1 move. Cloning the last one is wasteful. Send clones to the first N-1 receivers and move the original to the last.
- **Not handling the case where a receiver gets both a None sentinel and a channel disconnect.** Your receive logic should handle both gracefully -- either a None message or a RecvError means the stream is done.

## Step-by-Step Implementation Order
1. Start with `ExchangeChannel::new()` -- create an `mpsc::channel::<Option<DataChunk>>()`, wrap the sender in ExchangeSender and receiver in ExchangeReceiver, return the pair.
2. Implement `ExchangeSender::send()` -- wrap the chunk in Some and send through the mpsc sender; map errors to String.
3. Implement `ExchangeSender::close()` -- send None through the channel as a termination sentinel.
4. Implement `ExchangeReceiver::recv()` -- call `self.receiver.recv()`, return None if the channel yields Err or the message is None.
5. Implement `GatherOperator::new()` -- store receivers and set current index to 0.
6. Implement `GatherOperator::next_chunk()` -- iterate through receivers starting at current; when one returns a chunk, advance current and return it; when all receivers are exhausted, return None.
7. Implement `BroadcastOperator::broadcast()` -- clone the chunk and send to each sender (the last one can move instead of clone).
8. Implement `ShuffleOperator` execution -- for each row in the input chunk, hash the partition columns, determine the target partition, build per-partition chunks, then send each to the corresponding sender.
9. Implement `DistributedExecutor::execute()` -- set up exchange channels between fragments, spawn threads for each fragment, connect them via senders/receivers, collect final results via a gather.

## Rust Sidebar: Ownership Transfer Through Channels
If you hit `use of moved value` after calling `sender.send(chunk)` or `cannot borrow as mutable because it is also borrowed`, here's what's happening: `mpsc::Sender::send()` *moves* the value into the channel -- the sender no longer owns it. If you try to use `chunk` after sending, the compiler rejects it because ownership transferred.
The fix: for broadcast, clone the chunk for the first N-1 senders and move the original to the last: `for s in &senders[..n-1] { s.send(chunk.clone())?; } senders[n-1].send(chunk)?;`. This avoids an unnecessary final clone. For shuffle, build per-partition chunks first, then send each one -- each chunk is used exactly once.

## Reading the Tests
- **`test_exchange_channel`** sends a 3-row chunk through a channel, closes the sender, then receives. It asserts the received chunk has 3 rows and a second recv returns None. This validates the basic send-close-receive protocol and the sentinel-based termination.
- **`test_gather_operator`** creates two channels, sends one chunk through each (2 rows each), closes both senders, then uses GatherOperator to collect all chunks. It expects a total of 4 rows. This confirms the gather correctly drains multiple receivers and merges results into a single stream.
- **`test_broadcast`** creates two channels, broadcasts a 2-row chunk, and verifies both receivers get a 2-row chunk. This tests that broadcast creates proper copies for all destinations.
- **`test_shuffle_routing`** creates a ShuffleOperator with 2 output channels and partition column 0. This is a basic smoke test verifying that the shuffle operator can be constructed without crashing. The exact routing depends on your hash implementation.
- **`test_distributed_executor`** creates a full pipeline: a DistributedPlanner plans a scan, then DistributedExecutor runs the fragments with a catalog containing 10 rows of test data. It expects 10 rows in the final result. This is the end-to-end integration test for distributed execution.
- **`test_backpressure`** sends 100 single-row chunks through a channel without receiving any, then closes and drains all 100. It expects all 100 rows to be present. This verifies that the channel buffers data correctly when the sender outpaces the receiver, with no data loss.

## What Comes Next
You've built a distributed query execution framework. Part IX wraps up with two
**advanced topics** that push performance further. Lesson 34 introduces adaptive
execution -- using runtime statistics (Bloom filters, cardinality estimates) to adjust
query strategies on the fly. Lesson 35 closes with SIMD-style vectorization, writing
tight loops that the compiler auto-vectorizes for maximum throughput. These lessons
tie together everything you've built into a production-grade analytical database.
