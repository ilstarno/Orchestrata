mock_provider "helm" {}

variables {
  kubeconfig_path    = "/nonexistent/terraform-test-kubeconfig"
  kubeconfig_context = "mock-only"
  namespace          = "orchestrata"
  image_repository   = "registry.example.com/orchestrata/api"
  image_digest       = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
}

run "existing_namespace_release" {
  command = plan

  assert {
    condition = (
      !helm_release.orchestrata_api.create_namespace &&
      helm_release.orchestrata_api.atomic &&
      helm_release.orchestrata_api.cleanup_on_fail &&
      helm_release.orchestrata_api.wait
    )
    error_message = "The release must use the existing namespace, wait, and clean up on failure."
  }

  assert {
    condition     = yamldecode(helm_release.orchestrata_api.values[0]).image.digest == var.image_digest
    error_message = "The release must receive the selected immutable image digest."
  }
}

run "reject_image_tag_in_repository" {
  command = plan

  variables {
    image_repository = "registry.example.com/orchestrata/api:latest"
  }

  expect_failures = [var.image_repository]
}

run "reject_unpinned_image" {
  command = plan

  variables {
    image_digest = "latest"
  }

  expect_failures = [var.image_digest]
}

run "reject_implicit_context" {
  command = plan

  variables {
    kubeconfig_context = ""
  }

  expect_failures = [var.kubeconfig_context]
}
