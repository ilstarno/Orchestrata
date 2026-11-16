# Airline crew assignment

## Situation

Three flight duties overlap. Operations supplies each duty’s eligible crews after checking fleet qualifications and availability. Each crew can cover at most one duty during the overlap.

## Input and rules

Each row maps to a JSON variable. Its eligible resource IDs are recorded in its domain. The required rule makes the assignments distinct. Prefer **Crew Bravo** for **Duty 801**; missing that choice costs 1 soft point without invalidating a feasible assignment.

## Verified result

| Item | Eligible choices | Assigned |
| --- | --- | --- |
| Duty 801 | Crew Alpha, Crew Bravo | Crew Bravo |
| Duty 816 | Crew Alpha, Crew Charlie | Crew Alpha |
| Duty 842 | Crew Bravo, Crew Charlie | Crew Charlie |

Crew Bravo takes Duty 801, leaving Crew Alpha for Duty 816 and Crew Charlie for Duty 842.

The returned resource IDs are `[1, 0, 2]` in input order. The hard score is 0 and the soft score is 0; the exact run evaluated 8 candidate assignments.

## What this example does not cover

This check does not evaluate duty-time legality, rest, crew pairing, reserve recovery, or seniority.
