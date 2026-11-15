# Data and connectors

## Planned architecture

PostgreSQL is the intended operational store for tenants, jobs, and durable execution state. MariaDB, Microsoft SQL Server, and Snowflake are intended external source and destination connectors. They are not interchangeable transactional control-plane databases.

Polars is intended for ingestion and analytical transformations, not as a substitute for solver search. Large immutable artifacts may use S3-compatible storage.

## Integration boundaries

Read-only access is the default. External writes require authorization, idempotency design, and reconciliation for uncertain outcomes. Credentials remain server-side.

## Current status

No production database adapter, durable job queue, or connector implementation is present in the initial foundation.

## Gotchas

- Do not claim exactly-once external effects without destination-side cooperation.
- Keep dialect-specific queries and type conversions inside adapters.
