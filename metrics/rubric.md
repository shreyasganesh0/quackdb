# QuackDB Learning Metric Rubric

## Overview

Each lesson is scored on 6 dimensions (0–100). The overall lesson score is a weighted
sum. The curriculum aggregate penalizes inconsistency via standard deviation.

```
Overall = 0.20×Learnability + 0.20×Approachability + 0.20×Progression
        + 0.15×RustFriction + 0.10×ConceptDensity + 0.15×TestPedValue

Curriculum Score = mean(overall_scores) - 0.5 × stdev(overall_scores)
```

---

## Dimension 1: Learnability (weight 0.20)

*How easy is it to understand what to build and how?*

| Signal | Measurement | Points |
|--------|-------------|--------|
| Hint section completeness | 5 standard sections present | 0–20 |
| Doc comment coverage | `///` lines / `pub fn` count on stubs | 0–15 |
| Concept reference count | Links in "Rust Concepts You'll Need" | 0–10 |
| Analogy count | Code analogies in "Key Patterns" | 0–15 |
| Assertion message ratio | `assert!(…, "msg")` / total asserts | 0–15 |
| "Reading the Tests" quality | Specific test explanations (2+ = full) | 0–15 |
| Test name descriptiveness | Names convey intent (manual judgment) | 0–10 |

**Scoring guide:**
- 90–100: All signals strong; a learner can understand intent from hints alone
- 70–89: Most signals present; minor gaps in assertion messages or doc comments
- 50–69: Some sections weak; assertion messages sparse; doc comments thin
- Below 50: Major gaps; learner must reverse-engineer intent from code

---

## Dimension 2: Approachability (weight 0.20)

*How welcoming to someone feeling intimidated?*

| Signal | Measurement | Points |
|--------|-------------|--------|
| Scaffolding ratio | Non-todo lines / total lines in stub | 0–20 |
| Todo count | 5–8 ideal; penalty outside range | 0–20 |
| First test simplicity | First test ≤10 lines = full marks | 0–15 |
| Error types pre-defined | Error enums/types provided (not todo) | 0–15 |
| Opening brevity | "What You're Building" ≤ 4 lines | 0–10 |
| Concept prerequisites listed | "Rust Concepts" links present | 0–10 |
| Step count in guide | 7–9 steps ideal | 0–10 |

**Todo count scoring:**
- 5–8 todos → 20
- 3–4 or 9–10 → 15
- 1–2 or 11–13 → 10
- 14+ → 5

---

## Dimension 3: Progression Flow (weight 0.20)

*How well does this lesson connect to what came before and after?*

| Signal | Measurement | Points |
|--------|-------------|--------|
| Type reuse from prior lessons | Uses types defined in earlier lessons | 0–25 |
| New concept count | 2–3 new Rust concepts = optimal | 0–15 |
| Concept overlap with prior | Shares ≥1 concept ref with L(N-1) | 0–15 |
| Import continuity | Uses modules from prior parts | 0–15 |
| Part boundary bridging | At boundaries: back-ref to prior part | 0–20 |
| Forward setup | Introduces patterns used in next lesson | 0–10 |

**Part boundaries** (score ×1.3 urgency):
- L04→L05 (Foundations → Compression)
- L08→L09 (Compression → Storage)
- L12→L13 (Storage → Execution)
- L19→L20 (Execution → SQL)
- L24→L25 (Planning → Optimization)
- L26→L27 (Optimization → Transactions)
- L28→L29 (Transactions → Parallelism)
- L33→L34 (Distribution → Advanced)

---

## Dimension 4: Rust Friction (weight 0.15, inverse)

*How much does Rust complexity obscure the DB concept?*
100 = no friction (Rust is invisible); 0 = Rust dominates.

| Signal | Measurement | Points (deducted from 100) |
|--------|-------------|--------------------------|
| Unsafe block count | Each unsafe block: -8 | -0 to -24 |
| Lifetime annotations | Each `'a` annotation: -3 | -0 to -18 |
| Generic complexity | Each `<T: Trait>` bound: -2 | -0 to -20 |
| Concept coverage | Needed Rust concepts have refs | +0 to +15 |
| Rust-vs-DB effort ratio | Manual judgment of where time goes | -0 to -20 |

**Scoring guide:**
- 80–100: Rust is just syntax; focus is on DB concept
- 60–79: Some Rust complexity but well-explained in hints
- 40–59: Significant Rust hurdles (lifetimes, generics, unsafe)
- Below 40: Rust dominates; DB concept is secondary

---

## Dimension 5: Concept Density (weight 0.10)

*How many new concepts does this lesson introduce?*

| New Concepts | Score |
|-------------|-------|
| 2 | 100 |
| 3 | 85 |
| 1 | 60 |
| 4 | 60 |
| 5 | 35 |
| 0 | 60 |
| 6+ | 10 |

**Penalties:**
- Multi-file lesson (2 files): -10
- Multi-file lesson (3+ files): -25

New concepts counted: DB concepts + new Rust patterns not seen in prior lessons.

---

## Dimension 6: Test Pedagogical Value (weight 0.15)

*How well do tests teach through their structure?*

| Signal | Measurement | Points |
|--------|-------------|--------|
| Test count | 8–15 ideal | 0–15 |
| Helper functions | Present = full marks | 0–10 |
| Edge case tests | ≥2 edge cases tested | 0–10 |
| Assertion messages | ≥40% of asserts have messages | 0–20 |
| Test progression | Simple → complex ordering | 0–15 |
| Lines per test | 8–15 avg = optimal | 0–15 |
| Test names | Descriptive, convey intent | 0–15 |

**Test count scoring:**
- 8–15 tests → 15
- 5–7 or 16–18 → 10
- 1–4 or 19+ → 5

**Lines-per-test scoring:**
- 8–15 lines → 15
- 5–7 or 16–20 → 10
- <5 or >20 → 5

---

## Priority Formula

For improvement targeting:

```
Priority(lesson, dim) = Impact × Feasibility × Urgency

Impact      = (100 - score) × weight
Feasibility = 1.0 / estimated_effort  (1=easy, 0.5=medium, 0.25=hard)
Urgency     = position_weight × dependency_factor

Position weights:
  L01–L04 = 1.5  (foundations, everyone hits these)
  L05–L12 = 1.2  (early-mid)
  L13–L24 = 1.0  (mid)
  L25–L35 = 0.8  (late)

Dependency factor:
  Part boundary = 1.3
  Otherwise     = 1.0
```

---

## Convergence Criteria

Stop iterating when any of:
- Curriculum Score ≥ 80
- Improvement < 1 point over 3 consecutive iterations
- All lessons score ≥ 60 on all dimensions
