# Run the server from source

Start a local Orchestrata server, check that it responds and solve two jobs. You need this guide only when you are running the server yourself. To call a server your team already operates, go straight to the [Solve API](../solve-api/). To inspect a result without installation, open the [worked example](/Orchestrata/examples/assignment/).

## 1. Install the tools

**Rust** is the programming language used to build the server. **Cargo** is Rust's build and dependency tool: it downloads the libraries the project needs, compiles the source and starts the program. You do not need to learn Rust to run these commands.

Install the current stable Rust toolchain using the [official Rust installation guide](https://rust-lang.org/tools/install/). Its rustup installer includes Cargo. Follow the platform's compiler/linker prerequisites; on Windows, the official guide explains the C++ build tools. The shell examples here use Linux, macOS or WSL.

You also need **Git** to download the source and **curl** to send the HTTP requests. Open a new terminal after installation and check:

```sh
git --version
rustc --version
cargo --version
curl --version
```

If a command is not found, finish its installation and reopen the terminal. Do not continue until these checks print their versions. See the [Cargo installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html) for PATH troubleshooting. No Node.js, browser frontend or database is required for this example.

## 2. Download and start the server

Choose a directory for your source checkouts, then run:

```sh
git clone https://github.com/ilstarno/Orchestrata.git
cd Orchestrata
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 \
  cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

Leave this terminal running. The first build downloads dependencies and takes longer. `backend/Cargo.toml` is the project manifest; `--locked` uses the dependency versions recorded by the project; `-p orchestrata-api` selects the server program. All repository commands in this guide run from the **Orchestrata checkout root**, not inside `backend`.

The address `127.0.0.1:3000` keeps the server on this computer, on port 3000. It is also the default. This sample server has no authentication or production access controls; keep it on loopback and use sample input. The public website is a separate static site, not this server.

## 3. Check the server

Open a second terminal. These requests work from any directory:

```sh
API=http://127.0.0.1:3000
curl -fsS "$API/api/v1/health"
curl -fsS "$API/api/v1/openapi.json"
```

Health returns `{"status":"ok"}`. **OpenAPI** is the machine-readable description of the running server's routes and data fields. Keep the API terminal open while sending requests.

## 4. Solve the included problem

In the second terminal, change to the Orchestrata checkout root. Send the included JSON file:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

Expected response:

```json
{"solution":{"values":[1,0],"score":{"hard":0,"soft":0},"evaluated":4}}
```

Job A takes resource 1 and job B takes resource 0. Both required rules and the preference are satisfied. Continue with the [Solve API walkthrough](../solve-api/) to send inline JSON and handle errors, or the [Recipe API](../recipes-api/) to discover bundled examples. Stop the server with **Ctrl-C** in its terminal when finished.

## 5. Use a browser example

The [browser assignment tool](/Orchestrata/try/) calculates assignments on your device. It is a separate working example and does not connect to this local API. Use the HTTP walkthrough above to verify your server.

## Integration proof checklist

Use this checklist to record what your integration actually verifies. The local walkthrough proves one fixture and an input error in the running build. It does not establish any vendor connector, authentication flow, write-back, or production deployment.

1. **Identify both sides.** Record the Orchestrata build and enabled capabilities, the client or vendor product and environment, enabled API or export permissions, and the interface you actually exercised. Confirm any vendor entitlement with that tenant’s administrator.
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
| A recipe route returns 404 | Check the running OpenAPI document. Recipe catalog and grid routes are available only if the running OpenAPI document lists them. For a missing individual recipe, list the catalog and use an ID from its response. |

## Optional: evaluate PostgreSQL session persistence

If Cargo metadata lists `orchestrata-session-store`, you can evaluate the PostgreSQL adapter separately. Check the package manifest for its Rust and SeaORM requirements. The adapter accepts a PostgreSQL SeaORM connection. It does not change the baseline API setup, read a database URL from the API environment, or enable accounts, login, cookies, or any authenticated route. No database is needed for the basic API walkthrough above.

From the repository root, run the default adapter tests without Docker or database credentials:

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
2. Open a PostgreSQL connection with SeaORM and pass it to `SessionStore::postgres(connection)`. Keep database credentials out of code, output, and checked-in fixtures. This adapter rejects other drivers, regardless of what SeaORM itself supports.
3. Call `create` with a `SessionDigest` encapsulating 32 digest bytes, an internal UUID for the principal, and trusted server times in Unix milliseconds. Use fabricated values in an integration test. Never pass a raw session token as the digest. Expiry must be strictly later than the creation reference time.
4. Call `resolve` with the same digest and the current trusted time. An unknown or expired session returns no principal; expiry is exclusive, so a session is already expired at its exact expiry time. Call `revoke` to delete one digest; it returns false if no record remains to delete.

When a real check fails on schema or permissions, verify that the migration and test connections use the same target database and schema and that the test connection can read, insert and delete rows. An unsupported backend is a configuration error. A duplicate digest is not an authentication success. The adapter does not automatically purge expired rows, mint tokens, look up group memberships or establish a user session in the browser. The PostgreSQL test covers only the adapter. It does not test sign-in.

Source references: [Orchestrata repository](https://github.com/ilstarno/Orchestrata), [SeaORM database and runtime configuration](https://www.sea-ql.org/SeaORM/docs/install-and-config/database-and-async-runtime/), and [PostgreSQL Testcontainers module](https://rust.testcontainers.org/modules/postgres/).
