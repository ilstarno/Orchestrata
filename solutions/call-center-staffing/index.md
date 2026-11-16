## Cover three support queues at the same time

A support manager needs one agent per queue for a single coverage window. The IDs below identify eligible agents, not named people.

| Queue | Eligible agent IDs | Assigned agent ID |
| --- | --- | --- |
| Billing escalations | 0, 2 | 2 |
| Spanish-language support | 0, 1 | 0 |
| Account recovery | 1, 2 | 1 |

The input puts each queue in `variables[].name` and its eligible agent IDs in `variables[].domain`. One `all_different` rule prevents double-booking. A `prefer_value` preference with soft weight 5 favors agent 2 for billing escalations. The exact result is `[2, 0, 1]` in queue order, with hard and soft scores of 0 and 8 combinations evaluated.

[Download the problem](../../fixtures/call-center-staffing/problem.json) · [Download the expected result](../../fixtures/call-center-staffing/expected.json)

The fixture does not forecast arrivals, size staffing from service-level targets, schedule breaks, or send calls to agents.
