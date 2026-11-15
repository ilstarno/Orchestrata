# Orchestrata

**Constraint optimization for problems with competing rules and objectives.**

Orchestrata is a general-purpose planning and optimization platform. It is intended to let applications define decisions, allowed values, mandatory rules, and preferences, then search for a solution and verify its score. The solver is not tied to a particular industry or planning workflow.

Examples include assigning work to resources, scheduling shifts, sequencing production, routing deliveries, allocating capacity, and arranging tasks with dependencies. These are examples of problems the platform aims to support, not features all available today.

## How a problem is defined

A problem supplies variables and their possible values, constraints that must hold, and objectives that distinguish better solutions. For example, a resource-allocation problem might assign tasks to machines, forbid assignments that exceed capacity, and prefer lower operating cost. A scheduling problem might use different variables and rules but run through the same solver interface.

The intended processing path is **model → validate → compile constraints → solve → independently verify → return results**. Domain-specific systems can supply inputs and consume results without changing the solver core.

## Current functionality

The Rust reference solver supports integer-valued variables, all-different constraints, offset inequalities, and value preferences. It uses exhaustive search for small problems, with hard and soft scores. The HTTP endpoint limits searches to 100,000 combinations. This establishes correctness tests, not production-scale optimization.

## Interfaces

The React web application provides an editable JSON problem runner. The Rust backend exposes a versioned HTTP API. **MCP is planned** as an additional backend protocol interface for validating problems, submitting jobs, and retrieving solutions; it will use the same application services as HTTP. No MCP server is implemented yet.

## Implementation

The backend uses the latest stable Rust toolchain with the Rust 2024 edition, with the solver kept separate from HTTP and database code. The web application uses current stable React, TypeScript, Vite, and TanStack libraries, with compatible versions locked for reproducible builds. Frontend and backend are independently buildable and communicate through a versioned HTTP API.

PostgreSQL is the planned operational database. MariaDB, Microsoft SQL Server, and Snowflake are planned integrations for reading and exchanging business data; they are not presented as interchangeable transactional databases.

```text
backend/    Rust domain, solver, and API work
frontend/   React application
docs/       Technical decisions
.github/    Continuous integration
```

## Current status

**Early development.** The repository contains a Rust reference solver and a React interface for submitting small finite-domain problems. It does not yet demonstrate the field-service example, scalable optimization, database integrations, or production scheduling.

The current solver accepts integer-domain variables and three constraint types: all-different, unequal offset, and preferred value. It evaluates hard and soft penalties and exhaustively searches at most 100,000 combinations through the API. N-Queens and assignment tests exercise the same evaluator. This is a correctness baseline, not a scalable optimization algorithm. The next milestone adds a typed constraint compiler, move operators, local search, and independent validation.

## Development

Backend (requires a recent stable Rust toolchain):

```sh
cd backend
cargo run -p orchestrata-api
```

Frontend (requires Node.js 22 and npm):

```sh
cd frontend
npm install
npm run dev
```

The development API is configured for `127.0.0.1:3000`, with `/api` proxied from the frontend during local development.

---

Proprietary software. No license is granted for reuse or redistribution.
