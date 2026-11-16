## Assign three concurrent workloads to separate nodes

Three infrastructure jobs are ready at once. The operator decides which nodes can accept each job before passing those choices in as node IDs.

| Workload | Eligible node IDs | Assigned node ID |
| --- | --- | --- |
| GPU batch job | 0, 2 | 2 |
| Search index rebuild | 0, 1 | 0 |
| Analytics export | 1, 2 | 1 |

Each workload supplies `variables[].name` and `variables[].domain`. The `all_different` rule gives each workload its own node; `prefer_value` favors node 2 for the GPU batch. The exact result is `[2, 0, 1]` in workload order, with hard and soft scores of 0 and 8 combinations evaluated.

[Download the problem](../../fixtures/cluster-workload-placement/problem.json) · [Download the expected result](../../fixtures/cluster-workload-placement/expected.json)

This is exclusive placement with precomputed eligibility, not a scheduler for CPU or memory packing, resource quotas, retries, affinity, or autoscaling.
