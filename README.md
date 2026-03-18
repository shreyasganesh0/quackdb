# 🦆 QuackDB

**Build a distributed analytical database from scratch in Rust.**

You're about to build something real. Not a toy, not a tutorial that holds your hand through copy-paste — a genuine OLAP database engine with arena allocators, columnar storage, vectorized execution, a SQL frontend, query optimization, MVCC transactions, and distributed query processing. In 35 lessons, you'll go from an empty `todo!()` to a working distributed analytical database. Every line of core logic is yours.

No prior Rust experience required. No database background assumed. Just bring curiosity and a willingness to get stuck, then unstuck. That's how real learning works.

## Your First 5 Minutes

```bash
# 1. Clone and enter the project
git clone <your-fork-url> && cd quackdb

# 2. Run the first lesson's tests — they WILL fail. That's the point.
make lesson LESSON=01

# You'll see something like:
#   thread 'test_arena_allocate' panicked at 'not yet implemented'
#   ❌ Tests failed — this is expected! Your job is to make them pass.

# 3. Read the hint to understand what you're building
make hint LESSON=01

# 4. Open the source file and replace the todo!() stubs with real code
#    Lesson 01 → src/arena.rs

# 5. Run the test again
make lesson LESSON=01

# When you see "test result: ok" — you've completed your first lesson! ✅
```

That's the whole loop: **run → fail → read → implement → pass**. Repeat 35 times and you've built a database.

## How to Approach Each Lesson

```
┌─────────────────────────────────────────────────┐
│  1. Run the test          make lesson LESSON=NN │
│         ↓                                       │
│  2. See it fail ❌        Read the error output  │
│         ↓                                       │
│  3. Read the hint         make hint LESSON=NN   │
│         ↓                                       │
│  4. Read the test file    tests/lesson_NN_*.rs  │
│         ↓                                       │
│  5. Implement             Edit src/ files       │
│         ↓                                       │
│  6. Run again             make lesson LESSON=NN │
│         ↓                                       │
│  7. Pass ✅               Move to next lesson    │
└─────────────────────────────────────────────────┘
```

**Tip:** The test file is your specification. Read it carefully — it tells you exactly what your code needs to do. The hint file explains *how* to think about it.

## What You'll Learn

By the end of QuackDB, you won't just "know about" databases. You'll have built:

- **A memory arena** — manual allocation strategy used in real databases (L01)
- **A columnar type system** — the foundation of every OLAP engine (L02-L04)
- **Compression algorithms** — RLE, dictionary, bitpacking, delta encoding (L05-L08)
- **A storage engine** — page layouts, buffer pool management, columnar files (L09-L12)
- **A vectorized execution engine** — pipelines, hash joins, sort-merge joins (L13-L19)
- **A SQL parser from scratch** — lexer, Pratt parser, AST (L20-L21)
- **A query planner** — logical plans, catalog, binder, physical plans (L22-L24)
- **A query optimizer** — rule-based and cost-based with join reordering (L25-L26)
- **Transactions** — MVCC and write-ahead logging (L27-L28)
- **Parallel execution** — morsel-driven parallelism, window functions (L29-L30)
- **Distributed queries** — partitioning, distributed planning, shuffle (L31-L33)
- **Advanced techniques** — adaptive execution, SIMD-style vectorization (L34-L35)

These are skills that transfer directly to working on systems like DuckDB, ClickHouse, DataFusion, and Snowflake.

## Prerequisites

- **Rust** (stable toolchain) — [install here](https://rustup.rs/)
- **make** — already installed on most systems

Don't know Rust yet? That's fine. Each lesson has a hint file that teaches exactly the Rust concepts you need, right when you need them. See [New to Rust?](#new-to-rust) below.

## Quick Start

**Step 1: Verify your setup**

```bash
make check LESSON=01
```

This checks that lesson 01 compiles. If you see no errors, you're good.

**Step 2: Run your first lesson**

```bash
make lesson LESSON=01
```

You'll see test failures with `not yet implemented` — that's the `todo!()` macro telling you where to write code.

**Step 3: Implement and iterate**

Open `src/arena.rs`, replace the `todo!()` stubs with your implementation, and run the test again. Keep going until all tests pass.

**Step 4: Move forward**

```bash
make lesson LESSON=02   # Next lesson
make upto LESSON=04     # Run all tests through lesson 04
make progress           # See how far you've gotten
```

**Step 5: When you've done all 35**

```bash
make all                # Victory lap — runs the entire test suite 🦆
```

## Track Your Progress

```bash
make progress
```

This scans all 35 lessons and shows you which ones compile and pass:

```
🦆 QuackDB Progress
✅ Lesson 01 — Arena Allocator
✅ Lesson 02 — Data Types
✅ Lesson 03 — Columnar Vectors
❌ Lesson 04 — Data Chunks          ← you are here
   Lesson 05 — Run-Length Encoding
   ...
```

You can also jump straight to your next unfinished lesson:

```bash
make next    # Finds and runs the first failing lesson
```

## New to Rust?

Each lesson includes a hint file that teaches the Rust concepts you'll need.

```bash
make hint LESSON=01       # View hints for lesson 01
make concepts             # List all concept reference files
cat hints/concepts/ownership_and_borrowing.md   # Read a specific concept
```

- **`hints/lesson_NN.md`** — what you're building, which Rust concepts apply, key patterns with examples, and a step-by-step implementation order
- **`hints/concepts/`** — self-contained reference files for each Rust concept, with examples. Assumes you know another language but not Rust.

The hints never give away the solution. If you already know Rust, skip them.

## Getting Stuck?

**Common Rust errors and what they mean:**

| Error | What's happening | Fix |
|-------|-----------------|-----|
| `not yet implemented` | You hit a `todo!()` | Replace it with real code |
| `borrow of moved value` | You used a value after giving it away | Clone it, or restructure to use references |
| `cannot borrow as mutable` | You need `&mut` but only have `&` | Check function signatures; you may need `mut` |
| `lifetime may not live long enough` | A reference outlives its source | Look at the hint file's lifetime section |
| `mismatched types` | Return type doesn't match | Check what the test expects |

**General tips:**

1. **Read the test file first.** It's the spec. Understand what the test calls and what it asserts.
2. **Read the hint file.** It won't give the answer, but it explains the approach.
3. **Check that earlier lessons still pass:** `make upto LESSON=NN`
4. **Compile early, compile often:** `make check LESSON=NN` is faster than a full test run.
5. **Use `cargo clippy`** for idiomatic Rust suggestions.

**Still stuck?** Look at the test assertions line by line. Each one tells you something specific your implementation needs to satisfy. Work through them one at a time.

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
├── Makefile            # make lesson/upto/all/check/progress/next
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
├── hints/              # Hint files and Rust concept references
└── benches/
    └── micro.rs
```
