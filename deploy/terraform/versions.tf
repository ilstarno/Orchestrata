terraform {
  required_version = ">= 1.7, < 2.0"

  required_providers {
    helm = {
      source  = "hashicorp/helm"
      version = "~> 3.3.0"
    }
  }
}
