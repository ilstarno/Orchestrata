# Plan queues and staffing with your phone platform

A worked example demonstrates reporting inputs and planner recommendations. It does not send a call, change a queue, edit an agent, or publish a shift. Keep the phone platform in charge of live routing.

| Platform | Start with | Validate before a pilot |
| --- | --- | --- |
| Genesys Cloud CX | Queue activity and conversation aggregates, approved agent and skill visibility | Match queues to divisions; check the permissions that control analytics, routing and availability. Confirm workforce features for the subscribed edition. |
| RingCentral RingCX | Supervisory snapshots and historical reporting for RingCX; separate voice from digital | Confirm the account/subaccount, reporting entitlements and application scopes. The agent-availability endpoint requires `CX Routing`. RingEX has a different surface. |
| Asterisk PBX | AMI `QueueStatus` for a snapshot; `queue_log` for queue history | Confirm the Asterisk version and queue strategy. Match queue and member IDs to independently maintained skill and shift data. Queue members and penalties are not a workforce roster. |

## First decision

Compare demand against coverage in a window that matches your operation. This example uses a 30-minute window. Set hard rules for skills, language, shift and break, availability, and which branches may accept overflow. Assign work only when the operator is eligible. If no eligible operator can take it, leave the call with the platform’s existing waiting, overflow, or callback policy. A missing or stale state is not evidence of availability. Historical forecast inputs and current queue status are different records.

## Read-only trial

1. Agree on branch, service windows, queue IDs and time zones. Normalize timestamps without mixing event time and receipt time.
2. Use field-minimized exports or a narrowly scoped read-only account. Use stable call and operator references rather than phone numbers, recordings, or transcripts.
3. Check transferred, abandoned, answered and callback counts separately. Reconcile snapshots and historical reporting with the vendor’s counters before using a staffing estimate.
4. Show which rule a recommendation satisfied, which blocked the call, and what staffing change the supervisor can review. Fall back to native platform policy when state is stale or missing.
5. Compare coverage, wait, overflow, and unassigned demand with a baseline. Check results by branch, channel and queue. A separate, approved decision is required before changing any live routing or staffing policy.

## Reproduce the result

Download the [offline replay kit](contact-center-replay.tar.gz), extract it, and run `python3 verify.py` in its `contact-center-connectors` directory. It checks the three provider-labelled inputs against the archived staffing result. The [sample queue log and counter](README.md) are separate.

## Source references

- [Genesys Cloud CX Queues Activity Detail](https://help.genesys.cloud/articles/queues-activity-detail-view/)
- [Genesys Cloud CX workforce permissions](https://help.genesys.cloud/articles/workforce-management-permissions-overview/)
- [RingCX real-time supervisor reporting](https://developers.ringcentral.com/engage/voice/guide/analytics/reports/realtime-supervisor-view)
- [RingCX agent availability and scopes](https://ringcentral-ringcx-api-documentation.readthedocs-hosted.com/en/latest/integration/agent-avail/)
- [Asterisk AMI QueueStatus](https://docs.asterisk.org/Latest_API/API_Documentation/AMI_Actions/QueueStatus/)
- [Asterisk queue log events](https://docs.asterisk.org/Operation/Logging/Queue-Logs/)
