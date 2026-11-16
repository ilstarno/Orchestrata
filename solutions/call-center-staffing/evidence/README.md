# Call center coverage: six simultaneous queues

This sample sample covers six voice queues in one 30-minute window starting
at 10:00 in New York (UTC−05:00 on 16 November 2026). It screens 60 synthetic
operator profiles. IDs are invented; there are no names, emails, recordings,
customer records, or observed arrival statistics.

| File | Use |
| --- | --- |
| `raw-input.json` | The sample roster, timezone/offsets, shift and break windows, skill and language ratings, experience, voice/chat contact limits, queue requirements, and demand assumptions. |
| `problem.json` | Exact native V1 problem. Each queue is one variable; the allowed integers are zero-based positions in the raw operator array. |
| `expected.json` | Exact solution object from the native V1 result's `solution` field. |
| `enriched-result.json` | Per-queue coverage and selection audit: required and observed eligibility facts, eligible IDs, every excluded operator's reason codes, preferences, score, workload context, and exhaustive-search evidence. |

## Eligibility, objective, and proof

The Python normalizer validates the input shape, unique IDs, ratings, timestamps
and matching time-zone offsets, probability total, positive windows, and bounded
search budget. A shift must cover the whole queue window and no break may
overlap it. Windows are half-open; a break ending at the window start is
allowed. Operators also need a positive voice seat, at least the queue's
minimum skill and language ratings, and sufficient months in the role. Ratings
from 1 to 5 are invented ordinal levels, not certification or CEFR scores.
Missing ratings are treated as zero. Exclusion counts by reason may overlap
because one operator can fail several checks. Chat capacity is context only:
all six queues in this example are voice.

The normalizer turns qualification into each queue's native finite domain.
The exact solver sees only those domains, a hard `all_different` rule, and one
`prefer_value` rule per queue. Prefer the highest skill rating, then highest
language rating, then greater experience, then ascending operator ID.
Positive integer weights set the business priority of those preferences.
Assigning an otherwise eligible alternative subtracts that preference weight
from the soft score. The exact solver neither parses the raw roster nor
interprets its metadata.

The six domains contain 3, 2, 3, 3, 3 and 3 candidates, for 486 combinations.
Native exact search and an independent Python enumeration agree: 324
combinations satisfy the hard rules; all six queues are covered by distinct
eligible operators. OP-001 was preferred for both billing queues. Billing
escalations has weight 9 and keeps OP-001. Spanish-language billing takes
eligible OP-002 and incurs a 7-point penalty. The other preferences are met.
The reported score is hard 0 and soft −7. This proves optimality only for the
stated one-window model within the 100,000-combination evaluation budget.

## Demand calculations

Each queue has a typical scenario (70%) and a surge scenario (30%). These
probabilities are didactic assumptions, not estimates from calls. Arrivals
are contacts per window and handling time is seconds per contact. Scenario
workload equals arrivals multiplied by average handling seconds. Expected
workload is the probability-weighted sum; expected occupancy is expected
workload divided by 1,800 seconds. The planning target allocates at most 80%
of one person's window, or 1,440 handling seconds. The report gives the
probability of **strictly exceeding** that target under the two assumptions.

For example, billing escalations assumes 3 or 5 arrivals at 360 seconds per
contact. The workloads are 1,080 and 1,800 seconds, with an expected value of
1,296 seconds (72% occupancy). Only the surge case exceeds 1,440 seconds,
yielding 30% modeled risk. Four queues have nonzero modeled risk despite
everyone being assigned. This shows the distinction between covering a queue
and having enough capacity to meet demand. Workload and probability are
reported separately and do not change the solver objective, assign calls,
predict waiting times, or establish service-level achievement.

## Reproduce and check

Use Python 3.10 or later with the system's IANA time-zone data and an
Orchestrata native ABI V1 library from the standalone engine. From the engine
checkout, build the library with:

```sh
cargo build --locked --release --manifest-path backend/Cargo.toml -p orchestrata-ffi
```

Then from this sample directory, on Linux (substitute the native library file
for your platform):

```sh
python -m unittest -v
python reproduce.py --library /path/to/engine/backend/target/release/liborchestrata_ffi.so
```

The check rebuilds all four JSON documents in memory, calls the native
library, runs a separate exhaustive check, and compares the canonical output
with each saved file. Add `--write` only when intentionally refreshing the
sample. First checked against the [verified engine source](https://github.com/ilstarno/orchestrata-engine/tree/cddfdaee5b9ee9ff2577399726b95a12286e69c7).
