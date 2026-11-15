# Backend

The Rust 2024 workspace is under `backend/`.

- `crates/domain`: job lifecycle state and domain invariants.
- `crates/solver`: problem types, validation, scoring, and reference search.
- `crates/api`: Axum HTTP transport.

## HTTP

- `GET /api/v1/health`: health response.
- `POST /api/v1/solve`: accepts a solver `Problem` and returns an optional solution.

The development server binds to `127.0.0.1:3000`.

## Boundaries

Keep HTTP-specific types out of the solver. Reject invalid inputs before solving. Treat request limits, cancellation, authentication, tenant isolation, and durable execution as unfinished production concerns.

## Gotchas

- A successful HTTP response does not imply the returned solution has a zero hard penalty.
- The solve endpoint currently performs CPU-bound exhaustive work inside an async handler; isolate bounded blocking execution before accepting concurrent production traffic.
