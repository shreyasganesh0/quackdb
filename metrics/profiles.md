# QuackDB Learner Profiles

## Overview

Seven learner profiles model the diversity of QuackDB's audience. Each profile
applies multipliers to dimension weights, reflecting what matters most to that
type of learner. The Monte Carlo evaluation samples N=50 profiles per lesson
from this weighted distribution.

---

## Profile Definitions

### P1: Rust Novice, DB Novice (weight 0.20)

First-time Rustacean who has heard of databases but never implemented one.
Needs maximum hand-holding on both Rust mechanics and DB concepts.

| Dimension | Multiplier | Rationale |
|-----------|-----------|-----------|
| Learnability | 1.0 | Standard |
| Approachability | 1.5 | Intimidation is the #1 dropout risk |
| Progression | 1.0 | Standard |
| Rust Friction | 1.5 | Every borrow-checker error is a wall |
| Concept Density | 1.0 | Standard |
| Test Ped Value | 1.0 | Standard |

### P2: Rust Novice, DB Expert (weight 0.20)

Experienced database engineer learning Rust through this project. Knows what
a buffer pool is; struggles with lifetimes and trait bounds.

| Dimension | Multiplier | Rationale |
|-----------|-----------|-----------|
| Learnability | 1.0 | Standard |
| Approachability | 1.0 | DB confidence offsets some intimidation |
| Progression | 1.0 | Standard |
| Rust Friction | 1.8 | This is their primary pain point |
| Concept Density | 1.0 | Standard |
| Test Ped Value | 1.0 | Standard |

### P3: Rust Expert, DB Novice (weight 0.15)

Experienced Rustacean exploring database internals. Comfortable with unsafe,
lifetimes, and generics; needs clear DB concept explanations.

| Dimension | Multiplier | Rationale |
|-----------|-----------|-----------|
| Learnability | 1.3 | DB concepts need extra clarity |
| Approachability | 1.0 | Standard |
| Progression | 1.0 | Standard |
| Rust Friction | 0.5 | Rust is easy for them |
| Concept Density | 1.0 | Standard |
| Test Ped Value | 1.0 | Standard |

### P4: Rust Expert, DB Expert (weight 0.05)

Rare but exists. Wants a well-structured challenge; impatient with over-explanation.

| Dimension | Multiplier | Rationale |
|-----------|-----------|-----------|
| Learnability | 1.0 | Baseline |
| Approachability | 1.0 | Baseline |
| Progression | 1.0 | Baseline |
| Rust Friction | 1.0 | Baseline |
| Concept Density | 1.0 | Baseline |
| Test Ped Value | 1.0 | Baseline |

### P5: Intermediate Both (weight 0.25)

The target audience. Comfortable with basic Rust and has taken a databases
course. This is the largest segment and the baseline for scoring.

| Dimension | Multiplier | Rationale |
|-----------|-----------|-----------|
| Learnability | 1.0 | Baseline (target audience) |
| Approachability | 1.0 | Baseline |
| Progression | 1.0 | Baseline |
| Rust Friction | 1.0 | Baseline |
| Concept Density | 1.0 | Baseline |
| Test Ped Value | 1.0 | Baseline |

### P6: Impatient Expert (weight 0.05)

Highly skilled, wants to move fast. Values test clarity and compact hints
over lengthy explanations.

| Dimension | Multiplier | Rationale |
|-----------|-----------|-----------|
| Learnability | 0.7 | Reads code, not prose |
| Approachability | 1.0 | Standard |
| Progression | 1.0 | Standard |
| Rust Friction | 1.0 | Standard |
| Concept Density | 1.0 | Standard |
| Test Ped Value | 1.5 | Tests ARE the documentation |

### P7: Methodical Beginner (weight 0.10)

Careful learner who reads every hint section, follows every link, and wants
to understand deeply before writing code.

| Dimension | Multiplier | Rationale |
|-----------|-----------|-----------|
| Learnability | 1.5 | Thoroughness demands completeness |
| Approachability | 1.5 | Needs gentle on-ramp |
| Progression | 1.0 | Standard |
| Rust Friction | 1.0 | Standard |
| Concept Density | 1.0 | Standard |
| Test Ped Value | 1.0 | Standard |

---

## Monte Carlo Evaluation Protocol

For each lesson:

1. **Sample** 50 profiles from the weighted distribution above
2. **Compute** the modulated score for each sample:
   - Apply the profile's multipliers to the base dimension weights
   - Renormalize weights to sum to 1.0
   - Compute the weighted overall score
3. **Report** mean ± stdev of the 50 samples
4. **Flag** lessons where stdev > 8 (works for some profiles but not others)

### Effective Weights per Profile (after renormalization)

| Profile | Learn | Approach | Progress | RustFric | Density | TestPed |
|---------|-------|----------|----------|----------|---------|---------|
| P1 | 0.170 | 0.255 | 0.170 | 0.191 | 0.085 | 0.128 |
| P2 | 0.174 | 0.174 | 0.174 | 0.235 | 0.087 | 0.130 |
| P3 | 0.236 | 0.182 | 0.182 | 0.068 | 0.091 | 0.136 |
| P4 | 0.200 | 0.200 | 0.200 | 0.150 | 0.100 | 0.150 |
| P5 | 0.200 | 0.200 | 0.200 | 0.150 | 0.100 | 0.150 |
| P6 | 0.127 | 0.182 | 0.182 | 0.136 | 0.091 | 0.205 |
| P7 | 0.231 | 0.231 | 0.154 | 0.115 | 0.077 | 0.115 |

---

## Interpreting Results

- **High mean, low stdev**: Lesson works well for everyone
- **High mean, high stdev**: Lesson is great for some but poor for others
- **Low mean, low stdev**: Uniformly weak — needs improvement for all
- **Low mean, high stdev**: Polarizing — some profiles really struggle

Target: all lessons ≥ 65 mean with stdev ≤ 8.
