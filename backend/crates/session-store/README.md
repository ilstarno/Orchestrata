# PostgreSQL session storage

This crate stores a 32-byte session digest, an internal principal ID, and the expiry. It does not issue cookies, handle sign-in, or configure an identity provider. The caller supplies a digest of the opaque session token; do not pass the raw cookie value.

## Local checks

From the repository root, use the stable Rust toolchain and run:

```sh
cd backend
cargo test -p orchestrata-session-store
```

The default tests use SeaORM's mock connection. They do not require Docker or database credentials.

For a real PostgreSQL check, start a local Docker daemon and allow it to pull the official `postgres:17-alpine` image. Then run from `backend`:

```sh
cargo test -p orchestrata-session-store --features postgres-container-tests --test postgres_container
```

This test uses a disposable database, a per-run credential, and a loopback port. It does not connect to a shared database or identity provider. A missing Docker daemon causes the opt-in test to fail rather than skip.

The table and its index are declared in `backend/schema/postgres/app_sessions.sql`. The separately generated and checksummed Atlas migration stream is in `backend/migrations/postgres`; apply it with a migration connection before starting the application. It creates the table without replacing an existing unmanaged table. Keep applied migrations immutable.

`initialize_schema()` remains an idempotent compatibility bootstrap for disposable test or migration connections, not a request-time operation or a substitute for Atlas schema validation. The original crate-local SQL is retained unchanged for compatibility. The PostgreSQL test applies the versioned migration independently, checks reopening, expiry and revocation, and verifies that an incompatible existing table is not overwritten.

The opt-in fixture supports Unix hosts with a local Docker socket. It rejects remote endpoints before creating a container and pins the validated socket for every operation. It binds only a randomly assigned loopback port, bounds Docker commands and readiness waits, verifies cleanup on success, and attempts bounded cleanup on failure. Its temporary credential file is removed; connection strings and database details are not printed. The adapter supports PostgreSQL only; verified production authentication and a browser session flow are separate from this crate.
