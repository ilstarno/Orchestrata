# 0002 — Reference scorer before incremental optimization

Status: accepted

## Decision

Implement full score recalculation and bounded exhaustive search before incremental scoring and metaheuristics.

## Why

The reference evaluator provides a correctness oracle for randomized move sequences and future optimizations.

## Consequences

The initial solver scales poorly and is suitable only for small fixtures. New scoring implementations must match the reference evaluator on identical assignments.
