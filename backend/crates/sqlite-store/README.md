# Owned SQLite job storage

This adapter stores jobs and outbox events for local user IDs already resolved by the application. It does not authenticate requests, issue sessions, deliver events, or run a worker. `claim_next` is an installation-wide operation for a trusted worker; it must not be exposed directly to an end user.

## Schema maintenance

The SQLite definitions are in `backend/schema/sqlite`, one table per file. The separate PostgreSQL definition in `backend/schema/solver_jobs.sql` is not part of this adapter. Versioned SQLite migrations and their checksum are in `backend/migrations/sqlite`.

From the repository root, generate and validate changes with Atlas:

```sh
atlas migrate diff change_name --dir file://backend/migrations/sqlite \
  --to file://backend/schema/sqlite --dev-url 'sqlite://file?mode=memory'
atlas migrate validate --dir file://backend/migrations/sqlite \
  --dev-url 'sqlite://file?mode=memory'
```

Review new SQL before applying it. Never edit an applied migration. The initial migration creates an empty owned-job store and marks SQLite `application_id` and `user_version`. `SqliteStore::open` uses that same bootstrap only for an empty, unmarked database. The crate-local `migrations/003_*.sql` files are retained unchanged for compatibility verification.

## Existing databases

Existing owned stores with schema version 3 and the correct application marker remain usable without rewriting their schema or data. Tenant-based prototype versions 1 and 2 are rejected as `UnsupportedSchema`, preserving their files. No tenant-to-user ownership mapping is inferred. Moving those records requires a separately verified ownership migration; this adapter does not provide one.

Foreign, populated or incorrectly marked databases are rejected. Use a separate database path for a fresh store; do not remove version markers to make an existing database appear empty. Session validation, authorization, immutable input/result references and migration operations remain responsibilities of the calling application.
