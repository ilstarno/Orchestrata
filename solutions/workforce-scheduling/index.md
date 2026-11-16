## Cover four shifts after someone calls out

Both stores need two workers at 14:00. Ben becomes unavailable. Compare the baseline with a second run that removes Ben from the two slots he could cover. The shift list, rules, weights, and other eligible choices stay the same.

### What to provide

List each location and role, each worker qualified and available for that shift, any fixed assignment, and the preferences that can be missed. Keep worker indices stable across both runs so the results can be compared.

| Recipe field | Type | Example value |
| --- | --- | --- |
| `shift-names` | List of strings | North keyholder, North floor, South register, South stock |
| `worker-names` | List of strings | Ana, Ben, Clara, Diego, Elena; indices 0 through 4 |
| `eligible-pairs` | List of integers | Pairs of shift and worker indices; remove 1,1 and 2,1 for Ben’s absence |
| `pinned-pairs` | Optional list of integers | 0,0 fixes Ana on North keyholder |
| `preferred-assignments` | Optional list of integers | Shift, worker, weight triples: 1,1,5; 2,2,3; 3,3,2 |

In the solver input, each shift is a variable. A fixed assignment is a domain with a single allowed worker: `north_keyholder` has `domain: [0]`. The hard `all_different` rule stays the same in both runs. The three `prefer_value` rules keep their original weights even when a preference becomes impossible.

### Compare the plans

| Shift | Eligible before callout | Baseline | After callout |
| --- | --- | --- | --- |
| North keyholder | Ana | Ana | Ana |
| North floor | Ben, Clara | Ben | Clara |
| South register | Ben, Clara, Elena | Clara | Elena |
| South stock | Diego, Elena | Diego | Diego |

Both plans cover the four shifts without double-booking. Baseline returns `[0, 1, 2, 3]` with hard/soft scores 0/0 and 12 combinations checked. After the callout it returns `[0, 2, 4, 3]`, scores 0/−8, and 4 combinations checked. The −8 comes from missing two preferences, 5 for Ben on North floor and 3 for Clara on South register. It is not a hard-rule violation.

### Run both versions

Save the [baseline problem](baseline-problem.json) and [adjusted problem](adjusted-problem.json):

```sh
orchestrata-cli baseline-problem.json
orchestrata-cli adjusted-problem.json
```

Compare the [baseline output](baseline-expected.json) and [adjusted output](adjusted-expected.json). You can also run each problem on the [local API](../../docs/api/).

### Where this example stops

The four shifts already exist and all overlap. There is no demand forecast, calendar sync, payroll calculation, overtime or rest-period model, labor compliance check, or multi-week roster.
