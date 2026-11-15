# 0003 — Operational storage and external connectors

Status: accepted

## Decision

Use PostgreSQL as the planned transactional control plane. Treat MariaDB, Microsoft SQL Server, and Snowflake as source/sink integrations through isolated adapters.

## Why

The control plane needs transactional job state and recovery semantics. External data platforms differ in SQL dialect, transaction guarantees, and intended workloads.

## Consequences

Connector support does not imply interchangeable operational databases. Durable storage and connectors remain unimplemented.
