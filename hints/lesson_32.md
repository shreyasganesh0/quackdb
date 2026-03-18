# Lesson 32: Distributed Plan

## What You're Building
A distributed query planner that takes a logical query plan and inserts exchange operators to describe how data must move between nodes. In a distributed database, different parts of a query run on different machines. Exchanges specify whether data should be gathered to one node, repartitioned by hash across nodes, or broadcast to all nodes. The planner breaks the logical plan into fragments, each representing a unit of work that can execute on a single node.

## Concept Recap
Building on Lessons 25-26 (Optimization) and Lesson 31 (Partitioning): The logical plan tree from the optimizer is what the distributed planner splits into fragments. The partitioning strategies (hash, broadcast) from Lesson 31 directly inform what type of exchange to insert. A hash repartition exchange means "re-partition both sides by the join key so co-located rows end up on the same node" -- the same concept as hash partitioning, but applied dynamically during query execution.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- ExchangeType and LogicalPlan are both enums; pattern matching drives decisions about what exchange to insert at each plan boundary
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- LogicalPlan is a recursive enum (e.g., Join contains Box<LogicalPlan> children); you must recursively walk the tree to insert exchanges
- [Iterators](../concepts/iterators.md) -- collecting fragments, iterating over plan children

## Key Patterns

### Recursive Plan Traversal with Transformation
Walk a tree structure recursively, deciding at each node whether to insert a new node (exchange) between parent and child. This is like a road-trip planner that inserts gas stops between cities -- you walk the route and decide at each leg whether a refueling point is needed.

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
Different operations need different data distributions. A join on column X needs both sides partitioned by X. An aggregation grouped by Y needs data partitioned by Y. A final result needs gathering to one node. Think of it like coordinating a group project -- some tasks need everyone to have the same data (broadcast), some need data split by topic (repartition), and the final report needs to be collected in one place (gather).

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

### Fragment as a Unit of Work
Each fragment represents a portion of the query that can execute on a single node. Fragments are connected by exchanges that describe how data flows between them. The fragment_id provides a unique identifier for scheduling and coordination.

```rust
// Analogy: dividing a construction project into contractor assignments (NOT the QuackDB solution)
struct ContractorJob {
    job_id: usize,
    task: String,
    receives_materials_from: Option<usize>, // another job_id
    sends_results_to: Option<usize>,        // another job_id
}
```

## Common Mistakes
- **Forgetting the Gather exchange at the root.** The final results must be collected to a single coordinator node. Without a Gather at the top, your distributed plan has no way to merge results for the client.
- **Not inserting exchanges on both sides of a join.** For a hash join to work in a distributed setting, both the left and right inputs must be repartitioned by the join key. If you only repartition one side, co-located join pairs will not be on the same node.
- **Assigning duplicate fragment IDs.** Each fragment must have a unique ID for the executor to schedule them independently. Use a monotonically increasing counter in FragmentBuilder.

## Step-by-Step Implementation Order
1. Start with `FragmentBuilder::add_fragment()` -- assign the next_id, create a PlanFragment with the given plan and exchange types, push it, increment next_id, return the assigned id.
2. Implement `FragmentBuilder::build()` -- return the accumulated fragments.
3. Implement `DistributedPlanner::needs_exchange()` -- examine the parent node type: joins need Repartition on join keys, aggregates need Repartition on group keys, the root needs Gather.
4. Implement `DistributedPlanner::plan()` -- recursively walk the logical plan tree; at each node, check needs_exchange for each child; when an exchange is needed, split the plan into separate fragments connected by the exchange type.
5. For joins, insert Repartition exchanges on both children so they are co-partitioned by the join key columns.
6. For aggregates, consider a two-phase strategy: local pre-aggregation on each node, then Repartition by group keys, then final aggregation.
7. The root of the plan should have a Gather exchange output to collect results to the coordinator.
8. Watch out for: a single scan with no joins or aggregates still needs at least one fragment; make sure fragment_ids are unique and sequential.

## Reading the Tests
- **`test_fragment_builder`** directly calls add_fragment with a Gather exchange output and checks the returned id is 0 and the built fragments vec has length 1 with fragment_id 0. This validates the basic fragment construction mechanics and sequential ID assignment.
- **`test_single_scan_plan`** plans a simple scan and expects at least one fragment. Even without joins or aggregates, a scan needs a fragment (and typically a Gather exchange to return results to the coordinator).
- **`test_join_repartition`** creates a join plan between two scans with an equality condition and verifies that the resulting fragments contain at least one Repartition exchange or produce multiple fragments. This confirms your planner recognizes that both sides of a join must be co-partitioned.
- **`test_aggregate_repartition`** plans a SUM aggregate grouped by column "id" and expects non-empty fragments. This tests that your planner handles distributed aggregation, potentially with a two-phase (partial + final) strategy.
- **`test_filter_pushdown_in_distributed`** plans a Filter over a Scan and expects non-empty fragments. The filter should be pushed into the scan fragment rather than creating a separate fragment, testing that simple predicates stay local.
- **`test_broadcast_exchange`** plans a single small table scan. This verifies that your planner handles cases where broadcast might be appropriate (small tables).
- **`test_multi_join_fragments`** creates a 3-way join (Join(Join(a, b), c)) and expects at least 2 fragments. This tests that your planner correctly handles nested joins by inserting exchanges at each join boundary.
