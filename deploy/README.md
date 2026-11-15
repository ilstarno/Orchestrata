# Deploy Orchestrata

The recipe runs two images: an API and a web server. The web image serves the UI and proxies the same-origin `/api/` path to the internal API service. No browser-side cluster address or cross-origin configuration is needed.

**Current boundary:** this is an internal preview. The API does not authenticate users, and these recipes do not provide TLS termination, a durable job store, or public access. Keep the services internal. Do not attach an internet-facing ingress or send sensitive data to a preview.

## Build images

From the repository root:

```sh
docker build -f deploy/containers/api.Dockerfile -t orchestrata-api:dev .
docker build -f deploy/containers/web.Dockerfile -t orchestrata-web:dev .
```

For a release, build reviewed revisions in your own pipeline, pin build bases and published runtime images by digest, and scan them. The development recipes resolve dependencies at build time when the source tree has no lockfile. For an isolated local cluster such as kind, load both images into that cluster. On a remote cluster, publish the images to a registry the nodes can access and replace both image references. The repository does not imply that those images have been published.

## Option A: Helm

Use an explicit Kubernetes context and an existing namespace. The image variables below default to the local development tags, which you must load into your local cluster. To deploy to a remote cluster, set both variables to the image references in your registry.

```sh
helm lint deploy/helm/orchestrata
helm template orchestrata deploy/helm/orchestrata --namespace "$NAMESPACE"
helm upgrade --install orchestrata deploy/helm/orchestrata \
  --kube-context "$CONTEXT" --namespace "$NAMESPACE" \
  --set-string api.image="${API_IMAGE:-orchestrata-api:dev}" \
  --set-string web.image="${WEB_IMAGE:-orchestrata-web:dev}" \
  --atomic --wait --timeout 5m
kubectl --context "$CONTEXT" -n "$NAMESPACE" port-forward svc/orchestrata-web 8080:8080
```

Open `http://127.0.0.1:8080` or verify the proxy with `curl -fsS http://127.0.0.1:8080/api/v1/health`.

Resource settings and replica counts are adjustable in `values.yaml`. Pass only names of pre-created secrets through `imagePullSecrets`. Both services are ClusterIP. Containers run without root or service account token; startup, readiness, and liveness probes test HTTP. Web readiness refers to the web process, not the state of the solver or a database.

## Option B: Plain manifests

The [Kustomize base](kubernetes/) is namespace-neutral and generated from the default local Helm chart. Prefer an overlay with image substitutions for remote clusters. Use either raw manifests or Helm, not both for the same release name and namespace.

```sh
kubectl kustomize deploy/kubernetes
kubectl --context "$CONTEXT" -n "$NAMESPACE" apply -k deploy/kubernetes
```

Regenerate after changing the chart:

```sh
helm template orchestrata deploy/helm/orchestrata > deploy/kubernetes/orchestrata.yaml
```

## Option C: Terraform

The [existing-cluster example](terraform/existing-cluster/) owns the Helm release only. It does not create a cluster, namespace, registry, permissions, DNS, or a database. Prepare the target namespace and registry access separately. Keep state in a backend with access controls, use least-privilege cluster credentials, and keep kubeconfig and credentials out of version control.

```sh
cd deploy/terraform/existing-cluster
cp example.tfvars local.tfvars
# Update local.tfvars with the explicit context, namespace and image references.
terraform init
terraform validate
terraform plan -var-file=local.tfvars
```

Once the plan has been reviewed, the operator can apply it. This example is provider-neutral; use the same Terraform directory for a prepared managed or self-hosted cluster.

| Existing cluster | Set up kubeconfig access | Common registry |
| --- | --- | --- |
| AWS EKS | [`aws eks update-kubeconfig`](https://docs.aws.amazon.com/eks/latest/userguide/create-kubeconfig.html) | ECR |
| Google GKE | [`gcloud container clusters get-credentials`](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/cluster-access-for-kubectl) | Artifact Registry |
| Azure AKS | [`az aks get-credentials`](https://learn.microsoft.com/en-us/azure/aks/control-kubeconfig-access) | ACR |
| Other Kubernetes | Use the operator's kubeconfig and named context. | An OCI registry accessible to the cluster |

These recipes install into existing clusters. You need cluster access, a namespace, and image pull access. Check your cluster policies and test install and rollback in a non-production namespace. The preview does not require a database.

### Point the recipe at a managed cluster

Use a separate local kubeconfig, especially when working with more than one cluster. Run only the command for the existing cluster you intend to use:

```sh
# AWS EKS
aws eks update-kubeconfig --name "$CLUSTER" --region "$REGION" \
  --kubeconfig "$KUBECONFIG_PATH" --alias "$CONTEXT"

# Google GKE; install gke-gcloud-auth-plugin first.
KUBECONFIG="$KUBECONFIG_PATH" gcloud container clusters get-credentials "$CLUSTER" \
  --project "$PROJECT" --location "$LOCATION"

# Azure AKS; use your normal user context, not --admin.
az aks get-credentials --name "$CLUSTER" --resource-group "$RESOURCE_GROUP" \
  --file "$KUBECONFIG_PATH"
```

Read the resulting context name with `kubectl --kubeconfig "$KUBECONFIG_PATH" config get-contexts`, then set `kubeconfig_path` and `kubeconfig_context` in the Terraform variables. For the Helm and kubectl examples, export `KUBECONFIG` before running them. The commands configure local access only; they do not create clusters or alter registry permissions.
