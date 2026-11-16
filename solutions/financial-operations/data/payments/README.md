# Payments operations: keep exception work inside the cutoff

A reconciliation check and a payment packet may reach the same desk with different deadlines. Four analysts share 24 internal work items. Some cases only need a reconciliation check. Others need packet preparation followed by an independent quality review. Those two stages need distinct people, and a partial plan does not count.

| Same work, different roster | Scheduled | Held | Sent to separate review |
| --- | ---: | ---: | ---: |
| Normal shift | 15 | 8 | 1 |
| Afternoon reviewer workstation outage | 13 | 10 | 1 |

The outage removes analyst AN-02 from 13:30 to 16:50 ET. PX-007 and PX-019 no longer fit. The scheduler can move work to qualified staff without treating a missing, rejected, or expired record as reviewable evidence. PX-016 is sent to a separate owner because the applicability of the consumer process is unclear; this example decides neither coverage nor rights.

## Where the times come from

- [Federal Reserve Financial Services: FedACH Processing Schedule](https://www.frbservices.org/resources/resource-centers/same-day-ach-resource-center/fedach-processing-schedule/), Electronic Return Items. The three sourced current-day transmission deadlines are 10:30, 14:45 and 16:45 ET. The public target distributions and settlement schedules are reference fields, not predictions or case instructions. The source is not an eligibility or authorization determination.
- [CFPB: Regulation E, § 1005.11](https://www.consumerfinance.gov/rules-policy/regulations/1005/11/) is context for a **separate** process when covered. Its timing, applicability, remedies, notice and investigation obligations are not computed here.

The earlier of the company's due time and 15 minutes before the sourced window is the internal work deadline. The 15-minute buffer and qualification, evidence and staff separation controls are company assumptions, not regulatory requirements. There is no permission to silently roll work to another cycle. The replay uses one weekday in the America/New_York time zone and does not implement a banking holiday calendar, network exception handling or settlement eligibility.

## Reproduce and inspect

Run `python3 -B replay.py` to regenerate the plans in memory, verify final assignments separately and compare the results byte for byte against `decision-traces.json`. Run `python3 -B -m unittest -v` for a fixed outcome comparison and negative verifier tests. The optional `--write-traces` updates the checked-in replay output. Python 3.9 or newer with standard time-zone data is sufficient.

The planner works through due times and chooses the earliest complete plan in five-minute increments for each case. It does not claim a globally optimal work plan. The replay checks that each scheduled assignment fits its work durations and due time, uses qualified analysts on their available shifts, avoids overlaps, follows preparation-before-review and uses separate people. The trace records hold reasons as well as analyst minutes. There are no account identifiers, payment amounts or customer records. No payment, return, credit, notice, consumer dispute or production integration is executed or decided.
