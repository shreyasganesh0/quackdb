# QuackDB Learning Metric Checkpoint

Generated: 2026-03-18 | Git SHA: 86226a8 | Iteration: 1

---

## Architecture Summary

QuackDB is a 35-lesson Rust tutorial teaching distributed analytical database internals.
Each lesson has: source stubs with `todo!()` markers (src/), test files (tests/),
hint files with 5-6 standard sections (hints/), and 20 Rust concept references (hints/concepts/).
Lessons are feature-gated in Cargo.toml with strict linear dependencies.
The curriculum spans 9 parts: Foundations, Compression, Storage, Execution, SQL,
Optimization, Transactions, Parallelism, and Advanced topics.
Total: ~3,900 lines of source stubs, ~5,500 lines of test code, ~3,100 lines of hints.
267 `todo!()` stubs across 38 source files. 406 test functions across 35 test files.

---

## Current Scores

| Lesson | Learn | Approach | Progress | RustFric | Density | TestPed | Overall | MC Mean±SD |
|--------|-------|----------|----------|----------|---------|---------|---------|------------|
| L01 Arena Allocator | 72 | 65 | 62 | 55 | 85 | 80 | 69 | 68.0±5.0 |
| L02 Data Types | 75 | 68 | 65 | 80 | 85 | 76 | 74 | 73.5±3.6 |
| L03 Columnar Vectors | 73 | 62 | 70 | 55 | 85 | 73 | 69 | 68.3±5.3 |
| L04 Data Chunks | 74 | 72 | **81** | 82 | 100 | 76 | **79** | 78.5±3.0 |
| L05 RLE Compression | 70 | 74 | 68 | 78 | 85 | 70 | 73 | 72.6±3.8 |
| L06 Dictionary Encoding | 72 | 72 | 72 | 75 | 85 | 73 | 74 | 73.5±3.3 |
| L07 Bitpacking & Delta | 68 | 58 | 75 | 72 | 60 | 70 | 68 | 67.0±4.0 |
| L08 Compression Framework | 70 | 65 | **84** | 75 | 85 | 78 | **75** | 74.8±3.2 |
| L09 Pages & Layout | 72 | 60 | 72 | 70 | 85 | 76 | 71 | 70.8±3.9 |
| L10 Buffer Pool | 74 | 58 | 75 | 65 | 60 | 76 | 69 | 68.0±4.8 |
| L11 Columnar Writer | 68 | 56 | 72 | 65 | 85 | 66 | 67 | 66.4±4.6 |
| L12 Columnar Reader | 70 | 62 | **84** | 65 | 85 | 68 | 72 | 71.4±4.1 |
| L13 Expressions | 72 | 68 | 68 | 72 | 85 | 73 | 72 | 71.5±3.4 |
| L14 Pipelines | 70 | 62 | 72 | 68 | 85 | 66 | 69 | 68.7±3.8 |
| L15 Scan/Filter/Project | 68 | 55 | 75 | 70 | 60 | 68 | 66 | 65.7±4.6 |
| L16 Hash Aggregation | 72 | 60 | 72 | 65 | 85 | 76 | 70 | 69.7±4.1 |
| L17 Hash Join | 70 | 58 | 78 | 72 | 85 | 76 | 72 | 71.4±4.3 |
| L18 Sort-Merge Join | 72 | 55 | 78 | 62 | 85 | 73 | 70 | 69.2±5.0 |
| L19 External Sort | 70 | 55 | **78** | 62 | 60 | 76 | 67 | 66.5±4.3 |
| L20 SQL Lexer | 72 | 68 | **63** | 75 | 85 | 70 | 71 | 70.3±4.5 |
| L21 SQL Parser | 68 | 52 | 78 | 62 | 60 | 68 | 65 | 64.4±5.1 |
| L22 Logical Plan | 70 | 65 | 72 | 68 | 85 | 66 | 70 | 69.6±3.7 |
| L23 Binder & Catalog | 72 | 58 | 72 | 58 | 60 | 70 | 66 | 65.0±4.8 |
| L24 Physical Plan | 68 | 62 | **84** | 58 | 85 | 63 | 69 | 68.6±4.9 |
| L25 Rule Optimizer | 70 | 55 | 72 | 70 | 60 | 66 | 66 | 65.4±4.2 |
| L26 Cost Optimizer | 65 | 48 | **78** | 65 | 35 | 66 | 61 | 60.7±5.4 |
| L27 MVCC | 72 | 55 | 62 | 62 | 85 | 70 | 66 | 65.7±4.6 |
| L28 WAL | 70 | 60 | **81** | 68 | 85 | 63 | 70 | 69.5±4.0 |
| L29 Morsel Parallelism | 68 | 52 | 68 | 55 | 60 | 63 | 61 | 61.0±5.3 |
| L30 Window Functions | 70 | 55 | 65 | 62 | 85 | 70 | 66 | 65.8±4.3 |
| L31 Partitioning | 70 | 60 | 68 | 72 | 85 | 66 | 69 | 68.2±3.9 |
| L32 Distributed Plan | 65 | 58 | 72 | 68 | 85 | 63 | 67 | 66.5±4.1 |
| L33 Shuffle & Exchange | 65 | 50 | **81** | 55 | 60 | 63 | 63 | 62.3±5.6 |
| L34 Adaptive Execution | 68 | 55 | 68 | 68 | 60 | 66 | 64 | 63.7±4.2 |
| L35 SIMD Vectorization | 68 | 52 | 62 | 52 | 85 | 63 | 62 | 61.5±5.2 |

