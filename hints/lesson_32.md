# Lesson 32: Distributed Plan

## What You're Building
A distributed query planner that takes a logical query plan and inserts exchange operators to describe how data must move between nodes. In a distributed database, different parts of a query run on different machines. Exchanges specify whether data should be gathered to one node, repartitioned by hash across nodes, or broadcast to all nodes. The planner breaks the logical plan into fragments, each representing a unit of work that can execute on a single node.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- ExchangeType and LogicalPlan are both enums; pattern matching drives decisions about what exchange to insert at each plan boundary
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- LogicalPlan is a recursive enum (e.g., Join contains Box<LogicalPlan> children); you must recursively walk the tree to insert exchanges
- [Iterators](../concepts/iterators.md) -- collecting fragments, iterating over plan children

## Key Patterns

### Recursive Plan Traversal with Transformation
Walk a tree structure recursively, deciding at each node whether to insert a new node (exchange) between parent and child.

```rust
// Analogy: inserting checkpoints into a recipe workflow (NOT the QuackDB solution)
enum Step {
    Simple(String),
    Sequential(Vec<Step>),
    Parallel(Box<Step>, Box<Step>),
}

fn insert_checkpoints(step: Step) -> Step {
    match step {
        Step::Simple(s) => Step::Simple(s),
        Step::Sequential(steps) => {
            Step::Sequential(steps.into_iter().map(insert_checkpoints).collect())
        }
        Step::Parallel(left, right) => {
            // Insert a "sync point" before merging parallel branches
            let l = insert_checkpoints(*left);
            let r = insert_checkpoints(*right);
            Step::Sequential(vec![
                Step::Parallel(Box::new(l), Box::new(r)),
                Step::Simple("sync_checkpoint".into()),
            ])
        }
    }
}
```

### Determining Data Movement Requirements
Different operations need different data distributions. A join on column X needs both sides partitioned by X. An aggregation grouped by Y needs data partitioned by Y. A final result needs gathering to one node.

```rust
// Analogy: deciding shipping routes for a warehouse system (NOT the QuackDB solution)
enum ShipMode { DirectToStore, Redistribute, BroadcastToAll }

fn decide_shipping(order_size: usize, num_stores: usize) -> ShipMode {
    if order_size < 10 {
        ShipMode::BroadcastToAll  // small enough to send everywhere
    } else if num_stores > 1 {
        ShipMode::Redistribute    // split across stores
    } else {
        ShipMode::DirectToStore   // single destination
    }
}
```

## Step-by-Step Implementation Order
1. Start with `FragmentBuilder::add_fragment()` -- assign the next_id, create a PlanFragment with the given plan and exchange types, push it, increment next_id, return the assigned id
2. Implement `DistributedPlanner::needs_exchange()` -- examine the parent node type: joins need Repartition on join keys, aggregates need Repartition on group keys, the root needs Gather
3. Implement `DistributedPlanner::plan()` -- recursively walk the logical plan tree; at each node, check needs_exchange for each child; when an exchange is needed, split the plan into separate fragments connected by the exchange type
4. For joins, insert Repartition exchanges on both children so they are co-partitioned by the join key columns
5. For aggregates, consider a two-phase strategy: local pre-aggregation on each node, then Repartition by group keys, then final aggregation
6. The root of the plan should have a Gather exchange output to collect results to the coordinator
7. Watch out for: a single scan with no joins or aggregates still needs at least one fragment; make sure fragment_ids are unique and sequential

## Reading the Tests
- **`test_join_repartition`** creates a join plan between two scans and verifies that the resulting fragments contain at least one Repartition exchange. This confirms your planner recognizes that both sides of a join must be co-partitioned.
- **`test_fragment_builder`** directly calls add_fragment with a Gather exchange output and checks the returned id is 0 and the built fragments vec has length 1 with fragment_id 0. This validates the basic fragment construction mechanics.
