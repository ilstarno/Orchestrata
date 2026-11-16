# Hiring operations: interview coordination

Start with interviews already approved for scheduling. The solver assigns times and interviewers; it does not decide who gets an interview, advance applicants, assess people, or make hiring decisions. There are no names, contact details, application contents, scores, or staff absence reasons in this worked example. It uses local files and never opens a calendar, writes an event, or sends an invitation.

## Replay the three-day calendar

Use Python 3.11 or later. No packages, credentials, or service are needed.

```sh
python3 -B replay.py
python3 -B verify.py
```

The replay compares its result byte-for-byte with the checked-in JSON. To regenerate your local copy, use `python3 -B replay.py --write`. The verifier reads the two published JSON files independently, checks each scheduled appointment against the inputs, and establishes the best possible meeting coverage in the two recommended comparisons for this case. The demonstration does not establish large-calendar throughput, measured savings, or a production integration.

| Plan | Scheduled | Held for a coordinator | Existing appointments kept with a new time or panel |
| --- | ---: | ---: | ---: |
| First available slot in request order | 9/14 | 5 | — |
| Recommended | 12/14 | 2 | 2 |
| One specialist unavailable | 10/14 | 4 | 1 |

The recommended plan moves two appointments by 60 and 90 minutes respectively, which frees room for three requests the first-available plan could not place. When one specialist becomes unavailable, appointment INT-004 keeps its time and changes specialist. INT-005 goes to hold because its only offered time has no eligible specialist. Its dependent debrief INT-008 goes to hold as well. Neither previously confirmed appointment, INT-001 or INT-007, moves or changes staff.

## Rules checked

- Every start is picked from a finite list already offered to the person scheduling. A full external appointment must fit inside a shared attendee window; internal debriefs do not require external attendance.
- Screens use one recruiter; panels use one specialist and one manager; internal debriefs use a recruiter and a manager. All assigned staff must be available for the entire appointment and clear of calendar busy blocks.
- The solver avoids double-booking staff, enforces a ten-minute buffer between interviews for each interviewer, and accepts at most four interview appointments per staff member per UTC day.
- Debriefs follow their required panel by at least 15 minutes. Confirmed appointments lock both the start and the assigned staff. A conflict in the confirmed set stops the replay instead of overriding the confirmation.
- The recommended solve first minimizes holds. It then minimizes changes to appointments in the reference plan, including those now held, and then prefers earlier total start times. Stable roster and request identifiers resolve remaining ties; they are not signals about the people interviewing.
- The displayed change count is narrower: a recommendation counts only if an appointment remains scheduled but has a different time or panel. Newly scheduled and newly held rows are tracked in their outcomes, not counted as moves.

## What a hold means

| Code | Coordinator action |
| --- | --- |
| `no-overlap` | Ask for another permitted time window; the full appointment did not fit. |
| `no-panel` | Find an available person for a required panel role or present another time. |
| `schedule-conflict` | Review other appointments, shared interviewer capacity, and buffers. |
| `prerequisite` | Schedule the preceding panel before trying to place the internal debrief. |

No hold is an assessment of a person. Treat it as unresolved calendar work.

## Input and output

- `operational-case.json` includes request windows, roster time zones for display, free/busy boundaries, rule parameters, the availability disruption, three saved plans, and a review trace. Timestamps are unambiguous UTC; intersect them as half-open intervals. The example takes explicit windows, not local recurrence or daylight-saving conversion rules.
- `visual.json` contains the exact rows and counts shown in the walkthrough.
- Calendar implementations can map permitted availability from [Google Calendar free/busy](https://developers.google.com/workspace/calendar/api/v3/reference/freebusy/query), [Microsoft Graph getSchedule](https://learn.microsoft.com/en-us/graph/api/calendar-getschedule?view=graph-rest-1.0), or [iCalendar VFREEBUSY](https://www.rfc-editor.org/rfc/rfc5545#section-3.6.4). These are data format references, not connected integrations. A live deployment should query only approved calendars, discard event subjects and participant addresses when not required, treat missing availability as unknown, and recheck it just before a coordinator accepts a proposal.