**Bold** = changed from iteration 0.

---

## Dimension Averages

| Dimension | Mean | Min | Max | Stdev | Δ from iter 0 |
|-----------|------|-----|-----|-------|---------------|
| Learnability | 70.1 | 65 | 75 | 2.7 | — |
| Approachability | 59.7 | 48 | 74 | 6.8 | — |
| Progression | 72.9 | 62 | 84 | 6.5 | **+2.2** |
| Rust Friction | 66.1 | 52 | 82 | 7.5 | — |
| Concept Density | 76.7 | 35 | 100 | 15.5 | — |
| Test Ped Value | 70.1 | 63 | 80 | 5.0 | **+8.0** |

---

## Curriculum Aggregate Score

```
Mean Overall:     68.6  (+1.9 from iter 0)
Stdev Overall:     4.0
Curriculum Score: 66.6  (+1.9 from iter 0)
```

Target: ≥ 80. Distance: 13.4 points (was 15.3).

---

## Weakest Links (Top 10 Priority Pairs)

| Rank | Lesson | Dimension | Score | Proposed Fix |
|------|--------|-----------|-------|--------------|
| 1 | L26 | Approachability | 48 | Reduce todo count, add scaffolding, pre-define types |
| 2 | L26 | Concept Density | 35 | Split into sub-steps or reduce scope |
| 3 | L33 | Approachability | 50 | Pre-define channel wrappers, reduce concurrency complexity |
| 4 | L29 | Approachability | 52 | Pre-implement thread spawn boilerplate |
| 5 | L21 | Approachability | 52 | Pre-implement parser helpers |
| 6 | L35 | Approachability | 52 | Better unsafe explanation, more scaffolding |
| 7 | L35 | Rust Friction | 52 | Add inline comments for unsafe aligned_alloc |
| 8 | L01 | Rust Friction | 55 | Expand unsafe concept ref linking |
| 9 | L03 | Rust Friction | 55 | Reduce unsafe complexity in hints |
| 10 | L29 | Rust Friction | 55 | Better concurrency pattern explanation |

---

## Changes Made This Iteration

### Test Assertion Messages (all 35 lessons)
- Added ~285 pedagogical assertion messages across all test files
- Assertion message ratio: ~15% → ~40%
- TestPedValue dimension: mean 62.1 → 70.1 (+8.0)
- Commit: `metrics(L01-L35): add pedagogical assertion messages across all test files`

### Part Boundary Bridging (10 hint files)
- Added "What Comes Next" sections to: L01, L04, L08, L12, L19, L24, L26, L28, L33
- Added "Connection to Part IV" bridge to L20
- Progression dimension: mean 70.7 → 72.9 (+2.2)
- Commit: `metrics(hints): add part-boundary bridging sections to hint files`

### Score Deltas
- Overall mean: 66.7 → 68.6 (+1.9)
- Curriculum score: 64.7 → 66.6 (+1.9)
- No lessons below 60 (was: L26=59, L29=60)
- Lessons above 75: L04=79, L08=75 (was: L04=77)

---

## Context for Next Iteration

**What worked:**
- Assertion messages gave the biggest single-dimension improvement (+8 avg)
- Part boundary bridging was high ROI (small text additions, +6 per boundary lesson)
- Systematic approach (all 35 files at once) was more efficient than lesson-by-lesson

**What didn't:**
- Approachability remains the weakest dimension (mean 59.7, unchanged)
- Rust Friction was not addressed this iteration
- Late lessons (L29-L35) still cluster near the bottom

**Risks:**
- L26 still weakest at 61; concept density (35) remains the single worst score
- Approachability requires structural changes (reducing todos, adding scaffolding)
  which are more invasive than text additions
- Diminishing returns on text-only improvements going forward

---

## Next Steps (Prioritized)

1. **Iteration 2: Approachability sweep** — L26 (48), L33 (50), L29 (52), L21 (52),
   L35 (52). Requires reducing todo counts and adding scaffolding in source stubs.

2. **Iteration 2: Rust Friction** — L35 (52), L01 (55), L03 (55), L29 (55).
   Add inline comments explaining unsafe/lifetime/generic patterns.

3. **Iteration 3: Concept Density** — L26 (35) needs restructuring or scope reduction.

4. **Iteration 3: Learnability fine-tuning** — Add more specific test explanations
   in "Reading the Tests" sections for lessons below 70.

---

## Research Notes

**Applied this iteration:**
- Assertion messages as micro-documentation (from test-driven learning research)
- Part boundary bridging (from cognitive load theory: reduce context-switch cost)

**Queue for next iteration:**
- Rustlings scaffolding patterns for progressive difficulty
- Vygotsky's zone of proximal development: each lesson stretches exactly one level
- CMU 15-445 approach to splitting dense topics
