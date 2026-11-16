# ServiceNow planning examples

Three offline examples: incident queues, scheduled changes, and outgoing-to-incoming duty handoffs. These examples use illustrative support records and duty rosters, not a ServiceNow connection, release approval, ticket assignment, on-call change, or a production write. The calculator is a small deterministic review tool, not an Orchestrata engine run.

```sh
node calculate.mjs > results.json
node verify.mjs
```

The 24 records carry priority, required skills, support group (the JSON `region` field), a modeled authority, estimate and time window. Operators have windows and a cap on existing and new work. The clock is relative integer minutes. Stage-change tasks require explicitly supplied sample approval evidence before booking. A dependent task cannot start before all predecessors finish. A window's end is an allowed finish.

Ready work is considered in priority, deadline, and stable ID order. The earliest eligible finish is selected, with approval, capacity and non-overlap constraints. The verifier independently checks every assignment and hold. A held item needs review; this earliest-finish plan does not prove infeasibility if coverage, priorities, or staffing change.
