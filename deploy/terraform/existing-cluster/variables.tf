variable "kubeconfig_path" {
  type        = string
  description = "Kubeconfig file outside the repository. Do not embed credentials."
  default     = "~/.kube/config"
}

variable "kubeconfig_context" {
  type        = string
  description = "Explicit existing target context; never falls back to the current context."

  validation {
    condition     = length(trimspace(var.kubeconfig_context)) > 0
    error_message = "The kubeconfig context must not be empty."
  }
}

variable "namespace" {
  type        = string
  description = "Existing namespace prepared by the cluster operator."
}

variable "release_name" {
  type        = string
  description = "Helm release name."
  default     = "orchestrata"
}

variable "api_image" {
  type        = string
  description = "Published Orchestrata API image reference. Prefer a digest."
}

variable "web_image" {
  type        = string
  description = "Published Orchestrata web image reference. Prefer a digest."
}

variable "image_pull_secrets" {
  type        = list(string)
  description = "Names of existing pull secrets in the target namespace; never their contents."
  default     = []
}
