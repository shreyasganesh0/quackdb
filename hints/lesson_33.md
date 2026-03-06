# Lesson 33: Shuffle & Exchange

## What You're Building
The data movement operators that physically transfer data chunks between threads (simulating nodes) in a distributed execution. ExchangeChannel provides a typed communication pipe using Rust's mpsc channels. ShuffleOperator routes outgoing data to the correct partition via hashing. GatherOperator merges results from multiple sources. BroadcastOperator replicates data to all receivers. Together, these implement the exchange types planned in Lesson 32.

## Rust Concepts You'll Need
- [Concurrency](../concepts/closures.md) -- mpsc::channel for message passing between threads; Sender and Receiver enforce ownership-based communication
- [Ownership and Borrowing](../concepts/ownership_and_borrowing.md) -- sending a DataChunk through a channel transfers ownership; the sender loses access after `send()`, and the receiver gains it

## Key Patterns

### mpsc Channels for Data Movement
Rust's `mpsc::channel` creates a sender/receiver pair. Sending moves data into the channel; receiving takes ownership on the other end. Closing the sender (dropping it) signals completion.

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
Instead of relying on channel closure, wrap messages in `Option<T>` and send `None` as a termination signal. This allows explicit close semantics.

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
Round-robin or sequential polling across multiple receivers until all are exhausted.

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

## Step-by-Step Implementation Order
1. Start with `ExchangeChannel::new()` -- create an `mpsc::channel::<Option<DataChunk>>()`, wrap the sender in ExchangeSender and receiver in ExchangeReceiver, return the pair
2. Implement `ExchangeSender::send()` -- wrap the chunk in Some and send through the mpsc sender; map errors to String
3. Implement `ExchangeReceiver::recv()` -- call `self.receiver.recv()`, return None if the channel yields Err or the message is None
4. Implement `GatherOperator::new()` -- store receivers and set current index to 0
5. Implement `GatherOperator::next_chunk()` -- iterate through receivers starting at current; when one returns a chunk, advance current and return it; when all receivers are exhausted, return None
6. Implement `BroadcastOperator::broadcast()` -- clone the chunk and send to each sender (the last one can move instead of clone)
7. Implement `ShuffleOperator` execution -- for each row in the input chunk, hash the partition columns, determine the target partition, build per-partition chunks, then send each to the corresponding sender
8. Implement `DistributedExecutor::execute()` -- set up exchange channels between fragments, spawn threads for each fragment, connect them via senders/receivers, collect final results via a gather
9. Watch out for: the close() method sends None as a sentinel, so your receiver must handle both None messages and channel disconnection; cloning DataChunk for broadcast requires all N-1 copies to be cloned while the last can be moved

## Reading the Tests
- **`test_exchange_channel`** sends a 3-row chunk through a channel, closes the sender, then receives. It asserts the received chunk has 3 rows and a second recv returns None. This validates the sentinel-based termination protocol.
- **`test_gather_operator`** creates two channels, sends one chunk through each, closes both senders, then uses GatherOperator to collect all chunks. It expects a total of 4 rows (2 + 2), confirming the gather correctly drains multiple receivers.
