# Call center coverage across five branches

This sample models New York, London, Frankfurt, Tirana, and Rome as five
independent operational branches. Each branch has 100 sample operator
profiles, spread evenly across billing and payments, account access,
technical support, and customer resolutions. The cities set branch
context, local time zones, and sample service languages; the data does not
describe a real employer, operator, or caller.

## What runs

Each branch has three independent 30-minute local windows, each evaluated
as four separate alternatives: baseline, an overlapping short break, a
same-department-only rule, and a surge-priority scenario. Eight coverage
positions are solved together per run: one voice queue and one chat queue
per department. Each position needs one qualified operator; operators
cannot take two positions in a single run. Middle-window approved
cross-trained operators can be considered for a neighboring department
except in the same-department-only variant. Models never borrow a worker
from another branch or coordinate assignments across periods.

Qualification checks that the shift covers the complete window, no break
overlaps it, a role capability is recorded, task and service-language
proficiency meet stated minimums, relevant role experience is sufficient,
and the declared channel capacity meets the position. Time periods are
half-open and have explicit offsets. All ranks, experience, qualifications,
capacity limits, and call volumes are sample assumptions. A language level
is modeled only when the queue requires that language, using a service
proficiency field, not nationality. The roster contains no personal
attributes, outcomes of employment assessments, or identity proxies.
For any real deployment, review job-related requirements, accessibility,
privacy, labor and local policy independently. This example does not
evaluate hiring, promotion, pay, or legal compliance.

The Python normalizer uses those checks to form integer domains, then
passes the **domains and rules only** to native ABI V1. Each exact request
uses an `all_different` constraint for distinct simultaneous assignments
and a weighted preferred choice for each position. Ranking among
qualified candidates uses task proficiency, language proficiency, relevant
experience, then stable operator ID. Higher preference weights indicate
higher cost of assigning another eligible operator, not higher hard
eligibility. Scores do not measure operator performance.

## Evidence

- 500 profiles in 20 departments at 5 branches. Each branch is screened
  against its own roster for each window.
- 60 independent native solves, all verified against a separate exhaustive
  enumerator. 94,990 candidates evaluated (64 to 6,561 per run), including
  17,940 hard-feasible candidates.
- All 480 positions covered: 240 voice and 240 chat. There are 50 distinct
  selections when scoped by branch, 173 distinct operators assigned across
  the examples, and 23 selected cross-department positions where permitted.
- 220 preferred choices met and 260 missed due to competing assignments.
  Every reported optimum applies only within its bounded case.

Demand calculations show the weighted expected contact-seconds from stated
typical and surge counts and handling times. For voice, the report also
shows the probability, under these two assumptions, of exceeding 80% of
one operator's 30-minute window in handling workload. This is an
illustrative planning target, not a universal benchmark. Chat records
contain contact-seconds but no claimed operator occupancy: handling
conversations concurrently makes such a conversion inappropriate without
a separate model. Demands, sample probabilities and occupancy targets are
reporting context only. This example does not forecast real traffic,
estimate wait times, route contacts, or optimize a whole network.

## Artifacts and reproducibility

| File | Content |
| --- | --- |
| `roster.json` | Complete sample input for 500 operator profiles and their branches. |
| `raw-scenarios.ndjson.gz` | 60 local scenario windows, capacities required, task/language prerequisites, variants and workload assumptions. |
| `normalized-problems.ndjson.gz` | 60 native input problems and index-to-operator mappings. |
| `verified-output.ndjson.gz` | 60 scores, eligibility evidence, exclusion reason totals, proof and per-position decisions. |
| `assignments.ndjson.gz` | 480 flattened coverage decisions, one per position per scenario. |
| `summary.json` | Counts and exact download/uncompressed SHA-256 hashes. |
| `sources.json` | Primary operational and review references, checked 16 November 2026. |

Join line-delimited records by `scenario_id`. Build the engine's native
`orchestrata-ffi` library for your platform, then run:

```sh
python -B -m unittest -v
python -B build.py --library /path/to/liborchestrata_ffi.so
```

No extra Python packages are needed; Python 3.10+ and system IANA time zone
data are required. The build recomputes the roster and calls native exact
search for every case, runs the independent oracle, and compares canonical
artifacts on the same compression implementation. `--write` intentionally
rebuilds the common files. The SHA-256 hashes of the uncompressed streams
can be compared even across gzip implementations. The native revision
first used for verification was `cddfdaee5b9ee9ff2577399726b95a12286e69c7`.

The [workload forecast reference](https://docs.aws.amazon.com/en_en/connect/latest/adminguide/forecasting.html),
[occupancy metric definition](https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html),
[channel concurrency guide](https://docs.aws.amazon.com/connect/latest/adminguide/channels-and-concurrency.html)
and [skill-matching guide](https://learn.microsoft.com/en-us/dynamics365/customer-service/administer/set-up-skill-based-routing)
inform the model distinctions, not its numerical assumptions. The
[EEOC guidance](https://www.eeoc.gov/laws/guidance/employment-tests-and-selection-procedures)
provides background on job-related assessment for teams considering real
data; it is not a determination for this sample or any jurisdiction.
