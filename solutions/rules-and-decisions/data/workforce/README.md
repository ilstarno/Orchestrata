# Thanksgiving-week staffing: Albany County retail

This worked example plans November 22–28, 2026 in `America/New_York` (EST/UTC−05:00 in this window). It represents two private, non-factory retail work sites outside New York City and a company of 128 employees; the ten anonymous adult roster entries are assumed covered and nonexempt. Sunday operations and predesignated rest windows are explicit inputs. No union agreement, special classification, approved variance, actual payroll, or live employee records are included.

The fixture assumes the employer's fixed FLSA workweek and the model's Sunday-to-Saturday calendar week coincide. It covers the complete specified week, not just Friday automation. Each person's employer-wide work outside the model is explicitly assumed zero. Hours, overtime, meals and paid absence are integer minutes. A live setup needs the employer's actual workweek and reconciled hours across all sites and work types. This is a scheduling demonstration under the listed assumptions, not legal advice, a payroll calculator, or a claim of compliance.

## What the plan changes

| Plan | Covered / requested | Held | Weekly overtime hours | Of which new from open work |
| --- | ---: | ---: | ---: | ---: |
| First permitted worker | 34 / 37 | 3 | 7.5 | 5.0 |
| Recommended | 34 / 37 | 3 | 2.5 | 0.0 |
| Replan after two unavailable windows | 32 / 37 | 5 | 2.5 | 0.0 |

All 27 confirmed shifts are kept. Baseline picks W02 for Saturday checkout O-07: 37.5 scheduled work hours before that shift plus 7.5 hours of duty equals 45 hours, yielding 5 overtime hours. The recommended roster assigns the same shift to the available, trained W03 instead. The remaining 2.5 overtime hours already exist in the locked inventory schedule; the optimizer does not erase overtime or stop treating it as payable. No cost or savings in dollars is inferred.

Three holds remain in the recommended plan: O-02 arrived for a store that the company's Thanksgiving policy closes; O-09 lacks an available worker with floor training and no conflicting commitment; O-10 has the proposed meal at 15:00, outside the 11:00–14:00 meal window this shift requires. In the disruption, Friday checkout O-04 and Saturday keyholder O-08 also stay open. For O-08 the previously recommended worker becomes unavailable, one alternative is inside a designated rest block, one has approved vacation, and another is already committed to the other store. These are shifts held for a coordinator, not assessments of staff.

`operational-case.json` records the shift, date, work site, meal schedule, assignment decision, and all vetoing rule IDs on considered alternatives. For a feasible alternative the audit shows its projected overtime minutes if it took that shift while other assignments stayed fixed. A proposal assumes pre-cleared relief capacity exists during meal slots; supervisors should check coverage and actual duty-free release before relying on the schedule.

## Where the rules come from

Official sources checked November 16, 2026. Links are descriptive context for the scoped fixtures, not advice on an individual situation.

| Rule in the model | Source and relevant limitation |
| --- | --- |
| `US-FLSA-40` | [U.S. DOL Fact Sheet 23](https://www.dol.gov/agencies/whd/fact-sheets/23-flsa-overtime-pay): for covered, nonexempt workers, hours worked above 40 in a fixed workweek are overtime. The replay reports minutes, not a rate or amount; it does not cap work at 40 hours. |
| `US-HOURS-WORKED`, `US-PAID-ABSENCE` | [U.S. DOL Fact Sheet 22](https://www.dol.gov/agencies/whd/fact-sheets/22-flsa-hours-worked) and [Hours Worked Advisor](https://webapps.dol.gov/elaws/whd/flsa/hoursworked/screenEr14.asp): bona fide duty-free meals can be excluded; paid time off that is not worked does not add to FLSA hours worked. The modeled 20-minute evening interval is paid and included explicitly. |
| `NY-161-REST` | [NY Labor Law §161](https://www.nysenate.gov/legislation/laws/LAB/161): this assumed covered mercantile workplace preserves predesignated 24-hour rest windows and models no variance. Actual designation/notice evidence and exceptions need separate review. |
| `NY-162-NOON`, `NY-162-EVENING`, `NY-162-LATE` | [NY Labor Law §162](https://www.nysenate.gov/legislation/laws/LAB/162): daytime shifts in the example spanning 11:00–14:00 and lasting more than six hours require a 30-minute meal within that window. A modeled 10:30–19:30 shift also has 20 minutes between 17:00 and 19:00. A modeled non-factory 14:00–21:45 shift has a 45-minute interval centered at its midpoint. Overnight shifts, borderline windows and waiver/permit cases are not evaluated. |
| `NY-196B-APPROVED` | [NY Labor Law §196-b](https://www.nysenate.gov/legislation/laws/LAB/196-B) and [NY paid sick leave guidance](https://www.ny.gov/programs/new-york-paid-sick-leave): employer-size requirements apply; a 100+ employee employer must provide up to 56 hours of paid sick leave annually under the statute. The replay only honors an already approved unavailable window. It does not infer leave eligibility, determine benefits, accrue or pay leave, or store an absence reason. |
| `NY-142-SCOPE` | [NY Miscellaneous Industries Wage Order](https://dol.ny.gov/minimum-wage-order-miscellaneous-industries-and-occupations-cr142) is a state payroll implementation reference. State rules such as spread-of-hours or call-in pay are outside the calculation; the example's own one-shift-per-day limits reduce the modeled schedule scope. |
| `US-NY-HOLIDAY` | [OPM federal holiday calendar](https://www.opm.gov/policy-data-oversight/pay-leave/federal-holidays/) lists November 26, 2026; [NY General Construction Law §24](https://www.nysenate.gov/legislation/laws/GCN/24) specifies the fourth Thursday in November. Neither calendar table in this example is used to infer a private closure or premium. |

Company-only rules keep SITE-A closed on Thanksgiving and SITE-B at limited hours, protect approved vacation, verify training tags and each site's roster, disallow two shifts in one day, and attach an operations-supplied relief reference to planned meal breaks. The comparison baseline takes the first eligible worker in the named pool. The recommended solution first minimizes unfilled demand, then overtime and changes to prior assignments, using stable IDs only for a final reproducible tie break. An unfillable slot stays held instead of weakening a rule.

## Reproduce

Python 3.11+ with standard library and the system's IANA time zone data, with no network access or credentials:

```sh
python3 -B replay.py
python3 -B verify.py
```

`inputs.json` has the raw worker roster, availability, leave approval blocks, rest designations, site calendar, seat-level demand and disruption windows; `make_inputs.py` rebuilds this fixed dataset. `visual.json` is the buyer-page comparison; `operational-case.json` preserves the raw tables, alternative vetoes, full roster output, and trace back to a rule and exact local meal times. The independent verifier enumerates feasible plans from the JSON inputs, checks the recorded optimum for the bounded instance, and independently checks worked/paid absence totals and the immutable roster. It validates the model's stated rules and results, not the employer's legal obligations in another setting.
