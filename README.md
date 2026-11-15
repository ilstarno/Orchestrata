# Orchestrata

Orchestrata is a constraint-based planning and optimization platform. It is designed to model resources, assignments, objectives, and operational rules, then produce solutions that can be checked against those rules.

The project is under active development. The current repository is a foundation, not a production-ready solver or integration service.

## Architecture

The backend is written in Rust 2024. The frontend uses React, TypeScript, Vite, and TanStack libraries. They are developed and deployed independently, with a versioned HTTP API between them.

The intended backend components are:

- **Domain:** typed planning models, constraints, and invariants.
- **Compiler:** validation and compilation of problem definitions.
- **Solver:** optimization algorithms and incremental scoring.
- **Validation:** independent verification of feasibility and results.
- **Orchestration:** execution, cancellation, recovery, and audit.
- **Connectors:** controlled data exchange with external systems.

PostgreSQL is the planned transactional store. MariaDB, Microsoft SQL Server, and Snowflake are planned data-source and destination integrations. They are not interchangeable control-plane databases.

## Repository layout

```text
backend/    Rust workspace and API
frontend/   React application
docs/       Architecture and design decisions
.github/    CI workflows
```

## Development

Requires a recent stable Rust toolchain, Node.js 22, and npm.

Run the backend:

```sh
cd backend
cargo run -p orchestrata-api
```

Run the frontend in a separate terminal, once the frontend application is present:

```sh
cd frontend
npm install
npm run dev
```

The development API listens on `127.0.0.1:3000`. The planned frontend development server proxies `/api` to that address.

## Engineering principles

- Keep the solver independent of HTTP, storage, and UI concerns.
- Validate constraints before execution and verify solutions independently.
- Prefer explicit types, ownership, and error handling.
- Isolate database-specific behavior behind adapters.
- Keep credentials server-side; require explicit authorization for external writes.
- Benchmark performance claims and test failure recovery.

## Status

The first milestone is a working end-to-end planning fixture: submit a typed problem, validate its constraints, solve it, verify the result, and display it in the frontend. Database connectors and production orchestration follow after the core path is tested.

Proprietary project. No license is granted for reuse or redistribution.
