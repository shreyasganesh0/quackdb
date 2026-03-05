# QuackDB

Build a distributed analytical database from scratch in Rust.

QuackDB is a Boot.dev-style tutorial with 35 lessons that walks you through building a complete OLAP database engine — from arena allocators to distributed query execution. Each lesson has a pre-written test suite. You read the tests, understand the API, then implement until they pass.

By completion, you'll have a working distributed analytical database demonstrating deep systems expertise.

## Prerequisites

- Rust (stable toolchain)
- `make`

## Quick Start

```bash
# Check that lesson 01 compiles
make check LESSON=01

# Run lesson 01 tests (they'll fail with todo!())
make lesson LESSON=01

# Implement src/arena.rs, then run again
make lesson LESSON=01

# Run all tests through lesson 04
make upto LESSON=04

# Run the full suite (only passes when all 35 lessons are complete)
make all
```

## How It Works

- Each lesson maps to one or more source files with `todo!()` stubs
- Test files in `tests/` contain complete assertions — your job is to make them pass
- Lessons are feature-gated (`lesson01` through `lesson35`), each enabling all prior lessons
- Only the code needed for a given lesson is compiled

## Curriculum

### Part I: Foundations (L01-L04)

| Lesson | Topic | Source |
|--------|-------|--------|
| 01 | Arena Allocator | `src/arena.rs` |
| 02 | Data Types & Type System | `src/types.rs` |
| 03 | Columnar Vectors | `src/vector.rs` |
| 04 | Data Chunks | `src/chunk.rs` |

### Part II: Compression (L05-L08)

| Lesson | Topic | Source |
|--------|-------|--------|
| 05 | Run-Length Encoding | `src/compression/rle.rs` |
| 06 | Dictionary Encoding | `src/compression/dictionary.rs` |
| 07 | Bitpacking & Delta Encoding | `src/compression/bitpack.rs`, `src/compression/delta.rs` |
| 08 | Compression Framework | `src/compression/frame.rs` |

### Part III: Storage Engine (L09-L12)

| Lesson | Topic | Source |
|--------|-------|--------|
| 09 | Pages & Page Layout | `src/storage/page.rs` |
| 10 | Buffer Pool Manager | `src/storage/buffer_pool.rs` |
| 11 | Columnar File Writer | `src/storage/columnar_file.rs` |
| 12 | Columnar File Reader | `src/storage/reader.rs` |

### Part IV: Vectorized Execution Engine (L13-L19)

| Lesson | Topic | Source |
|--------|-------|--------|
| 13 | Expression Evaluation | `src/execution/expression.rs` |
| 14 | Pipeline Execution Model | `src/execution/pipeline.rs` |
| 15 | Scan, Filter, Projection | `src/execution/scan.rs`, `filter.rs`, `projection.rs` |
| 16 | Hash Aggregation | `src/execution/hash_aggregate.rs` |
| 17 | Hash Join | `src/execution/hash_join.rs` |
| 18 | Sort-Merge Join | `src/execution/sort_merge_join.rs` |
| 19 | External Sort | `src/execution/sort.rs` |

### Part V: SQL Frontend (L20-L24)

| Lesson | Topic | Source |
|--------|-------|--------|
| 20 | SQL Lexer | `src/sql/lexer.rs` |
| 21 | SQL Parser (Pratt Parsing) | `src/sql/parser.rs`, `src/sql/ast.rs` |
| 22 | Logical Query Plan | `src/planner/logical_plan.rs` |
| 23 | Catalog & Binder | `src/planner/catalog.rs`, `src/planner/binder.rs` |
| 24 | Physical Plan & Execution | `src/planner/physical_plan.rs`, `src/db.rs` |

### Part VI: Query Optimization (L25-L26)

| Lesson | Topic | Source |
|--------|-------|--------|
| 25 | Rule-Based Optimizer | `src/optimizer/rules.rs` |
| 26 | Cost-Based Optimizer & Join Ordering | `src/optimizer/statistics.rs`, `cost_model.rs`, `join_order.rs` |

### Part VII: Transactions & Durability (L27-L28)

| Lesson | Topic | Source |
|--------|-------|--------|
| 27 | MVCC | `src/transaction/mvcc.rs` |
| 28 | Write-Ahead Logging | `src/transaction/wal.rs` |

### Part VIII: Parallelism & Distribution (L29-L33)

| Lesson | Topic | Source |
|--------|-------|--------|
| 29 | Morsel-Driven Parallelism | `src/parallel/morsel.rs`, `src/parallel/scheduler.rs` |
| 30 | Window Functions | `src/execution/window.rs` |
| 31 | Data Partitioning | `src/distributed/partition.rs` |
| 32 | Distributed Query Planning | `src/distributed/planner.rs` |
| 33 | Shuffle & Exchange | `src/distributed/shuffle.rs`, `src/distributed/coordinator.rs` |

### Part IX: Advanced (L34-L35)

| Lesson | Topic | Source |
|--------|-------|--------|
| 34 | Adaptive Query Execution | `src/execution/adaptive.rs` |
| 35 | SIMD-Style Vectorization | `src/simd.rs` |

## Dependency Graph

```
L01 -> L02 -> L03 -> L04 -> L05-L08 -> L09-L12 -> L13-L19 -> L20-L24 -> L25-L26
                                                                           |
                                              L27-L28 <- L29 -> L30 -> L31-L33 -> L34-L35
```

## Constraints

No external crates for core database logic. The only allowed dependencies are:

- `thiserror` — error type derivation
- `byteorder` — byte serialization
- `crc32fast` — checksums
- `rand` — test-only (dev dependency)

## Project Structure

```
quackdb/
├── Cargo.toml          # Features: lesson01-lesson35
├── Makefile            # make lesson/upto/all/check
├── src/
│   ├── lib.rs          # Module declarations (feature-gated)
│   ├── arena.rs        # L01
│   ├── types.rs        # L02
│   ├── vector.rs       # L03
│   ├── chunk.rs        # L04
│   ├── compression/    # L05-L08
│   ├── storage/        # L09-L12
│   ├── execution/      # L13-L19, L30, L32-L34
│   ├── sql/            # L20-L21
│   ├── planner/        # L22-L24
│   ├── optimizer/      # L25-L26
│   ├── transaction/    # L27-L28
│   ├── parallel/       # L29
│   ├── distributed/    # L31-L33
│   ├── simd.rs         # L35
│   └── db.rs           # L24+ (top-level facade)
├── tests/              # 35 test files (one per lesson)
└── benches/
    └── micro.rs
```
