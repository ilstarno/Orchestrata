## Match overlapping flights to compatible gates

At a small terminal, three flights need gates at the same time. Compatibility is provided before the solve. The fixture uses integer gate IDs, not actual gate names.

| Flight | Eligible gate IDs | Assigned gate ID |
| --- | --- | --- |
| Flight 201 | 0, 1 | 0 |
| Flight 415 | 0, 2 | 2 |
| Flight 630 | 1, 2 | 1 |

The input uses `variables[].name` for the flight and `variables[].domain` for eligible gates. An `all_different` rule forbids gate reuse; `prefer_value` gives Flight 201 a preference for gate 0. The exact result is `[0, 2, 1]` in flight order, with hard and soft scores of 0 and 8 combinations evaluated.

[Download the problem](../../fixtures/airport-gate-assignment/problem.json) · [Download the expected result](../../fixtures/airport-gate-assignment/expected.json)

This is a three-flight overlap, not a full gate schedule. Aircraft type, turnaround buffers, towing, passenger transfers, and disruption recovery are not modeled.
