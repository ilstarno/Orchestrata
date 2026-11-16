# Hiring operations: availability rules and an employer notice checkpoint

This separate offline example shows what an interview coordinator can verify before making a schedule proposal. It does not evaluate applicants, reveal why staff are unavailable, determine leave eligibility or employer coverage, or book/send anything.

| Evidence | What this replay does | What stays with the responsible owner |
| --- | --- | --- |
| U.S. Thanksgiving, November 26, 2026 | Holds a New York interview because the **sample company** adopted that date as an office closure. An out-of-scope office is held for policy review, not subject to the New York rule. See the [OPM 2026 holiday schedule](https://www.opm.gov/policy-data-oversight/pay-leave/federal-holidays/) and [5 U.S.C. § 6103](https://uscode.house.gov/us/usc/t5/s6103/). | Verify the actual office calendar and agreements. The federal calendar is not itself the private employer's scheduling policy. [DOL holiday-pay guidance](https://www.dol.gov/general/topic/workhours/holidays) discusses separate pay considerations. |
| Staff availability | Reads UNAVAILABLE, an opaque window ID and UTC timestamps. A fresh window ending exactly when the proposed interview starts is not a conflict. A stale observation must be refreshed. The ten-minute limit is a company example, not a legal deadline. | Keep underlying HR reasons and documentation out of scheduling inputs. [29 CFR § 825.500(g)](https://www.ecfr.gov/current/title-29/subtitle-B/chapter-V/subchapter-C/part-825/subpart-E/section-825.500) provides federal confidentiality context for certain FMLA records, not an assumption about any staff slot. |
| Employer general notice | Routes an employer-level checkpoint to HR. Unknown coverage, missing notice evidence and a supplied evidence record each produce distinct review actions. | Verify applicability, actual posting and any required distribution. [29 CFR § 825.300(a)](https://www.ecfr.gov/current/title-29/subtitle-B/chapter-V/subchapter-C/part-825/subpart-C/section-825.300) distinguishes covered employer posting from individual distribution when applicable. The [DOL fact sheet](https://www.dol.gov/agencies/whd/fact-sheets/28-fmla) distinguishes employer coverage from employee leave eligibility. |

In the eight records, one slot is proposed for coordinator review, two are held because of the adopted closure or staff availability, two require schedule input review, and three go to HR for notice review. These checks do not attempt a complete scheduling optimization, jurisdictional compliance determination, or pay calculation. The example does not model every state, local, occupation, union or employer requirement.

Every executed check records a rule ID, revision, verification date, scope, source IDs and the evidence actually used. The calendar lane and the employer notice lane are independent; an employer notice status does not rank, reject or affect a person or their appointment.

Run locally with Python 3.9+ and system time-zone data:

```sh
python3 -B replay.py
python3 -B verify.py
```

The case can be replayed without network access or personal data. Consult the source catalog for edition and scope per source. Running replay.py with --write regenerates only the deterministic decision trail from the checked-in inputs.
