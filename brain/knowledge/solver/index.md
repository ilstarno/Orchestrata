# Solver

Source: `backend/crates/solver/src/lib.rs`.

## Current execution

`validate` checks domain emptiness, constraint weights, and referenced variable indices. `score` checks assignments and calculates a lexicographic `Score { hard, soft }`. `solve` enumerates combinations up to a supplied limit, evaluates each, and retains the best.

The API caps the search space at 100,000 assignments. The library itself accepts a caller-provided cap.

## Constraints

- `AllDifferent`: penalizes each equal-valued pair.
- `NotEqualOffset`: penalizes an equality between the difference of two values and a specified offset.
- `PreferValue`: penalizes choosing a value other than the preferred one.

## Tests

The existing suite covers four queens, infeasible three queens, a two-job assignment, and invalid variable references.

## Next work

Introduce a typed intermediate representation, explicit move operators and undo semantics, construction heuristics, local search, and incremental scoring. Differential-test incremental scoring against `score` after randomized moves.

## Gotchas

- Full recomputation and exhaustive enumeration are correctness scaffolding, not scalable optimization.
- Current scoring uses saturating subtraction; define and test overflow semantics before introducing larger weights.
- Constraint references are numeric indices; a compiled symbol table is needed before exposing a richer external DSL.
