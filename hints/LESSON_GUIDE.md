# QuackDB Lesson Guide: Multi-File Lessons

This guide shows which files belong to each multi-file lesson and the
recommended implementation order. Single-file lessons are not listed here
since they are self-contained.

---

## Lesson 07: Compression (Bitpacking + Delta Encoding)

| Order | File                          | What it does                                    |
|-------|-------------------------------|-------------------------------------------------|
| 1     | `src/compression/bitpack.rs`  | Pack/unpack integers using minimal bit widths    |
| 2     | `src/compression/delta.rs`    | Delta & frame-of-reference encoding; combined delta+bitpack |

**Why this order:** `delta.rs`'s combined `delta_bitpack_encode`/`decode`
functions call into the bitpacking routines in `bitpack.rs`.

---

## Lesson 15: Execution Operators (Scan, Filter, Projection)

| Order | File                              | What it does                                  |
|-------|-----------------------------------|-----------------------------------------------|
| 1     | `src/execution/scan.rs`           | Table scan — leaf operator, reads from a data source |
| 2     | `src/execution/filter.rs`         | Filter — evaluates a predicate, removes non-matching rows |
| 3     | `src/execution/projection.rs`     | Projection — selects/computes output columns   |

**Why this order:** Follows the data flow in a typical pipeline:
scan produces chunks, filter removes rows, projection reshapes columns.

---

## Lesson 21: SQL Frontend (Parser + AST)

| Order | File                    | What it does                                       |
|-------|-------------------------|----------------------------------------------------|
| 1*    | `src/sql/ast.rs`        | AST node type definitions (read-only, no `todo!()`) |
| 2     | `src/sql/parser.rs`     | Recursive descent parser with Pratt expression parsing |

**Why this order:** Read `ast.rs` first to understand the data structures
the parser must produce. `ast.rs` has no logic to implement — it is all
type definitions — but knowing the "shape" of the AST makes writing the
parser much easier.

---

## Lesson 23: Query Planning (Catalog + Binder)

| Order | File                         | What it does                                    |
|-------|------------------------------|-------------------------------------------------|
| 1     | `src/planner/catalog.rs`     | Central metadata store (table schemas + row data) |
| 2     | `src/planner/binder.rs`      | Name resolution & type checking against the catalog |

**Why this order:** The binder calls `Catalog::get_table()` and
`TableInfo::find_column()` to resolve names, so the catalog must be
working first.

---

## Lesson 24: End-to-End Execution (Physical Plan + Database Facade)

| Order | File                               | What it does                                 |
|-------|------------------------------------|----------------------------------------------|
| 1     | `src/planner/physical_plan.rs`     | Converts logical plan to physical pipelines   |
| 2     | `src/db.rs`                        | Top-level `Database` struct; orchestrates the full SQL pipeline |

**Why this order:** `Database::execute_sql` calls `execute_plan()` from
`physical_plan.rs`, so the plan builder must work first. `db.rs` is a
thin integration layer.

---

## Lesson 26: Query Optimization (Statistics + Cost Model + Join Order)

| Order | File                              | What it does                                   |
|-------|-----------------------------------|------------------------------------------------|
| 1     | `src/optimizer/statistics.rs`     | Column statistics & cardinality estimation      |
| 2     | `src/optimizer/cost_model.rs`     | Multi-dimensional cost model (CPU, I/O, network) |
| 3     | `src/optimizer/join_order.rs`     | DPsub join order optimization                   |

**Why this order:** Each layer builds on the previous. Statistics feed
the cost model (row counts drive cost formulas), and the cost model feeds
the join order optimizer (DPsub compares candidate plans by cost).

---

## Lesson 29: Parallel Execution (Morsel Queue + Scheduler)

| Order | File                          | What it does                                    |
|-------|-------------------------------|-------------------------------------------------|
| 1     | `src/parallel/morsel.rs`      | `MorselQueue` (thread-safe chunk queue) + `ParallelCollector` |
| 2     | `src/parallel/scheduler.rs`   | Spawns worker threads, drives morsel-based execution |

**Why this order:** The scheduler's worker loop calls
`MorselQueue::take()` and `ParallelCollector::push()`, so those data
structures must be implemented first.

---

## Lesson 32: Distributed Execution — Planning (Planner + Exchange)

| Order | File                              | What it does                                   |
|-------|-----------------------------------|-------------------------------------------------|
| 1     | `src/distributed/planner.rs`     | Splits a logical plan into distributed fragments |
| 2     | `src/execution/exchange.rs`       | Physical exchange operator for pipeline boundaries |

**Why this order:** The planner decides *where* exchanges are needed and
*what type*; the exchange operator is the runtime component that executes
data transfer at those boundaries.

---

## Lesson 33: Distributed Execution — Runtime (Shuffle + Coordinator)

| Order | File                               | What it does                                  |
|-------|------------------------------------|-----------------------------------------------|
| 1     | `src/distributed/shuffle.rs`       | Exchange channels, shuffle, broadcast, gather operators |
| 2     | `src/distributed/coordinator.rs`   | Wires fragments together, spawns threads, collects results |

**Why this order:** The coordinator calls `ExchangeChannel::new()` and
passes senders/receivers to the operators defined in `shuffle.rs`, so the
data movement primitives must exist first.
