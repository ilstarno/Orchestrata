# Traffic planning: DNS and regional capacity

Answer two different questions: which eligible region could a new lookup be pointed to, and how much of the independently supplied request load could each region's backends handle? This worked example never sends a DNS query, inspects a running network, edits DNS, changes a load balancer, or contacts a provider. Its region and backend IDs are labels, not hosts.

## Replay the comparison

Requires Python 3.11 or later, without packages or credentials.

```sh
python3 -B replay.py
python3 -B verify.py
```

The first command compares the calculated output with both published JSON files. Add `--write` to regenerate a local copy. The second independently checks the per-market choices, freshness, capacity, reserves, proportionate rounding, and conservation of regional offered load.

| Scenario | DNS markets held for review | Backend demand (req/s) | Allocated (req/s) | Unserved in the model (req/s) |
| --- | ---: | ---: | ---: | ---: |
| Normal | 0 of 5 | 510 | 510 | 0 |
| Peak demand | 0 of 5 | 975 | 750 | 225 |
| Region unavailable | 1 of 5 | 650 | 435 | 215 |

At peak, EU-C's last healthy check has expired and AP-C has no observation; neither gets an allocation. When the EU edge goes down, the EU-only market has no allowed destination. The flexible EU market uses its permitted fallback. The AP pool still has 85 req/s of usable backend capacity for its own supplied demand, but its single healthy fault domain no longer meets the separate eligibility rule for new DNS lookups. Cached DNS answers and existing connections are not moved by this calculation.

## Rules

### New DNS lookups

- Each market has an explicit destination allowlist and ranked groups of approved destinations. "Market" is a policy label, not an inferred client location or nationality.
- A destination needs a healthy regional edge and usable healthy backends in at least two distinct fault domains. An observation over 30 seconds old is stale. Unknown, draining and down are not treated as healthy.
- Pick the first group with an eligible destination and scale its relative weights to 100 points. Integral remainders are assigned by largest fraction, then stable ID. If no group qualifies, leave all 100 points unassigned and flag the row for review. This is a policy proposal, not a DNS response code.
- The proposed record TTL is 60 seconds. It is a caching parameter, not an outage duration, guaranteed convergence time, or request-level distribution. DNS proposal weights apply to new answers at the authoritative policy layer, not existing cached answers or application requests.

### Regional backend allocation

- Receive each region's offered demand as a separate scenario input in requests per second. Do not multiply that number by DNS lookup weights or redistribute it to another region.
- Allocate to a fresh healthy backend only when its regional edge is healthy. Subtract the backend's required hard reserve from its rated capacity first. Never use the reserve; drain, down, missing or expired observations reduce usable capacity to zero.
- Allocate the smaller of the region's offered demand and eligible usable capacity. Split by usable capacity using integral req/s targets and largest-fraction remainder, with a stable ID tie break. Unserved demand is the arithmetic gap in this snapshot, not a measured error rate.
- These examples are single snapshots. They do not model time-to-drain, request duration, connection affinity, retries, cross-zone balancing, provider-specific fail-open behavior, or autoscaling. Test those policies in a staging system before proposing a live rollout.

## Files and standards

`operational-case.json` contains the observations, locality policies, explicit demand inputs, outputs and outage review trace. `visual.json` is the smaller payload behind the website example. Provider docs illustrate why deployment needs its own approved translation and validation: [DNS terminology and caching](https://www.rfc-editor.org/rfc/rfc9499.html), [Serving stale DNS data](https://www.rfc-editor.org/rfc/rfc8767.html), and [Route 53 weighted routing and health checks](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/routing-policy-weighted.html). This is not a Route 53 adapter. A provider's last-resort behavior may differ from the planner's explicit "hold for review."
