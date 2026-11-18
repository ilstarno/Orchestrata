locals {
  chart_path = abspath(pathexpand(var.chart_path))
  values = concat(
    [for value_file in var.values_files : file(pathexpand(value_file))],
    [yamlencode({
      image = {
        repository = var.image_repository
        digest     = var.image_digest
      }
    })],
  )
}

resource "helm_release" "orchestrata_api" {
  name             = var.release_name
  chart            = local.chart_path
  namespace        = var.namespace
  values           = local.values
  create_namespace = false
  atomic           = true
  cleanup_on_fail  = true
  wait             = true
  timeout          = var.release_timeout_seconds
  max_history      = 5

  lifecycle {
    precondition {
      condition     = fileexists("${local.chart_path}/Chart.yaml")
      error_message = "chart_path must name a local Helm chart directory containing Chart.yaml."
    }
  }
}
