# Salesforce planning drafts

Two illustrative scenarios: Service Cloud agent routing and Sales Cloud renewal ownership and pricing handoff. Nothing connects to Salesforce, reads actual records, changes ownership, or approves terms. These are review drafts from the included deterministic JavaScript calculator, not runs of the Orchestrata engine.

```sh
node calculate.mjs > results.json
node verify.mjs
```

Each record specifies skills, region or territory, a modeled permission, priority, duration, and a service/review window. The modeled permission is an input to this example, not evidence of a Salesforce permission or an approval. Each candidate has a shift window and a cap on new plus current work. Prerequisites must be scheduled before the dependent step. Time is integer minutes relative to the plan start. A task can finish at the deadline.

The calculator visits ready records by highest priority, earliest deadline, then stable ID. It picks the earliest finish that respects prerequisites, eligibility, capacity and non-overlap. The reported holds are specific to this greedy draft, not proofs of global infeasibility. The verifier independently checks assignments, bounds, non-overlap, dependency timing, and holds including ordered review collisions.
