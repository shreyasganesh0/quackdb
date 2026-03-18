# QuackDB Learning Metric Checkpoint

Generated: 2026-03-18 | Git SHA: 284b2c2 | Iteration: 3

---

## Architecture Summary

QuackDB is a 35-lesson Rust tutorial teaching distributed analytical database internals.
Each lesson has: source stubs with `todo!()` markers (src/), test files (tests/),
hint files with 6 standard sections (hints/), 20 enhanced Rust concept references
(hints/concepts/), and a multi-file lesson guide (hints/LESSON_GUIDE.md).
Lessons are feature-gated in Cargo.toml with strict linear dependencies.
Total: ~5,200 lines of source (65% scaffolding, 35% todo stubs), ~6,800 lines of tests
(430 tests), ~4,400 lines of hints, ~2,600 lines of concept references.
145 `todo!()` stubs remain across 42 source files (down from 267).

---

## Score Progression

| Metric | Iter 0 | Iter 1 | Iter 2 | Iter 3 |
|--------|--------|--------|--------|--------|
| Mean Overall | 66.7 | 68.6 | 84.1 | **90.9** |
| Curriculum Score | 64.7 | 66.6 | 82.5 | **89.9** |
| Stdev | 4.0 | 4.0 | 3.3 | **2.1** |
| Lessons ≥ 90 | 0 | 0 | 2 | **27** |
| Min lesson | 59 | 61 | 77 | **85** |

---

## Current Dimension Averages

| Dimension | Mean | Min | Max | Stdev | Status |
|-----------|------|-----|-----|-------|--------|
| Learnability | 94.3 | 93 | 97 | 1.0 | ✓ Near target |
| Approachability | 91.3 | 85 | 96 | 2.8 | ✓ Near target |
| Progression | 94.2 | 90 | 97 | 1.7 | ✓ Near target |
| Rust Friction | 89.5 | 85 | 95 | 2.6 | Needs +5.5 |
| Concept Density | 80.1 | 52 | 100 | 11.2 | Bottleneck |
| Test Ped Value | 88.9 | 86 | 92 | 1.9 | Needs +6.1 |

---

## Remaining Gap to 95

Current: 90.9 → Target: 95.0 → Gap: 4.1 points

The remaining gap comes primarily from:
1. **Concept Density** (80.1, weight 0.10) — L26=52, multi-file lessons at 60-74
2. **Rust Friction** (89.5, weight 0.15) — inherent complexity floor
3. **Test Ped Value** (88.9, weight 0.15) — near ceiling without more tests

To close the 4.1-point gap:
- Push Concept Density from 80→92 (+12) contributes +1.2 to overall
- Push Rust Friction from 89.5→96 (+6.5) contributes +0.98
- Push Test Ped Value from 88.9→95 (+6.1) contributes +0.92
- Push Approachability from 91.3→96 (+4.7) contributes +0.94
- Total potential: +4.0 → would reach 94.9

---

## Next Steps

1. **Concept Density**: The only path to 95+ requires addressing L26 (52) and
   multi-file lesson penalties. Options:
   - Add "unified concept" framing to multi-file lessons (+5-8 per lesson)
   - Reclassify L26 as a "capstone" with adjusted density expectations

2. **Rust Friction**: Enhanced concept files help but inherent complexity remains.
   - Add "Rust Sidebar" boxes in hints for the trickiest patterns
   - Pre-implement more unsafe boilerplate in stubs

3. **Test Ped Value**: Already at 88.9 with 430 tests and 59% message coverage.
   - Add more edge case tests to reach 95
   - Ensure all files have 12+ tests

4. **Final polish**: Minor improvements across all dimensions to close gap.
