# Staff a week across two warehouses

A scheduler needs to cover North Warehouse and South Exchange for five days. Each site opens two eight-hour shifts. Every shift requires a handover lead and two dispatch operators. Monday's early shift also needs an authorized controlled-goods specialist at each site.

The proposed roster covers all 62 seats. Then a specialist at South Exchange calls out. North Warehouse adds one late dispatch seat; South Exchange also requests late controlled-goods coverage. The planner fills the added dispatch seat, keeps 61 original assignments, and puts both uncovered South Exchange specialist seats on hold. No one else fits the location, skill, authorization, and availability requirements, even before checking hour caps. A supervisor can arrange authorized relief or move the controlled work to a covered window.

| Scenario | Required | Assigned | Unfilled | Paid hours | Kept from baseline |
| --- | ---: | ---: | ---: | ---: | ---: |
| Staff both sites | 62 | 62 | 0 | 496 | — |
| Specialist absent; late demand climbs | 64 | 62 | 2 | 496 | 61 |

## What to provide

| Input | Meaning |
| --- | --- |
| `metadata.weekStarting`, `timeZone` | Local roster week and clock. |
| `locations[]` | Site IDs used by demand and worker authorizations. |
| `workers[].id`, `homeLocation`, `authorizedLocations` | Sample worker key, preferred site, and hard site-access boundaries. |
| `workers[].skills`, `roleGrants` | Skill and permission are separate checks. |
| `workers[].availability[]` | Local date, start, and end. The entire work window must fit. |
| `workers[].weeklyTargetHours`, `weeklyCapHours`, `preferredShift` | Fair distribution target, hard individual ceiling, and preferred shift. A target is not a minimum assignment guarantee. |
| `coverage[]` | Each seat has ID, site, shift, date and start/end, role, required skill and required permission. |
| `policy` | Example rules: one shift per day, at least 12 hours between shifts, and no more than 40 hours per week or the worker's lower cap. Meal relief is outside this example. |
| `scenarios[]` | Changes to the baseline demand, dated absences, and the planner's grants. Planning and publishing are separate permissions. |

## What to review

`scenarios[].coverage[]` holds the assigned worker or a hold reason per seat, including the eligible candidate count when the seat was processed. `holds[]` shows why uncovered seats could not be assigned and who must act on publication. Counts under `rejected` record the first failed eligibility check per worker; they are not overlapping totals. `metrics` reconciles seats, worker count, paid hours, and assignment changes.

This is a vendor-neutral example of planning with sample workforce data. It did not connect to a Workday environment, call a Workday API, or publish a roster. The working-time limits are sample site policies, not a legal compliance claim. The deterministic scheduler checks feasible assignments and prefers to retain the published baseline, but does not assert global optimality.

Recompute and compare checked JSON with `python3 reproduce.py --check`. Omit `--check` to regenerate the input and output files.
