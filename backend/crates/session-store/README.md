# PostgreSQL session storage

This crate stores a 32-byte session digest, an internal principal ID, and the expiry. It does not issue cookies, handle sign-in, or configure an identity provider. The caller supplies a digest of the opaque session token; do not pass the raw cookie value.

## Local checks

From the repository root, use Rust 1.94 or newer and run:

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

The table and its index are in `schema/001_app_sessions.sql`. Use a migration connection to apply the schema. The adapter supports PostgreSQL only; verified production authentication and a browser session flow are separate from this crate.
