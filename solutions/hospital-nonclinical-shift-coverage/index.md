# Hospital operations staffing

## Situation

Visitor desk, supply desk, and courier intake need coverage in one fixed block. The facility supplies the people available and cleared for each post; a person can cover only one post.

## Input and rules

Each row maps to a JSON variable. Its eligible resource IDs are recorded in its domain. The required rule makes the assignments distinct. Prefer **Staff B** for **visitor-desk**; missing that choice costs 1 soft point without invalidating a feasible assignment.

## Verified result

| Item | Eligible choices | Assigned |
| --- | --- | --- |
| visitor-desk | Staff A, Staff B | Staff B |
| supply-desk | Staff C | Staff C |
| courier-intake | Staff A, Staff B | Staff A |

Supply desk has only one eligible worker, Staff C. Assigning Staff B to Visitor desk leaves Staff A for Courier intake.

The returned resource IDs are `[1, 2, 0]` in input order. The hard score is 0 and the soft score is 0; the exact run evaluated 4 candidate assignments.

## What this example does not cover

This is nonclinical logistics only. It does not determine clinical staffing, qualifications, staffing ratios, breaks, or coverage across blocks.
