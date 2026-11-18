# Deploy Orchestrata API to an existing Kubernetes cluster

This Terraform root installs the local Orchestrata API Helm chart into an existing namespace. The same root is used for Amazon EKS, Azure AKS, Google GKE, and other Kubernetes clusters. HashiCorp's Helm provider uses the explicit Kubernetes context and its own auth client; no cloud provider account, network, cluster, namespace, identity, database, or registry is provisioned. A separate Kubernetes provider is intentionally unnecessary because this root manages only the Helm release. This deploys the API only, not a complete hosted enterprise service.

## Before planning

- Use a reachable cluster endpoint with working certificate validation. For private endpoints, run from an authorized network. Use a scoped deployment identity with permissions for the rendered chart and the Helm release metadata in the existing namespace, not a general-purpose cluster administrator.
- Arrange a namespace and image access with the platform team. Supply an already-published registry image by immutable digest. For a private image use cluster-managed image-pull identity, or pass only the name of an existing image pull Secret via a chart values file. Do not put database passwords, tokens, kubeconfig content, or secret values into Terraform variables, Helm values, command lines, or state.
- Use one Terraform state per cluster/environment/release; set up access-controlled, encrypted, lock-enabled remote state before shared use. No remote backend is selected in this example; otherwise Terraform uses local state. Treat both plans and state as sensitive, even when passing only non-secret Helm settings. Terraform manages the release after it is first installed; do not edit it with another release manager. An existing release is not adopted automatically; agree on ownership and import it explicitly if transferring lifecycle management.
- Use only a trusted kubeconfig, with an explicit path and context. Its exec credential process runs locally as the Terraform caller. Do not set competing provider auth environment overrides (such as `KUBE_HOST`, `KUBE_TOKEN`, or `KUBE_CONFIG_PATHS`). Use a dedicated file per environment and do not commit it.
- These examples show operator access to **existing** clusters. In CI use your platform's existing workload federation or short-lived credential chain, not a stored admin kubeconfig, and ensure the runner has the relevant executable. The examples were not tested against live cloud clusters.

## Select the existing target

Choose one setup below. Replace the uppercase placeholders and create `$HOME/.kube` if it does not already exist. Keep the kubeconfig file owner-only; the commands never need to print or copy credential material into Terraform.

### Amazon EKS

Use an AWS CLI identity authorized for the intended EKS cluster. AWS writes an exec entry that invokes `aws eks get-token` whenever Kubernetes credentials are needed.

```sh
export TARGET_KUBECONFIG="$HOME/.kube/orchestrata-eks"
aws eks update-kubeconfig --name CLUSTER --region REGION \
  --alias orchestrata-eks --kubeconfig "$TARGET_KUBECONFIG"
```

Use `examples/eks.tfvars.example` with `kubeconfig_context = "orchestrata-eks"`. See [EKS kubeconfig guidance](https://docs.aws.amazon.com/eks/latest/userguide/create-kubeconfig.html) and [the exec token command](https://docs.aws.amazon.com/cli/latest/reference/eks/get-token.html).

### Azure AKS

For a Microsoft Entra-integrated cluster, use a signed-in Azure CLI identity scoped to the intended subscription and install `kubelogin`. Request the user configuration, not `--admin`. The local operator example uses the Azure CLI exec login mode.

```sh
export TARGET_KUBECONFIG="$HOME/.kube/orchestrata-aks"
az aks get-credentials --subscription SUBSCRIPTION --resource-group GROUP \
  --name CLUSTER --file "$TARGET_KUBECONFIG" --format exec
kubelogin convert-kubeconfig --kubeconfig "$TARGET_KUBECONFIG" -l azurecli
```

Use `examples/aks.tfvars.example` with the actual context printed below. See [Microsoft's AKS kubelogin guidance](https://learn.microsoft.com/en-us/azure/aks/kubelogin-authentication).

### Google GKE

Install the Google Cloud CLI and its `gke-gcloud-auth-plugin` using the platform's supported install method. Configure the intended Google identity before fetching credentials. GKE's generated kubeconfig carries the plugin command.

```sh
export TARGET_KUBECONFIG="$HOME/.kube/orchestrata-gke"
KUBECONFIG="$TARGET_KUBECONFIG" gcloud container clusters get-credentials \
  CLUSTER --location LOCATION --project PROJECT
```

Use `examples/gke.tfvars.example` with the actual context printed below. See [GKE cluster access and auth plugin guidance](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/cluster-access-for-kubectl).

### Other Kubernetes

Set `TARGET_KUBECONFIG` to the trusted kubeconfig from your cluster operator and use `examples/kubernetes.tfvars.example`; the same Terraform root also works with compatible vendor-native exec entries. Do not disable certificate checks.

## Verify and plan

```sh
kubectl --kubeconfig "$TARGET_KUBECONFIG" config get-contexts
kubectl --kubeconfig "$TARGET_KUBECONFIG" --context EXACT_CONTEXT \
  -n NAMESPACE auth can-i get deployments
```

The permission check is a sanity check, not a complete RBAC audit. Check the permissions needed by the rendered chart and by Helm's release metadata. Copy the selected template to `deploy/terraform/local.tfvars` and fill in the exact context, namespace, repository and digest. All relative paths in the variables are relative to this Terraform root; the default chart path is `../helm/orchestrata-api`.

```sh
terraform -chdir=deploy/terraform init -lockfile=readonly
terraform -chdir=deploy/terraform validate
terraform -chdir=deploy/terraform plan -var-file=local.tfvars -out=review.tfplan
```

Have the target owner review the plan and approve any live changes before applying it. The release will not create the namespace, waits for runtime readiness reported through Helm, and uses atomic rollback and cleanup on failures. Retain access to a known-good image digest for rollback via a new reviewed plan. Application uptime, IAM permissions, cluster connectivity and live provider compatibility still require verification.

## Offline/mock verification

```sh
terraform -chdir=deploy/terraform fmt -check -recursive
terraform -chdir=deploy/terraform init -backend=false -lockfile=readonly
terraform -chdir=deploy/terraform validate
terraform -chdir=deploy/terraform test
```

Provider initialization may require access to the Terraform Registry if it is not cached. The tests mock the Helm provider, use only plan runs, and do not access a cluster. The adjacent chart's `Chart.yaml` must be present. Provider syntax: [HashiCorp Helm provider](https://registry.terraform.io/providers/hashicorp/helm/latest/docs), [Helm release resource](https://registry.terraform.io/providers/hashicorp/helm/latest/docs/resources/release).
