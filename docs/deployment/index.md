Run the service and UI on your workstation first. Orchestrata's project site is static; opening it does not start an API. The preview is for fictional data and local evaluation, not public access or production use.

## Local API and web UI

Use a stable Rust toolchain for the API, and Node.js 22 and npm for the web UI. From the Orchestrata repository root:

```sh
(cd backend && cargo run -p orchestrata-api)
```

In a second terminal:

```sh
(cd frontend && npm install && npm run dev)
```

The API uses `127.0.0.1:3000`. See the [local API guide](../api/) for request and outcome details. Open the UI address printed by the frontend command. The development web server forwards `/api/` requests to the local service. Check health and run the bundled assignment before evaluating a larger example:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

Keep this build on loopback. A successful health check confirms a responding process, not authentication, service capacity, or availability of a durable job store.

## Evaluate the standalone engine and SDKs

The [standalone engine repository](https://github.com/ilstarno/orchestrata-engine/tree/main) contains the Rust engine, native library, and bindings for C#, Java, Python, Node.js, PHP and Go. Build the SDK from matching source; this guide does not require a packaged release. On the engine checkout, with stable Rust:

```sh
cargo build --manifest-path backend/Cargo.toml --release --locked -p orchestrata-ffi
```

Follow the SDK README for loader and runtime requirements for your language. Use the native library from the same checkout and an ABI-compatible binding. A native library built on one platform is not proof of packaging support for other platforms.

## Before using a cluster

Experimental deployment manifests have been checked for rendering and schema consistency in separate review work. No production image, managed-cloud certification, public endpoint, or sign-in configuration is required or implied by this local guide. A cluster evaluation needs its own reviewed revision, protected ingress, tested access controls, explicit resource limits, and install/rollback evidence. Do not expose a preview API to the internet or enter real customer records to see whether it works.
