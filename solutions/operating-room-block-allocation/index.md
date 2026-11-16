# Operating room blocks

## Situation

The facility supplies approved room choices for three sample placeholders in the same fixed block. Each placeholder needs a distinct room.

## Input and rules

Each row maps to a JSON variable. Its eligible resource IDs are recorded in its domain. The required rule makes the assignments distinct. Prefer **OR South** for **slot-b**; missing that choice costs 1 soft point without invalidating a feasible assignment.

## Verified result

| Item | Eligible choices | Assigned |
| --- | --- | --- |
| slot-a | OR North, OR South | OR North |
| slot-b | OR Central, OR South | OR South |
| slot-c | OR Central, OR South | OR Central |

Using OR South for Slot B leaves OR Central for Slot C and OR North for Slot A.

The returned resource IDs are `[0, 2, 1]` in input order. The hard score is 0 and the soft score is 0; the exact run evaluated 8 candidate assignments.

## What this example does not cover

This is room logistics, not a clinical tool. It does not determine clinical suitability, priority, intervention timing, clinical staffing, turnover, or emergency capacity.
