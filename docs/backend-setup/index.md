# Set up and troubleshoot the local backend

Run the public backend with the bundled example. Keep it on loopback and do not send customer data: the starter has no sign-in, database-backed job queue, or production access controls. The project website does not run the API.

## Prepare the checkout

Use a stable Rust toolchain with Cargo, plus Git and curl. If you also want the web interface, use Node.js 22 and npm. The optional PostgreSQL adapter below has a separate requirement of Rust 1.94 or later.

```sh
git clone https://github.com/ilstarno/Orchestrata.git
cd Orchestrata
git rev-parse --short HEAD
rustc --version
cargo --version
```

Run these commands from the repository root to check the starter and start the API. The first build may need access to the Cargo registry:

```sh
cargo test --manifest-path backend/Cargo.toml --locked --workspace
cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

Leave the API terminal running. In another terminal, go to the repository root and send the bundled example:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS http://127.0.0.1:3000/api/v1/openapi.json
curl -fsS "http://127.0.0.1:3000/api/v1/recipes/grid?page_size=3"
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

The health response confirms that the HTTP process answers. It does not check sign-in, database storage, or capacity. The example uses two jobs and two allowed assignment values. In the expected result, the solution values are `[1,0]` in input order. Use the OpenAPI output from your running revision for its payload and response fields.

If you need the UI, leave the API running and open another terminal at the repository root:

```sh
(cd frontend && npm install && npm run dev)
```

Open the local address printed by the command. The Vite server forwards `/api` requests to `http://127.0.0.1:3000`. The API listens on `127.0.0.1:3000` unless you set `ORCHESTRATA_HTTP_BIND` to an explicit IP address and port. For example, `127.0.0.1:8080` keeps access on the same computer while changing the port. Update the curl URLs and the local Vite proxy target if you change it. A hostname, port alone, or empty setting is invalid.

## Troubleshoot the starter

| Symptom | What to check |
| --- | --- |
| Cargo cannot find the manifest or the package | Run from the repository root. Use `--manifest-path backend/Cargo.toml`; if already in `backend`, omit that option. If Cargo rejects edition 2024 or a dependency's minimum Rust version, check the active toolchain with `rustc --version` and use a current stable toolchain. |
| Cargo cannot download dependencies | Check registry access or your approved proxy. Offline mode works only when all resolved dependencies are already cached. Keep dependency version changes separate from network troubleshooting. |
| Startup fails with “Address already in use” | Another process owns port 3000. Check whether it is your previous API instance; stop your own earlier process before restarting. Do not terminate an unidentified process or switch the API to a public interface. |
| curl reports connection refused | Check the API terminal for a startup error. Retry on the API port after the process starts. |
| curl cannot read `assignment.json` | Run from the repository root, or change the file path after the `@` to match your working directory. |
| A solve request fails | Use `curl -i` to see the HTTP status and response. Omit `curl -f` to keep the error body. Check the method, `Content-Type`, JSON structure, and allowed problem size. Read the `X-Orchestrata-Error-Code` header and look it up at `/api/v1/error-codes`. An invalid problem or a solve limit does not mean no plan exists. |
| The UI loads but cannot solve | Call the API health URL directly, then check the Vite terminal for proxy errors. Confirm that the UI and API use the intended revisions and that the proxy points at the API's loopback address. |
| A recipe route returns 404 | Check the running OpenAPI document. The current baseline serves the recipe catalog and grid. If the OpenAPI document does not list them, check the running revision. For a missing individual recipe, list the catalog and use an ID from its response. |

## Optional: evaluate PostgreSQL session persistence

[Review #25](https://github.com/ilstarno/Orchestrata/pull/25) adds the `orchestrata-session-store` crate with SeaORM 2.0.4. These steps apply to that revision only. The adapter accepts a PostgreSQL SeaORM connection. It does not change the baseline API setup, read a database URL from the API environment, or enable accounts, login, cookies, or any authenticated route. No database is needed for the basic API walkthrough above.

From the repository root of that revision, run the default adapter tests without Docker or database credentials:

```sh
cargo test --manifest-path backend/Cargo.toml -p orchestrata-session-store
```

The default tests use a SeaORM mock. To verify compilation of the optional database test without starting Docker, run:

```sh
cargo test --manifest-path backend/Cargo.toml -p orchestrata-session-store \
  --features postgres-container-tests --test postgres_container --no-run
```

A successful compile-only check is not a database test. For the actual integration test, use a local Docker daemon that your account can access and that can obtain the official `postgres:17-alpine` image. Run:

```sh
docker info
cargo test --manifest-path backend/Cargo.toml -p orchestrata-session-store \
  --features postgres-container-tests --test postgres_container
```

The real test starts a disposable PostgreSQL container with a per-run credential and a dynamically allocated IPv4 loopback port. It applies the schema twice, creates sessions, checks duplicate detection and the exact expiry boundary, revokes a session and checks repeated revocation. The opt-in test requires a reachable Docker daemon and fails if one is unavailable. If the image cannot be pulled, check your environment's registry access. Do not point the test at a shared or production database.

### Use the adapter in a local integration test

1. Use an isolated PostgreSQL database you control. Apply `backend/crates/session-store/schema/001_app_sessions.sql` with a migration connection that can create the table and expiry index. Alternatively call `initialize_schema()` on an adapter built with that migration connection. Do not apply schema changes on each request.
2. Open a PostgreSQL connection with SeaORM and pass it to `SessionStore::postgres(connection)`. Keep database credentials out of code, output, and committed fixtures. This adapter rejects other drivers, regardless of what SeaORM itself supports.
3. Call `create` with a `SessionDigest` encapsulating 32 digest bytes, an internal UUID for the principal, and trusted server times in Unix milliseconds. Use fabricated values in an integration test. Never pass a raw session token as the digest. Expiry must be strictly later than the creation reference time.
4. Call `resolve` with the same digest and the current trusted time. An unknown or expired session returns no principal; expiry is exclusive, so a session is already expired at its exact expiry time. Call `revoke` to delete one digest; it returns false if no record remains to delete.

When a real check fails on schema or permissions, verify that the migration and test connections use the same target database and schema and that the test connection can read, insert and delete rows. An unsupported backend is a configuration error. A duplicate digest is not an authentication success. The adapter does not automatically purge expired rows, mint tokens, look up group memberships or establish a user session in the browser. The PostgreSQL test covers only the adapter. It does not test sign-in.

Source references: [verified public source](https://github.com/ilstarno/Orchestrata/tree/b8d1271e3faddfffaf220698e381197bcae1752f), [SeaORM database and runtime configuration](https://www.sea-ql.org/SeaORM/docs/install-and-config/database-and-async-runtime/), and [PostgreSQL Testcontainers module](https://rust.testcontainers.org/modules/postgres/).
