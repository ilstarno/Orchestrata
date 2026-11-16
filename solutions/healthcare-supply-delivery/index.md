# Healthcare supply delivery

## Situation

Three fictional shipments weigh 90, 280, and 150 kg. Their receiving windows are 08:00–08:30, 08:05–08:40, and 08:10–08:50 respectively. A van must have enough capacity for its single job and be available for the entire receiving window. These tests are applied before solving to produce the eligible choices below.

## Input and rules

Each row maps to a JSON variable. Its eligible resource IDs are recorded in its domain. The required rule makes the assignments distinct. Prefer **East van** for **Care center PPE**; missing that choice costs 1 soft point without invalidating a feasible assignment.

## Verified result

| Item | Eligible choices | Assigned |
| --- | --- | --- |
| Clinic cartons | Cargo van, Compact van, Courier van | Compact van |
| Lab consumables | Cargo van | Cargo van |
| Care center PPE | Cargo van, Courier van, East van | East van |

Only Cargo van can cover Lab consumables. Compact van takes Clinic cartons, and East van matches the preference for the PPE shipment.

The returned resource IDs are `[1, 0, 3]` in input order. The hard score is 0 and the soft score is 0; the exact run evaluated 9 candidate assignments.

## What this example does not cover

Each van can cover only one of the overlapping jobs. There is no patient data, clinical decision, shared vehicle load, routing, or check of special handling beyond these weight and availability filters.
