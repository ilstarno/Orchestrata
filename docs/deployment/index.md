## Choose how to start

To inspect a result without installing tools, open the [worked assignment](/Orchestrata/examples/assignment/). To call a running server, follow the [first HTTP request](/Orchestrata/docs/solve-api/); the client needs no Rust installation.

Rust is the programming language used to build the engine and server. Cargo builds Rust programs and manages their dependencies. Only the source-build path needs these tools. The [server setup guide](/Orchestrata/docs/backend-setup/) explains installation and version checks.

All source commands below run from the Orchestrata checkout root. The shell examples use Linux, macOS or WSL.

## Run a local server

Use the [step-by-step server setup](/Orchestrata/docs/backend-setup/) to install the tools and start the API. The website is separate from the server. No browser frontend or database is needed for the first request.

```sh
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 \
  cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

## Use a native library

Choose a binding and follow the [SDK walkthrough](/Orchestrata/docs/sdk/). It uses the public standalone engine source, a matching native library and a local package installation.

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
