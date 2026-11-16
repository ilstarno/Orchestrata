# New York and London delivery access

Two routes can cover similar customer demand yet give different decisions. Step through a New York truck network connection, a delivery rider's prerequisites, and a London vehicle's timed access. Each record names the modeled rule, jurisdiction, asset mode, local clock, evidence and company policy. This is a replay of candidate segments, not a full-route solver or approval to dispatch.

| Worked decision | What matters |
| --- | --- |
| NYC-02 vs NYC-03 | Both serve a destination off the New York truck network. Only the first has a verified connection at the closest intersection. A light commercial van (NYC-09) is not treated as a truck under this modeled network rule; that does not exempt it from posted restrictions. |
| NYC-05 through NYC-07 | An adult employee doing commercial bicycle delivery needs confirmed course, equipment and ID checks. A missing course record is reviewed; failed brakes block the candidate. |
| LON-01 through LON-05 | For the specified Oxford Street section, the modeled loading vehicle must enter and exit before 07:00 local, use an approved access approach and actually load. Unknown approach evidence is not a pass. The ordinary cycle case cannot use the traffic-free section. |
| LON-06 through LON-08 | An ordinary pavement is not treated as a cycle track. A shared-use designation is used only with current route evidence. |
| NYC-08, LON-09 and LON-10 | Company controls are distinct from street rules: payload, confirmed receiving slots, chilled cargo checks and depot-return buffers can stop a candidate even when the selected legal condition passes. |

## Sources and limits

Authority context checked 16 November 2026. The observation date is when the reference was checked, not an enactment date. The Oxford access interval is applied as half-open in the example, so a 07:00 exit does not pass. Emergency exemptions and any other signed or permit-specific cases are outside the replay. Confirm current notices, exact approaches, curb signage, permits, and wider obligations before making real dispatch decisions.

- [NYC DOT Truck Route Network](https://nyc-business.nyc.gov/nycbusiness/description/truck-route-network)
- [NYC DOT delivery cycling and Administrative Code references](https://www.nyc.gov/html/dot/html/about/deliverybybike.shtml)
- [Transport for London: Oxford Street rules and exemptions](https://tfl.gov.uk/modes/driving/os-rules-and-exemptions)
- [UK Highway Code: rules 62–64](https://www.gov.uk/guidance/the-highway-code/rules-for-cyclists-59-to-82#rule64)

The fleet city maps elsewhere use independent solved datasets. No claim is made that those routes were verified against these restrictions, that the sample attestations come from the named authorities, or that a result covers all applicable laws. Company thresholds in the example are illustrative only. No customer addresses, driver names, account data, or actual permits are included.

## Replay

Run `python3 -B replay.py` to verify the 20 decisions against the reference, or add `--json` for the complete applied-rule trace. Run `python3 -B -m unittest -v` to test evidence failure, local UTC offsets, loading windows and mixed authority/company outcomes. Requires only Python 3.9+ with standard system timezone data. Statuses are `passes_modeled_checks`, `hold` and `review`. Every outcome remains non-authorizing.
