# Set up and troubleshoot the local backend

This public evaluation runs on your computer. It has no sign-in, user sessions, database-backed job queue, or production access controls. Keep it on loopback and use sample input. The project website does not run the API.

## 1. Check out the tested revision

Use Git, curl, and stable Rust with Cargo. Use Node.js 22 and npm only for the optional UI. The commands and HTTP results below were checked on Linux with Rust 1.100.0 at [public revision `b8d1271e3faddfffaf220698e381197bcae1752f`](https://github.com/ilstarno/Orchestrata/tree/b8d1271e3faddfffaf220698e381197bcae1752f). Check out that revision before comparing results.

```sh
git clone https://github.com/ilstarno/Orchestrata.git
cd Orchestrata
git checkout --detach b8d1271e3faddfffaf220698e381197bcae1752f
git rev-parse HEAD
rustc --version
cargo --version
```

`git rev-parse HEAD` should print the full revision above. Keep the checkout separate from a deployment branch.

## 2. Test and start the API

From the repository root, run the locked workspace tests. The first run may need registry access. The checked revision passed 70 tests.

```sh
cargo test --manifest-path backend/Cargo.toml --locked --workspace
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

Leave that terminal running. In a second terminal at the repository root:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS http://127.0.0.1:3000/api/v1/openapi.json
curl -fsS "http://127.0.0.1:3000/api/v1/recipes/grid?page_size=3"
```

Health returns `{"status":"ok"}`. The OpenAPI document lists the routes for the running revision. The grid returns page 1 with three records out of 13, starting with `airline-crew-assignment`. Health checks reachability only.

## 3. Compare a solve and an input error

```sh
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

Expected response:

```json
{"solution":{"values":[1,0],"score":{"hard":0,"soft":0},"evaluated":4}}
```

Job A takes slot 1 and Job B takes slot 0. Both required rules and preferences are satisfied. To exercise error handling, send a variable with no allowed values:

```sh
curl -isS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary '{"variables":[{"name":"empty","domain":[]}],"constraints":[]}'
```

Expect HTTP 422, `X-Orchestrata-Error-Code: invalid_problem`, a request ID, and the text `Invalid problem`. The request ID varies. Read [Solve API reference](../solve-api/) for contract details and distinguish a malformed request or search limit from a valid plan result.

## 4. Optional local web interface

With the API running on port 3000, open a third terminal. The [workspace screenshots](../#product-ui-evidence) show this development build with example records and the local API.

```sh
git clone https://github.com/ilstarno/orchestrata-frontend.git
cd orchestrata-frontend
git checkout --detach cb873a819ed8c79c22bce35198d42fbf982a5e12
cd frontend
npm ci
npm run dev
```

Open the loopback address printed by Vite. Select **Run example**. The result assigns Alex to lift repair, Sam to the safety inspection, and Noor to preventive maintenance. Select **Browse recipes** to see the 13 records returned by the local API.

Vite forwards `/api` to `http://127.0.0.1:3000`. If you use another loopback port, update the curl URLs and Vite proxy target. `ORCHESTRATA_HTTP_BIND` requires an IP address and port; a hostname, port alone, or empty setting is invalid. Stop the dev server and API with Ctrl-C when finished. This local setup does not connect to any vendor account.

## Integration proof checklist

Use this checklist to record what your integration actually verifies. The local walkthrough proves one fixture and an input error at the pinned revision. It does not establish any vendor connector, authentication flow, write-back, or production deployment.

1. **Identify both sides.** Record the Orchestrata revision, the client or vendor product and environment, enabled API or export permissions, and the interface you actually exercised. Confirm any vendor entitlement with that tenant’s administrator.
2. **Map a small approved input.** Save the sample source, exact normalized request, and field map. For each consumed field, record its identifier, unit or time zone, allowed values, and missing-value behavior. Note which source fields are ignored.
3. **Verify the decision.** Compare with an expected result. Check assignments, hard rules, preferences, and the reason for an unassigned item. Save the returned score and any search limit.
4. **Verify failure behavior.** Record a rejected input and a case the client cannot process, such as missing permissions, stale input, timeout, or an unsupported field. Preserve status and request ID where available; omit credentials and customer records.
5. **Confirm who can act.** Start read-only. Agree what a person reviews, where the evidence stays, and which writes would require approval. If testing an authorized write in a sandbox, verify repeated submissions and recovery before making any rollout claim.

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

[Review #25](https://github.com/ilstarno/Orchestrata/pull/25) adds the `orchestrata-session-store` crate with SeaORM 2.0.4. It requires Rust 1.94 or later. These steps apply to that revision only. The adapter accepts a PostgreSQL SeaORM connection. It does not change the baseline API setup, read a database URL from the API environment, or enable accounts, login, cookies, or any authenticated route. No database is needed for the basic API walkthrough above.

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
