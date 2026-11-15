# Orchestrata

**Plan work when resources, deadlines, and rules compete.**

Orchestrata is a planning and optimization system for decisions that are difficult to make by hand: which technician should take each job, which vehicle should handle each delivery, or how production work should be assigned to machines.

These decisions have constraints. A technician needs the right skills and must be available. A vehicle cannot exceed its capacity. A job may have a deadline or require another job to finish first. Several valid schedules may exist, but some involve more travel, overtime, or late work than others.

Orchestrata is being built to evaluate those trade-offs and find a workable allocation.

## Example: scheduling field-service jobs

A service company has 40 jobs to complete tomorrow and 8 technicians. Each job has a location, duration, time window, and required skills. Each technician has working hours, qualifications, and a starting location.

The intended workflow is:

1. **Import** jobs, technicians, and availability from existing systems.
2. **Define rules** such as required skills, working hours, and appointment windows. Mark rules as mandatory or preferences.
3. **Run optimization** to assign jobs and order visits while reducing travel and unassigned work.
4. **Check the result** against mandatory constraints and report any jobs that could not be scheduled.
5. **Review and export** the proposed schedule. Changes to external systems require authorization.

The same approach applies to shift planning, delivery allocation, and production scheduling. Each use case supplies its own resources, decisions, constraints, and objectives; it should not require a separate hardcoded scheduling engine.

## How it works

A planning problem consists of:

| Element | Field-service example |
| --- | --- |
| Resources | Technicians and vehicles |
| Work | Service appointments |
| Hard constraints | Skills, availability, capacity, appointment windows |
| Objectives | Reduce travel, overtime, and missed appointments |
| Output | Assignments, visit order, scores, and constraint validation |

The planned execution path is **input → typed problem model → constraint validation → optimization → independent solution validation → review/export**.

Orchestrata is not a replacement for a CRM, ERP, or dispatch system. It is intended to optimize decisions using data from those systems and return results they can consume.

## Implementation

The backend uses Rust 2024, with the solver kept separate from HTTP and database code. The web application uses React, TypeScript, Vite, and TanStack libraries. Frontend and backend are independently buildable and communicate through a versioned HTTP API.

PostgreSQL is the planned operational database. MariaDB, Microsoft SQL Server, and Snowflake are planned integrations for reading and exchanging business data; they are not presented as interchangeable transactional databases.

```text
backend/    Rust domain, solver, and API work
frontend/   React application
docs/       Technical decisions
.github/    Continuous integration
```

## Current status

**Early development.** The repository currently establishes the application foundation. It does not yet demonstrate the field-service example, a complete constraint solver, database integrations, or production scheduling.

The first functional milestone is deliberately small: accept a typed scheduling problem, reject invalid constraints, generate assignments, verify the result independently, and display it in the web application. Performance and optimization quality will be measured against reproducible test cases rather than assumed.

## Development

Backend (requires a recent stable Rust toolchain):

```sh
cd backend
cargo run -p orchestrata-api
```

Frontend (once the frontend scaffold is committed; requires Node.js 22 and npm):

```sh
cd frontend
npm install
npm run dev
```

The development API is configured for `127.0.0.1:3000`, with `/api` proxied from the frontend during local development.

---

Proprietary software. No license is granted for reuse or redistribution.
