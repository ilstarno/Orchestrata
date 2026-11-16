# Property claims operations: desk and field scheduling

Schedule work a coordinator has already approved: prepare a customer contact or reserve a field inspection slot. The case never decides whether a claim is covered, what it is worth, whether to pay or deny it, or which customer is eligible for anything. It uses work, staff and service-zone codes instead of claim files, addresses or personal data.

| Variant | Proposed | Held | Needs review |
| --- | ---: | ---: | ---: |
| Intake-order baseline | 7 of 14 | 4 | 3 |
| Combined schedule | 9 of 14 | 2 | 3 |
| Surge adds seven work items | 12 of 21 | 5 | 4 |
| One field-staff window becomes unavailable | 12 of 21 | 5 | 4 |

The combined plan makes room for two additional tasks in the same normal workload. During the disruption, two proposed appointments exchange windows. All twelve previously proposed work items remain in the plan, ten stay in the same time and staff assignment, and both confirmed bookings remain unchanged.

Each work item can occupy one 60-minute window. Staff may only perform the approved duty in their approved service zone, and field work additionally needs confirmed site access and the shared remote support seat. The baseline proposes the first legal fit in intake order. The exact comparison maximizes booked work, then keeps as many already proposed work items as possible, then minimizes exact placement changes, then favors earlier windows. A configured company target is an internal cutoff, not a statutory deadline. Service zones are schematic. The fixture does not model travel, routes, duration uncertainty, legal exemptions, or payroll.

Every result records the governing company rule and revision, evidence-check date and next human action. Unconfirmed scope, pending site access, or an expired operational credential routes to review; a known scheduling bottleneck is held. Neither label is a claim disposition. No records are written to an external system and no appointment or communication is sent.

## Official rules, separate from modeled policy

- The [California Department of Insurance Fair Claims Settlement Practices Regulations](https://www.insurance.ca.gov/01-consumers/130-laws-regs-hearings/05-CCR/fair-claims-regs.cfm) lists distinct obligations for documentation, communications, training and property claims. An insurance compliance owner must verify the actual trigger, applicability, and deadline. This comparison neither sets nor proves compliance with any of them.
- The [CDI 2026 Guide for Adjusting Property Claims After a Major Disaster](https://www.insurance.ca.gov/0200-industry/0050-renew-license/0200-requirements/upload/2026-Guide-for-Adjusting-Property-Claims-in-California-After-a-Major-Disaster_Final.pdf), pages 6–8, distinguishes ordinary licensing from conditional registration, supervision and certification in Commissioner-declared emergencies. The [CDI licensing information](https://www.insurance.ca.gov/0200-industry/0050-renew-license/0200-requirements/insurance-adjuster.cfm) links those procedures. A spike in the queue does not turn those procedures on. The sample reads signed-off duty and zone scopes as inputs; it never looks up, issues, or interprets a license.

Run locally with Python 3.9 or later and no dependencies:

```sh
python3 -B replay.py
python3 -B verify.py
```

The first command rebuilds the comparison in memory and checks the checked-in case and presentation data. The second verifies recorded constraints and upper bounds, checks what changed, and verifies that a confirmed booking which becomes invalid requires coordinator review instead of silently moving.
