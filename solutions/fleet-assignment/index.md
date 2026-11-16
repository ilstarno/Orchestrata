## Match three overlapping jobs to vans

The depot delivery carries 700 kg and runs from 09:10 to 09:50. It needs a van that can handle that weight and remain available for the entire job. The parcel pickup and small-box drop overlap it, so they cannot use the same van.

### What to provide

| Recipe field | Type | What to enter |
| --- | --- | --- |
| `job-names` | List of strings | Job labels in order |
| `job-weights-kg` | List of integers | Whole kilograms per job |
| `job-windows-minutes` | List of integers | Start and end pairs in minutes after midnight |
| `vehicle-names` | List of strings | Van labels in index order |
| `vehicle-capacities-kg` | List of integers | Usable load limit in kilograms per van |
| `vehicle-windows-minutes` | List of integers | Availability start and end pairs on the same day |
| `eligible-pairs` | List of integers | Job and vehicle index pairs after checking weight and availability |
| `preferred-depot-vehicle` | Optional integer | 2, the late-start van |

| Van | Capacity | Availability |
| --- | --- | --- |
| Large van | 1,000 kg | 08:30–10:20 |
| Short-shift van | 500 kg | 08:50–09:40 |
| Late-start van | 800 kg | 09:05–10:30 |

Use the [source inputs](inputs.json) to inspect the original minute values. Before solving, the example turns each job’s eligible vehicles into an integer `domain`. The solver receives those domains, a hard `all_different` rule, and one soft `prefer_value` rule with weight 1. It does not compute compatibility from weights or times during the solve.

### What comes back

| Job | Load | Service window | Assigned |
| --- | --- | --- | --- |
| Parcel pickup | 450 kg | 09:00–09:30 | Short-shift van |
| Depot delivery | 700 kg | 09:10–09:50 | Late-start van |
| Small-box drop | 150 kg | 09:15–10:00 | Large van |

The late-start van cannot take the parcel pickup because it is not available at 09:00. The short-shift van cannot take the depot delivery because of both load and availability. The expected result is `values: [1, 2, 0]` in job order, hard and soft scores 0/0, and 8 combinations checked.

### Run it

Save the [normalized problem](problem.json) and run:

```sh
orchestrata-cli problem.json
```

Compare the [expected output](expected.json), or run the problem on the [local API](../../docs/api/).

### Where this example stops

These are independent jobs in overlapping windows. The example does not order stops or calculate travel time, group loads across stops, link a pickup to a delivery, or produce a route. The catalog’s larger benchmark inputs generate stop data, not solved routes or measured route performance.
