# Run the API in a container or on Kubernetes

This is a private evaluation deployment of the Orchestrata HTTP API. Use sample data. The image and chart do not deploy the web interface, MCP, a database, authentication, or TLS. The API is unauthenticated; do not expose it through a public or shared ingress.

## Build and test your image locally

Use Docker with BuildKit and curl. Run from the repository root; the build context must include `backend/` and the bundled `recipes/`.

```sh
docker build --file deploy/container/api.Dockerfile --tag orchestrata-api:0.1.0 .
bash deploy/container/smoke.sh orchestrata-api:0.1.0
```

The multi-stage build compiles the checked-in lockfile in a Rust image and copies only the binary into a matching Debian-family runtime. For repeatable deployments, override `RUST_IMAGE` and `RUNTIME_IMAGE` with reviewed immutable image digests from the same distribution family, then publish the result to an approved private registry. Select the image platform for your Kubernetes nodes. The API process runs as numeric UID/GID 10001.

To leave the API running locally, keep the host port explicitly on loopback:

```sh
docker run --rm --read-only --cap-drop=ALL --security-opt=no-new-privileges \
  --publish 127.0.0.1:3000:3000 orchestrata-api:0.1.0
```

In another terminal:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS http://127.0.0.1:3000/api/v1/openapi.json
```

A native run defaults to `127.0.0.1:3000`. The image and chart set `ORCHESTRATA_HTTP_BIND=0.0.0.0:3000` **inside** the container so the published port or Pod IP can reach it. This is not permission to publish the host port on all interfaces. The health endpoint returns `{"status":"ok"}`; it checks the HTTP process only, not access control, capacity, or durable storage.

## Existing Kubernetes cluster

Use Helm 3 and kubectl, an existing cluster context and namespace you are authorized to use, and an image reachable by its nodes. Put your image location in a values file; if pulling from a private registry, refer to a registry pull Secret already provisioned in that namespace.

```yaml
image:
  repository: registry.example.com/team/orchestrata-api
  tag: "0.1.0"
# imagePullSecrets:
#   - name: team-registry
```

When deploying a reviewed artifact, provide its actual `image.digest` (`sha256:` plus 64 lowercase hexadecimal characters). A digest takes precedence over the tag. The chart does not create credentials. It installs a deployment and a ClusterIP service, with no external load balancer, ingress, or mounted API token. A ClusterIP does not block other workloads in the cluster: use a dedicated evaluation cluster or verify isolation with your administrator first.

Render and lint before contacting a cluster:

```sh
helm lint --strict deploy/helm/orchestrata-api -f ./orchestrata-values.yaml
helm template orchestrata deploy/helm/orchestrata-api \
  --namespace orchestrata-eval -f ./orchestrata-values.yaml
```

Confirm the target context and namespace, then install in your approved evaluation cluster:

```sh
KUBE_CONTEXT=your-reviewed-context
kubectl config get-contexts "$KUBE_CONTEXT"
kubectl --context "$KUBE_CONTEXT" get namespace orchestrata-eval
helm --kube-context "$KUBE_CONTEXT" upgrade --install orchestrata deploy/helm/orchestrata-api \
  --namespace orchestrata-eval -f ./orchestrata-values.yaml \
  --atomic --wait --timeout 3m
kubectl --context "$KUBE_CONTEXT" -n orchestrata-eval rollout status deployment/orchestrata-api --timeout=2m
kubectl --context "$KUBE_CONTEXT" -n orchestrata-eval port-forward --address 127.0.0.1 svc/orchestrata-api 3000:3000
```

In another terminal, run the same health and OpenAPI checks above. Startup, readiness, and liveness all call the health endpoint at the Pod's port 3000. Resource defaults are requests of 100m CPU and 64 MiB, limits of 1 CPU and 256 MiB, and one replica. These are evaluation defaults, not capacity estimates. For capacity checks, use sample data, measure actual peaks, and set resource limits for your workload.

| Symptom | What to check |
| --- | --- |
| `ImagePullBackOff` | Confirm registry reachability, image tag/digest, node architecture, and the namespace-local pull Secret. Never put a registry password in the values file. |
| `Pending` | Inspect namespace quotas, node resources and Pod scheduling events. |
| Readiness or startup timeout | Inspect the Pod events and `kubectl --context "$KUBE_CONTEXT" -n orchestrata-eval logs deployment/orchestrata-api`. Verify the binary and its `0.0.0.0:3000` container bind. |
| Rising restarts or slow solves | Check termination reason, memory, and CPU throttling before changing probe timing or resource limits. |
| Local port is in use | Change only the first port in port-forward, for example `13000:3000`, and update the curl URL. |

To remove the evaluation release, run `helm --kube-context "$KUBE_CONTEXT" uninstall orchestrata --namespace orchestrata-eval`. The chart does not remove the namespace or the image. Before any shared or public use, a separate review must verify authentication, ingress, authorization and workload isolation for your environment.

## Manage the install with Terraform

For an existing Kubernetes cluster, use the [Terraform release guide](terraform/README.md). It includes explicit context and image-digest examples for Amazon EKS, Azure AKS, Google GKE, and other Kubernetes installations. Terraform installs the API chart; it does not create cloud resources, a namespace, a registry, or an identity.
