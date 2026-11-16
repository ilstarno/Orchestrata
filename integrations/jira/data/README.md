# Jira planning examples

Three worked examples: prioritize a Jira Service Management queue, sequence a Jira Software release, and plan incident follow-up in a change window. These are offline examples and do not connect to Atlassian, use credentials, read real issues, mutate anything in Jira, or represent results from an Orchestrata engine call. A planner must review the output.

```sh
node calculate.mjs > results.json
node verify.mjs
```

Inputs: issue keys, skills, priorities, uninterrupted work durations, allowed time windows, prerequisites, and sample approval status. A resource has a skill list and one working window. Times are integer minutes from the start of an illustrative planning period. Work may end at a window boundary; back-to-back assignments are allowed. No SLA pause rules, holidays, Jira approval records, or real team schedules are implied.

The deterministic planner considers ready issues in priority order, then deadline, then issue key. It picks the earliest finishing eligible slot, then stable resource ID. The planner is greedy and does not prove any hold would remain under a different sequence or staffing. The independent checker verifies recorded holds, skills, duration, windows, dependencies, and non-overlap; it also walks every candidate minute for examples with a deadline hold.
