terraform {
  required_version = ">= 1.5, < 2.0"

  required_providers {
    helm = {
      source  = "hashicorp/helm"
      version = "~> 3.0"
    }
  }
}

provider "helm" {
  kubernetes = {
    config_path    = pathexpand(var.kubeconfig_path)
    config_context = var.kubeconfig_context
  }
}

resource "helm_release" "orchestrata" {
  name             = var.release_name
  namespace        = var.namespace
  create_namespace = false
  chart            = "${path.module}/../../helm/orchestrata"
  atomic           = true
  cleanup_on_fail  = true
  wait             = true
  timeout          = 300

  values = [
    yamlencode({
      api              = { image = var.api_image }
      web              = { image = var.web_image }
      imagePullSecrets = [for name in var.image_pull_secrets : { name = name }]
    })
  ]
}

output "release" {
  value = {
    name      = helm_release.orchestrata.name
    namespace = helm_release.orchestrata.namespace
  }
}
