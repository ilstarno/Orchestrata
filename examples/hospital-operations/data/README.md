# Hospital operations worklists

Six illustrative nonclinical planning drafts: New York appointment administration, Rome diagnostic room logistics, Tirana sealed supplies transport, London room-cleaning logistics, Paris instrument set transport and paperwork, and Munich facilities shift handoff. City labels organize the examples; no named hospital, measured behavior, patient, booking, clinical eligibility, treatment, room clearance, patient movement, or site-specific floor plan is represented.

```sh
python3 calculate.py
python3 calculate.py --check
python3 verify.py
```

The calculator books tasks in the supplied administrative order. It tests role capability, operator and work windows, prerequisites, shared equipment conflicts, and an explicit requirement for sample evidence. Tasks lacking an eligible slot or evidence are held for the operational reviewer. Booked times are relative illustrative minutes, not real staffing or performance. The planner's policy and scope are not healthcare policy. Public NHS references in the inputs provide general workflow context and do not imply legal or local applicability.

The verifier independently enumerates feasible start minutes, checks all 31 decisions, and verifies the declared earliest-fit policy while accounting for prior bookings. This is not a globally optimized schedule or an Orchestrata-engine run; nothing connects to a healthcare system. `visual.json` is the renderable summary, `inputs.json` preserves the resource and work windows, and `operational-case.json` traces structured decision evidence.
