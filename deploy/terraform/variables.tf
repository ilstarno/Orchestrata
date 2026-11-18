variable "kubeconfig_path" {
  type        = string
  description = "Path to one trusted kubeconfig. Treat kubeconfig exec entries as local code."

  validation {
    condition     = trimspace(var.kubeconfig_path) != ""
    error_message = "Set an explicit kubeconfig path."
  }
}

variable "kubeconfig_context" {
  type        = string
  description = "Exact context in that kubeconfig; no implicit current-context fallback."

  validation {
    condition     = trimspace(var.kubeconfig_context) != ""
    error_message = "Set the target kubeconfig context explicitly."
  }
}

variable "namespace" {
  type        = string
  description = "An existing namespace for this release. The stack never creates a namespace."

  validation {
    condition     = can(regex("^[a-z0-9]([-a-z0-9]{0,61}[a-z0-9])?$", var.namespace))
    error_message = "Use a valid Kubernetes namespace name (1–63 characters)."
  }
}

variable "release_name" {
  type        = string
  description = "Name for the Helm release."
  default     = "orchestrata-api"

  validation {
    condition     = can(regex("^[a-z0-9]([-a-z0-9]{0,51}[a-z0-9])?$", var.release_name))
    error_message = "Use a lowercase Helm release name of 1–53 characters (letters, digits, or hyphens)."
  }
}

variable "chart_path" {
  type        = string
  description = "Path to the local Orchestrata API chart directory. Relative to the Terraform root."
  default     = "../helm/orchestrata-api"
}

variable "image_repository" {
  type        = string
  description = "OCI image repository without an image tag or digest, e.g. registry.example.com/orchestrata/api."

  validation {
    condition     = can(regex("^(([a-z0-9][a-z0-9.-]*[.][a-z0-9]+|localhost)(:[0-9]{1,5})?|[a-z0-9]+:[0-9]{1,5})(/[a-z0-9]+([._-]+[a-z0-9]+)*)+$", var.image_repository))
    error_message = "Provide a registry and repository path without a tag, digest, whitespace, or scheme."
  }
}

variable "image_digest" {
  type        = string
  description = "Immutable digest of the already-published image. No registry publication is performed here."

  validation {
    condition     = can(regex("^sha256:[0-9a-f]{64}$", var.image_digest))
    error_message = "Set the actual immutable image digest: sha256 followed by 64 lowercase hex digits."
  }
}

variable "values_files" {
  type        = list(string)
  description = "Ordered, non-secret Helm values file paths. These may reference existing resources by name."
  default     = []
}

variable "release_timeout_seconds" {
  type        = number
  description = "Timeout for operations on the Helm release, in whole seconds."
  default     = 300

  validation {
    condition = (
      var.release_timeout_seconds == floor(var.release_timeout_seconds) &&
      var.release_timeout_seconds >= 60 &&
      var.release_timeout_seconds <= 1800
    )
    error_message = "Use a whole timeout between 60 and 1800 seconds."
  }
}
