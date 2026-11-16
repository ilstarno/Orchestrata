# Dental treatment rooms

## Situation

Three fictional bookings share one fixed time block. The clinic supplies approved rooms for each booking; a room can only be assigned once.

## Input and rules

Each row maps to a JSON variable. Its eligible resource IDs are recorded in its domain. The required rule makes the assignments distinct. Prefer **Room West** for **booking-a**; missing that choice costs 1 soft point without invalidating a feasible assignment.

## Verified result

| Item | Eligible choices | Assigned |
| --- | --- | --- |
| booking-a | Room North, Room West | Room West |
| booking-b | Room North, Room East | Room North |
| booking-c | Room East, Room West | Room East |

Booking A keeps its preferred Room West. Booking B uses Room North and Booking C uses Room East.

The returned resource IDs are `[2, 0, 1]` in input order. The hard score is 0 and the soft score is 0; the exact run evaluated 8 candidate assignments.

## What this example does not cover

This allocation does not set appointment durations, evaluate clinical suitability, manage room turnover, or make care decisions.
