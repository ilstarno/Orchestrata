# 0001 — Rust core and separate web application

Status: accepted

## Decision

Implement the backend in Rust 2024 and the frontend in React/TypeScript. Keep solver code independent of HTTP, persistence, and UI. Use a versioned HTTP API between separately buildable applications.

## Why

Solver correctness and performance must be testable without a web runtime. The UI can evolve independently without changing solver internals.

## Consequences

Cross-boundary types need explicit contracts. API client generation and compatibility checks are still to be implemented.
