# QuackDB Learning Metric Checkpoint

Generated: 2026-03-18 | Git SHA: 8a36d6d | Iteration: 0 (Bootstrap)

---

## Architecture Summary

QuackDB is a 35-lesson Rust tutorial teaching distributed analytical database internals.
Each lesson has: source stubs with `todo!()` markers (src/), test files (tests/),
hint files with 5 standard sections (hints/), and 20 Rust concept references (hints/concepts/).
Lessons are feature-gated in Cargo.toml with strict linear dependencies.
The curriculum spans 9 parts: Foundations, Compression, Storage, Execution, SQL,
Optimization, Transactions, Parallelism, and Advanced topics.
Total: ~3,900 lines of source stubs, ~5,500 lines of test code, ~3,000 lines of hints.
267 `todo!()` stubs across 38 source files. 406 test functions across 35 test files.

---

## Current Scores

| Lesson | Learn | Approach | Progress | RustFric | Density | TestPed | Overall | MC Mean±SD |
|--------|-------|----------|----------|----------|---------|---------|---------|------------|
| L01 Arena Allocator | 72 | 65 | 55 | 55 | 85 | 72 | 66 | 65.2±5.1 |
| L02 Data Types | 75 | 68 | 65 | 80 | 85 | 68 | 73 | 72.5±3.8 |
| L03 Columnar Vectors | 73 | 62 | 70 | 55 | 85 | 65 | 68 | 67.1±5.5 |
| L04 Data Chunks | 74 | 72 | 75 | 82 | 100 | 68 | 77 | 76.3±3.2 |
| L05 RLE Compression | 70 | 74 | 68 | 78 | 85 | 62 | 72 | 71.4±4.0 |
| L06 Dictionary Encoding | 72 | 72 | 72 | 75 | 85 | 65 | 73 | 72.3±3.5 |
| L07 Bitpacking & Delta | 68 | 58 | 75 | 72 | 60 | 62 | 66 | 65.8±4.2 |
| L08 Compression Framework | 70 | 65 | 78 | 75 | 85 | 70 | 73 | 72.4±3.4 |
| L09 Pages & Layout | 72 | 60 | 72 | 70 | 85 | 68 | 70 | 69.6±4.1 |
| L10 Buffer Pool | 74 | 58 | 75 | 65 | 60 | 68 | 67 | 66.8±5.0 |
| L11 Columnar Writer | 68 | 56 | 72 | 65 | 85 | 58 | 66 | 65.2±4.8 |
| L12 Columnar Reader | 70 | 62 | 78 | 65 | 85 | 60 | 69 | 68.8±4.3 |
| L13 Expressions | 72 | 68 | 68 | 72 | 85 | 65 | 71 | 70.3±3.6 |
| L14 Pipelines | 70 | 62 | 72 | 68 | 85 | 58 | 68 | 67.5±4.0 |
| L15 Scan/Filter/Project | 68 | 55 | 75 | 70 | 60 | 60 | 65 | 64.5±4.8 |
| L16 Hash Aggregation | 72 | 60 | 72 | 65 | 85 | 68 | 69 | 68.5±4.3 |
| L17 Hash Join | 70 | 58 | 78 | 72 | 85 | 68 | 71 | 70.2±4.5 |
| L18 Sort-Merge Join | 72 | 55 | 78 | 62 | 85 | 65 | 69 | 68.0±5.2 |
| L19 External Sort | 70 | 55 | 72 | 62 | 60 | 68 | 65 | 64.3±4.5 |
| L20 SQL Lexer | 72 | 68 | 55 | 75 | 85 | 62 | 68 | 68.1±4.7 |
| L21 SQL Parser | 68 | 52 | 78 | 62 | 60 | 60 | 64 | 63.2±5.3 |
| L22 Logical Plan | 70 | 65 | 72 | 68 | 85 | 58 | 69 | 68.4±3.9 |
| L23 Binder & Catalog | 72 | 58 | 72 | 58 | 60 | 62 | 64 | 63.8±5.0 |
| L24 Physical Plan | 68 | 62 | 78 | 58 | 85 | 55 | 67 | 66.4±5.1 |
| L25 Rule Optimizer | 70 | 55 | 72 | 70 | 60 | 58 | 65 | 64.2±4.4 |
| L26 Cost Optimizer | 65 | 48 | 72 | 65 | 35 | 58 | **59** | 58.5±5.6 |
| L27 MVCC | 72 | 55 | 62 | 62 | 85 | 62 | 65 | 64.5±4.8 |
| L28 WAL | 70 | 60 | 75 | 68 | 85 | 55 | 68 | 67.3±4.2 |
| L29 Morsel Parallelism | 68 | 52 | 68 | 55 | 60 | 55 | **60** | 59.8±5.5 |
| L30 Window Functions | 70 | 55 | 65 | 62 | 85 | 62 | 65 | 64.6±4.5 |
| L31 Partitioning | 70 | 60 | 68 | 72 | 85 | 58 | 68 | 67.0±4.1 |
| L32 Distributed Plan | 65 | 58 | 72 | 68 | 85 | 55 | 66 | 65.3±4.3 |
| L33 Shuffle & Exchange | 65 | 50 | 75 | 55 | 60 | 55 | 61 | 60.1±5.8 |
| L34 Adaptive Execution | 68 | 55 | 68 | 68 | 60 | 58 | 63 | 62.5±4.4 |
| L35 SIMD Vectorization | 68 | 52 | 62 | 52 | 85 | 55 | 61 | 60.3±5.4 |

