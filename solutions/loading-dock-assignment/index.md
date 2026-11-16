# Loading dock assignment

## Situation

Three trailers overlap at 08:30. Equipment, trailer fit, and availability determine which doors are eligible. A door can serve only one of these appointments.

## Input and rules

Each row maps to a JSON variable. Its eligible resource IDs are recorded in its domain. The required rule makes the assignments distinct. Prefer **Door 7** for **parcel_outbound**; missing that choice costs 5 soft points without invalidating a feasible assignment.

## Verified result

| Item | Eligible choices | Assigned |
| --- | --- | --- |
| refrigerated_inbound | Door 4, Door 7 | Door 4 |
| parcel_outbound | Door 7, Door 9 | Door 7 |
| palletized_transfer | Door 4, Door 9 | Door 9 |

Parcel outbound keeps Door 7 near its staging area. Refrigerated inbound takes Door 4 and Palletized transfer takes Door 9.

The returned resource IDs are `[0, 1, 2]` in input order. The hard score is 0 and the soft score is 0; the exact run evaluated 8 candidate assignments.

## What this example does not cover

This does not set arrival times, sequence work, route trucks, or schedule crews.
