# School classroom assignment

## Situation

Chemistry lab, Digital design, and Civics seminar already meet in second period. Staff supply eligibility based on room equipment, capacity, accessibility, and availability. Each class needs a different room.

## Input and rules

Each row maps to a JSON variable. Its eligible resource IDs are recorded in its domain. The required rule makes the assignments distinct. Prefer **South Lab** for **civics_seminar**; missing that choice costs 5 soft points without invalidating a feasible assignment.

## Verified result

| Item | Eligible choices | Assigned |
| --- | --- | --- |
| chemistry_lab | North Lab, South Lab | North Lab |
| digital_design | North Lab, Media Studio | Media Studio |
| civics_seminar | South Lab, Media Studio | South Lab |

Civics seminar keeps South Lab, nearer to its preceding class. Chemistry lab uses North Lab and Digital design uses Media Studio.

The returned resource IDs are `[0, 2, 1]` in input order. The hard score is 0 and the soft score is 0; the exact run evaluated 8 candidate assignments.

## What this example does not cover

This room assignment does not choose periods, assign teachers, check student conflicts, or build a weekly timetable.