---

## Dimension Averages

| Dimension | Mean | Min | Max | Stdev |
|-----------|------|-----|-----|-------|
| Learnability | 70.1 | 65 | 75 | 2.7 |
| Approachability | 59.7 | 48 | 74 | 6.8 |
| Progression | 70.7 | 55 | 78 | 5.8 |
| Rust Friction | 66.1 | 52 | 82 | 7.5 |
| Concept Density | 76.7 | 35 | 100 | 15.5 |
| Test Ped Value | 62.1 | 55 | 72 | 5.0 |

---

## Curriculum Aggregate Score

```
Mean Overall:     66.7
Stdev Overall:     4.0
Curriculum Score: 64.7  (mean - 0.5 × stdev)
```

Target: ≥ 80. Distance: 15.3 points.

---

## Weakest Links (Top 10 Priority Pairs)

Priority = (100 - score) × weight × position_weight × dependency_factor

| Rank | Lesson | Dimension | Score | Priority | Proposed Fix |
|------|--------|-----------|-------|----------|--------------|
| 1 | L26 | Approachability | 48 | High | Reduce todo count (13→8), add more scaffolding |
| 2 | L26 | Concept Density | 35 | High | Split into 2 sub-lessons or reduce scope (3 files) |
| 3 | L01 | Progression | 55 | High | Add "What Comes Next" section, preview L02 types |
| 4 | L01 | Rust Friction | 55 | High | Add more unsafe explanation, link to concept ref more explicitly |
| 5 | L03 | Approachability | 62 | High | Reduce 20 todos to ~10 by pre-implementing boilerplate |
| 6 | L33 | Approachability | 50 | Medium | Pre-define channel wrapper types, reduce concurrency friction |
| 7 | L29 | Approachability | 52 | Medium | Pre-implement thread spawn boilerplate, add more scaffolding |
| 8 | L21 | Approachability | 52 | Medium | Pre-implement parser helpers, reduce from 10 todos |
| 9 | L20 | Progression | 55 | Medium | Add bridge from L19 execution to SQL frontend motivation |
| 10 | L35 | Rust Friction | 52 | Medium | Better inline comments for unsafe aligned_alloc |

---

## Changes Made This Iteration

Bootstrap (Iteration 0) — no changes to lesson content.

Files created:
- `CHECKPOINT.md` — this file
- `metrics/rubric.md` — full scoring rubric
- `metrics/profiles.md` — learner profile definitions
- `metrics/scores.json` — machine-readable scores

---

## Context for Next Iteration

**What worked:**
- Hint file structure is consistent across all 35 lessons (5 standard sections)
- Concept reference library (20 files) provides good Rust coverage
- Test counts are generally adequate (avg 11.6 per lesson)
- Feature-gated linear progression is well-designed

**What didn't:**
- Assertion messages are sparse (~15% of asserts have messages)
- Approachability is the weakest dimension (mean 59.7, min 48)
- Several multi-file lessons have high concept density (L07, L15, L26)
- Part boundaries lack explicit bridging content
- Late lessons (L29-L35) show consistent quality drop

**Risks:**
- L26 (Cost Optimizer) at 59 is below threshold; may need restructuring
- Concept Density has very high stdev (15.5) — some lessons overloaded, others thin
- Rust Friction stdev (7.5) indicates inconsistent handling of Rust complexity

---

## Next Steps (Prioritized)

1. **Iteration 1: Approachability sweep** — Focus on L26, L33, L29, L21 (lowest approachability scores). Add scaffolding, reduce todo counts, pre-define error types.

2. **Iteration 1: Assertion messages** — Add assertion messages to test files across all lessons. Target: ≥40% of asserts have messages. Start with L01-L04 (highest impact due to position weight).

3. **Iteration 2: Part boundary bridging** — Add "What Comes Next" sections to L04, L08, L12, L19, L24, L26, L28, L33 hint files.

4. **Iteration 2: L26 restructuring** — Consider splitting cost optimizer into sub-steps or pre-implementing base structures to reduce concept density.

5. **Iteration 3: Rust Friction mitigation** — Improve concept ref linking for high-friction lessons (L01, L03, L35). Add inline comments explaining Rust mechanisms.

---

## Research Notes

**Applied this iteration:** None (bootstrap)

**Queue for next iteration:**
- Rustlings scaffolding patterns for progressive difficulty
- CMU 15-445 teaching progression for DB internals
- Miller's 7±2 rule: verify no lesson exceeds 4 new concepts
- Test-as-documentation patterns from Exercism
