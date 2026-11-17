Run the API and web UI on your workstation. Use sample data and keep this build off the public internet. The website does not start the API.

## Local API and web UI

Use stable Rust with Cargo and curl. For the web UI, also install Node.js 22 and npm. Start with the [Orchestrata repository](https://github.com/ilstarno/Orchestrata). Obtain the source and check that the Cargo workspace lists `orchestrata-api`. Check the running OpenAPI document before using optional routes.

From the repository root, run the locked workspace tests, then start the API on loopback:

```sh
cargo test --manifest-path backend/Cargo.toml --locked --workspace
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

Leave the API running. From the repository root in a second terminal, check the health endpoint, request the first three rows of the recipe grid, and run the bundled assignment:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS 'http://127.0.0.1:3000/api/v1/recipes/grid?page_size=3'
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

The health request confirms the process responds. The recipe grid returns recipe metadata; it does not submit jobs. The expected assignment is `[1,0]` in input order. None of these checks proves sign-in, durable job storage, or service capacity.

To use the web UI, keep the API running and open a terminal in the separate [frontend repository](https://github.com/ilstarno/orchestrata-frontend) if you have access. Confirm it contains `frontend/package.json` and its API schema matches the running backend.

```sh
(cd frontend && npm install && npm run dev)
```

Open the local URL printed by the UI command. The development server forwards `/api` to `127.0.0.1:3000`. If you change `ORCHESTRATA_HTTP_BIND` to another loopback port, update both the curl URLs and the development proxy target.

See the [local API guide](../api/) for request and response details and the [backend setup and troubleshooting guide (English)](../backend-setup/) if startup or the proxy fails.

## Evaluate the standalone engine and SDKs

The [standalone engine repository](https://github.com/ilstarno/orchestrata-engine) contains the Rust engine, native library, and bindings for C#, Java, Python, Node.js, PHP and Go. Build the SDK from matching source; this guide does not require a packaged release. On the engine checkout, with stable Rust:

```sh
(
  if [ -f Cargo.toml ] && [ -f crates/ffi/include/orchestrata.h ]; then
      engine_manifest=Cargo.toml
  elif [ -f backend/Cargo.toml ] && [ -f backend/crates/ffi/include/orchestrata.h ]; then
      engine_manifest=backend/Cargo.toml
  else
      printf '%s\n' 'Run these commands from the standalone engine repository.' >&2
      exit 1
  fi
  cargo build --manifest-path "$engine_manifest" --release --locked -p orchestrata-ffi
)
```

Run this from the standalone engine repository, not the API repository above. The command finds the right build file for your checkout.

Follow the SDK README for loader and runtime requirements for your language. Use the native library from the same checkout and an ABI-compatible binding. A native library built on one platform is not proof of packaging support for other platforms.

## Checklist before testing on a cluster

Treat cluster use as a separate evaluation. The local checks above do not establish that an image, a managed-cloud configuration, or public access is ready. Use only sample data and leave public ingress disabled.

1. Record the digest of the image you tested. Use a dedicated, non-production namespace.
2. Set CPU and memory requests and limits. Run health, recipe grid, and assignment checks from an authorized client inside the test network.
3. Verify that access outside the intended boundary is blocked. Inspect logs without recording credentials or customer data.
4. Install the candidate and roll back to the previous image; repeat the checks. If you add persistent storage, test backup and restore separately before using it.

## Container, Kubernetes, and Terraform

Build the API image from the checked-in Dockerfile, then use Helm to install it in an existing private Kubernetes cluster. The Terraform instructions cover existing Amazon EKS, Azure AKS, and Google GKE clusters. They do not create cloud infrastructure.

For the Dockerfile, Helm chart, and Terraform notes, see `deploy/` and `deploy/terraform/` in the source if present.

The chart passed local Helm checks. The container and managed-cluster rollout have not been tested.
