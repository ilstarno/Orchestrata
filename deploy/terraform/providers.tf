provider "helm" {
  helm_driver = "secret"

  kubernetes = {
    config_path    = abspath(pathexpand(var.kubeconfig_path))
    config_context = var.kubeconfig_context
    insecure       = false
  }
}
