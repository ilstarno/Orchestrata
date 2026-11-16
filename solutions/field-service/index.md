## Three appointments, one time slot

The lift repair, safety inspection, and preventive maintenance happen at the same time. All three jobs need coverage. Alex is preferred for the lift repair, but only if each job can still be covered.

### What to provide

Make a list of the jobs and technicians in a stable order. Check qualifications and who is available for the slot before making each job’s eligible list. This example gives the solver the list of eligible people, not a separate skills database or calendar.

| Recipe field | Type | Example value |
| --- | --- | --- |
| `job-names` | List of strings | Lift repair, safety inspection, preventive maintenance |
| `technician-names` | List of strings | Alex, Sam, Noor; indices 0, 1, 2 |
| `eligible-pairs` | List of integers | Job and technician pairs: 0,0; 0,2; 1,0; 1,1; 2,1; 2,2 |
| `preferred-lift-technician` | Optional integer | 0 for Alex |

The normalized solver input uses one `variables` entry per job, with a string `name` and an integer `domain` of eligible technician indices. A hard `all_different` rule prevents double-booking. A soft `prefer_value` rule prefers Alex for lift repair with weight 5.

### What comes back

| Job | Eligible technicians | Assigned |
| --- | --- | --- |
| Lift repair | Alex, Noor | Alex |
| Safety inspection | Alex, Sam | Sam |
| Preventive maintenance | Sam, Noor | Noor |

The expected result is `values: [0, 1, 2]` in job order, hard score 0, soft score 0, and 8 combinations checked. Sam on inspection leaves Alex free for the preferred repair. Noor takes maintenance.

### Run it

Save the [input](problem.json) as `problem.json`. With the Orchestrata CLI on your path:

```sh
orchestrata-cli problem.json
```

Compare the result with the [expected output](expected.json). The same problem can be sent to the [local API](../../docs/api/).

### Where this example stops

This is assignment within an already chosen time slot. It does not pick appointment start times, calculate travel, plan a route, sync calendars, or dispatch anyone.
