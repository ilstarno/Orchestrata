# Cold-chain loading: a shared dock and six compartments

This worked example plans 24 sealed loads through three scheduled departures on November 16, 2026 UTC, each with two independent compartments. It uses shipper-approved handling profiles instead of a fixed universal temperature. No live sensors, actual food lots or product records, site connections, loading actions, shipment authorizations, or destination routing are involved. In the model, `loaded` means **proposed for a compartment and staging window**, not physically loaded.

| Comparison | Proposed loads | Held | Aggregate wait, load-minutes | Proposed mass |
| --- | ---: | ---: | ---: | ---: |
| First feasible choice | 10 of 24 | 14 | 120 | 2,030 kg |
| Recommended | 13 of 24 | 11 | 150 | 2,480 kg |
| Compartment and dock disruption | 10 of 24 | 14 | 110 | 1,855 kg |

First fit places three flexible profile-A loads in the earliest staging slot. The recommended plan moves each to a later slot, keeping the first slot for loads with only that choice. Six compartments independently protect mandatory mass and volume reserves: several individual loads fit by themselves but cannot fit alongside the recommended contents. Total waiting rises by 30 load-minutes while three more loads are placed. These are arithmetic results from the worked dataset, not operational savings or a safety claim. The lower wait for the disruption includes fewer loads and should not be read as better performance.

In the disruption, the second departure's B compartment becomes unavailable and the third departure loses an East dock slot. A load cannot move to another departure, break its packaging into smaller pieces, or borrow the compartment reserve. Among plans with maximum coverage, the disruption comparison first preserves existing assignments and then reduces staging wait.

## Rules, inputs and decisions

- Both rated mass and volume have explicit reserves; the available capacity is rated capacity minus reserve. Both limits apply independently to each compartment.
- The exact compartment must be on the shipper's approval list, be ready for the requested profile, accept sealed loads and have a preparation reference. The lot must have a verified observation in the input. Stale, missing or incomplete evidence stays held for review. The labels are data supplied by the fixture, not safety thresholds.
- A staging slot is feasible only when its entire half-open interval falls after release/readiness and before the supplied staging cutoff, compartment close and departure. The readiness timestamp and cutoff are explicit UTC; the receiver reference is not an arrival-route calculation.
- Each physical dock handles one load at a time in this model. Two simultaneous slots are allowed only when they use different docks. Wave intervals do not overlap and the compartments are not shared, allowing separate exact searches.
- Assigned and held rows expose evidence-specific reason codes. For example, L-1-06 has no temperature evidence; L-2-08 exceeds the available volume despite fitting by mass; L-3-08 lacks explicit shipper approval; L-1-07 finishes preparation after the allowed staging slot starts. When an individually eligible option is held, the decision trace names the actual dock and/or combined capacity conflicts with the chosen board.

## Regulatory context, not operational approval

The [FDA FSMA Sanitary Transportation final rule overview](https://www.fda.gov/food/food-safety-modernization-act-fsma/fsma-final-rule-sanitary-transportation-human-and-animal-food), [FDA small-entity guide](https://www.fda.gov/regulatory-information/search-fda-guidance-documents/small-entity-compliance-guide-sanitary-transportation-human-and-animal-food-what-you-need-know-about), and [21 CFR §1.908](https://www.ecfr.gov/current/title-21/chapter-I/subchapter-A/part-1/subpart-O/subject-group-ECFRe79dfa68f350fd3/section-1.908) were checked on November 16, 2026. They provide U.S. context for applicable food transport roles, preparation and shipper specifications. Who is covered, requirements under any agreement, sanitary conditions, actual excursion investigations, and shipment disposition all require independent review. The optimizer cannot approve release, replace records, or establish whether a product can safely ship.

## Reproduce the comparison

With Python 3.10+ and no external packages or credentials:

```sh
python3 -B replay.py
python3 -B verify.py
```

Use `replay.py --write` to regenerate the input and output from the fixed fixture. The independent verifier reads the JSON, verifies the selected requests against the hard constraints, reruns Cartesian search with an independent implementation, checks the tie breaks and verifies the totals. `inputs.json` contains the 24 requests, six compartments, dock windows, dated source and disruption. `operational-case.json` also carries options, rejected options, per-compartment totals, and a trace linking a hold to its evidence and reviewer boundary. `visual.json` is the derived compact view for the website.
