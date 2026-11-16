output "release_name" {
  description = "Helm release name."
  value       = helm_release.orchestrata_api.name
}

output "namespace" {
  description = "Existing Kubernetes namespace where the release runs."
  value       = helm_release.orchestrata_api.namespace
}
