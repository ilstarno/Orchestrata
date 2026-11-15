# Backend

The backend is a Rust workspace under `backend/`. Use the latest **stable** Rust toolchain supported by CI; Rust 2024 is the language edition, not a pinned compiler release.

- `crates/domain`: lifecycle types and invariants.
- `crates/solver`: finite-domain model, scoring, and reference search.
- `crates/api`: Axum HTTP transport.

## HTTP

- `GET /api/v1/health`
- `POST /api/v1/solve`

The development server binds to `127.0.0.1:3000`.

## MCP

MCP is a planned protocol adapter for discovering solver capabilities, validating problem definitions, submitting solve jobs, and reading results. See [MCP integration](../mcp/index.md). MCP and HTTP must share application services; neither protocol owns optimization rules.

## Boundaries

Keep the solver independent of transports, persistence, and domain-specific integrations. The current solve endpoint performs bounded exhaustive CPU work inside an async handler. Move execution to a bounded worker pool before production use. Authentication, tenant isolation, durable jobs, and cancellation are not implemented.
