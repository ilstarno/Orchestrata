"""Reproduce a bounded interview-operations schedule. Never send appointments."""
from argparse import ArgumentParser
from collections import Counter
from datetime import datetime, timedelta, timezone
from itertools import product
import json
from pathlib import Path

BASE = datetime(2026, 11, 17, tzinfo=timezone.utc)
BUFFER = 10
DAILY_CAP = 4
DEPENDENCY_GAP = 15
HERE = Path(__file__).resolve().parent


def utc(minutes):
    return (BASE + timedelta(minutes=minutes)).strftime("%Y-%m-%dT%H:%M:%SZ")


def slot(day, hour, minute=0):
    return 1440 * day + 60 * hour + minute


def windows(start_hour, end_hour):
    return [(slot(day, start_hour), slot(day, end_hour)) for day in range(3)]


def resource(identifier, role, zone, available, busy=()):
    return dict(id=identifier, role=role, zone=zone, available=available, busy=busy)


STAFF = [
    resource("REC-01", "recruiter", "Europe/London", windows(14, 17)),
    resource("REC-02", "recruiter", "America/New_York", windows(15, 18), [(slot(0, 15), slot(0, 15, 30))]),
    resource("ENG-01", "specialist", "Europe/Berlin", windows(14, 17)),
    resource("ENG-02", "specialist", "America/New_York", windows(15, 18), [(slot(1, 15), slot(1, 15, 30))]),
    resource("MGR-01", "manager", "Europe/London", windows(14, 17)),
    resource("MGR-02", "manager", "America/New_York", windows(15, 18), [(slot(2, 14), slot(2, 15, 30))]),
]
STAFF_BY_ID = {person["id"]: person for person in STAFF}
ROLES = {
    "screen": ("recruiter",),
    "panel": ("specialist", "manager"),
    "debrief": ("recruiter", "manager"),
}
DURATIONS = {"screen": 30, "panel": 45, "debrief": 30}
DISRUPTION = dict(person="ENG-01", start=slot(1, 14), end=slot(1, 17))


def request(num, stage, starts, available, *, prerequisite=None, locked=None):
    return dict(id=f"INT-{num:03}", stage=stage, starts=starts, available=available,
                prerequisite=prerequisite, locked=locked)


def daytime(day, low=14, high=17):
    return [(slot(day, low), slot(day, high))]


REQUESTS = [
    request(1, "screen", [slot(0, 14)], daytime(0), locked=(slot(0, 14), ("REC-01",))),
    request(2, "screen", [slot(0, 15), slot(0, 16)], daytime(0)),
    request(3, "screen", [slot(0, 15)], daytime(0)),
    request(4, "panel", [slot(1, 14), slot(1, 15, 30)], daytime(1)),
    request(5, "panel", [slot(1, 14)], daytime(1)),
    request(6, "panel", [slot(1, 16, 30), slot(2, 15, 30)], daytime(1, 16, 18)+daytime(2)),
    request(7, "panel", [slot(2, 14)], daytime(2), locked=(slot(2, 14), ("ENG-01", "MGR-01"))),
    request(8, "debrief", [slot(1, 15), slot(2, 15, 30), slot(2, 16, 30)], (), prerequisite="INT-005"),
    request(9, "screen", [slot(2, 14), slot(2, 15, 30)], daytime(2, 17, 18)),
    request(10, "panel", [slot(0, 17, 30)], daytime(0, 17, 18)),
    request(11, "screen", [slot(1, 14), slot(1, 15, 30)], daytime(1)),
    request(12, "debrief", [slot(2, 15, 30), slot(2, 16, 30)], (), prerequisite="INT-006"),
    request(13, "panel", [slot(2, 16)], daytime(2, 16, 18)),
    request(14, "screen", [slot(2, 16, 40)], daytime(2, 16, 18)),
]


def fits(start, end, spans):
    return any(low <= start < end <= high for low, high in spans)


def intersects(a, b, c, d, allowance=0):
    return a < d + allowance and c < b + allowance


def attending(req, start):
    return req["stage"] == "debrief" or fits(start, start + DURATIONS[req["stage"]], req["available"])


def pool(role, start, end, incident):
    for staff in STAFF:
        busy = list(staff["busy"])
        if incident and incident["person"] == staff["id"]:
            busy.append((incident["start"], incident["end"]))
        if (staff["role"] == role and fits(start, end, staff["available"])
                and all(not intersects(start, end, lo, hi) for lo, hi in busy)):
            yield staff["id"]


def candidates(req, incident):
    options = []
    for start in sorted(req["starts"]):
        if not attending(req, start):
            continue
        end = start + DURATIONS[req["stage"]]
        roles = ROLES[req["stage"]]
        options.extend((start, panel) for panel in product(*(pool(role, start, end, incident) for role in roles)))
    if req["locked"] is not None:
        if req["locked"] not in options:
            raise ValueError(f"Confirmed appointment conflicts with current calendars: {req['id']}")
        return [req["locked"]]
    return options


def feasible(req, choice, board, calendars):
    if choice is None:
        return req["locked"] is None
    start, panel = choice
    end = start + DURATIONS[req["stage"]]
    dependency = req["prerequisite"]
    if dependency:
        prior = board.get(dependency)
        if not prior or prior[0] + DURATIONS["panel"] + DEPENDENCY_GAP > start:
            return False
    for person in panel:
        commitments = calendars[person]
        if any(intersects(start, end, a, b, BUFFER) for a, b in commitments):
            return False
        if sum((a // 1440 == start // 1440) for a, _ in commitments) >= DAILY_CAP:
            return False
    return True


def changed(board, reference, *, include_holds=False):
    return sum(previous is not None
               and (include_holds or board.get(req) is not None)
               and board.get(req) != previous
               for req, previous in reference.items())


def objective(board, reference):
    held = sum(value is None for value in board.values())
    first_start = sum(value[0] for value in board.values() if value is not None)
    # In a dispute, comparing only anonymous IDs and resource IDs is reproducible;
    # it does not use or infer a candidate ranking.
    deterministic = tuple((10**8, ()) if not val else val for val in board.values())
    return held, changed(board, reference, include_holds=True), first_start, deterministic


def solve(kind, reference=None, incident=None):
    reference = reference or {}
    board = {}
    calendars = {person: [] for person in STAFF_BY_ID}
    option_sets = {req["id"]: candidates(req, incident) for req in REQUESTS}
    incumbent = None
    best = None
    fixed = [req for req in REQUESTS if req["locked"] is not None]
    for req in fixed:
        lock = req["locked"]
        if req["prerequisite"] or not feasible(req, lock, board, calendars):
            raise ValueError(f"Inconsistent confirmed appointment: {req['id']}")
        for person in lock[1]:
            calendars[person].append((lock[0], lock[0] + DURATIONS[req["stage"]]))

    def add(req, assignment):
        board[req["id"]] = assignment
        if assignment and req["locked"] is None:
            start, panel = assignment
            for person in panel:
                calendars[person].append((start, start + DURATIONS[req["stage"]]))

    def remove(req, assignment):
        if assignment and req["locked"] is None:
            for person in assignment[1]:
                calendars[person].pop()
        del board[req["id"]]

    def search(index, held):
        nonlocal incumbent, best
        if incumbent and held > incumbent[0]:
            return
        if index == len(REQUESTS):
            score = objective(board, reference)
            if incumbent is None or score < incumbent:
                incumbent, best = score, board.copy()
            return
        req = REQUESTS[index]
        choices = option_sets[req["id"]]
        if req["locked"] is not None:
            valid = choices
        else:
            valid = [choice for choice in choices if feasible(req, choice, board, calendars)]
        if kind == "baseline":
            valid = valid[:1] or [None]
        elif req["locked"] is None:
            valid.append(None)
        for choice in valid:
            add(req, choice)
            search(index + 1, held + (choice is None))
            remove(req, choice)

    search(0, 0)
    if best is None:
        raise ValueError("No feasible plan preserves confirmed appointments")
    return best, option_sets


def hold_reason(req, board, options):
    if req["prerequisite"]:
        prerequisite = board.get(req["prerequisite"])
        if not prerequisite or not any(prerequisite[0]+DURATIONS["panel"]+DEPENDENCY_GAP <= start
                                       for start, _ in options):
            return "prerequisite"
    if not any(attending(req, start) for start in req["starts"]):
        return "no-overlap"
    if not options:
        return "no-panel"
    return "schedule-conflict"


def display(kind, board, options, reference):
    rows = []
    for req in REQUESTS:
        identifier = req["id"]
        value = board[identifier]
        row = {"requestId": identifier, "stage": req["stage"], "locked": bool(req["locked"])}
        if not value:
            row.update(status="held", startUtc=None, endUtc=None, panelists=[],
                       reason=hold_reason(req, board, options[identifier]))
        else:
            start, people = value
            row.update(status="scheduled", startUtc=utc(start), endUtc=utc(start+DURATIONS[req["stage"]]),
                       panelists=list(people), reason=None)
        rows.append(row)
    counts = Counter(row["status"] for row in rows)
    return {"id": kind, "scheduled": counts["scheduled"], "held": counts["held"],
            "changed": changed(board, reference), "rows": rows}


def spans_to_json(spans):
    return [{"fromUtc": utc(first), "untilUtc": utc(last)} for first,last in spans]



def disruption_audit(incident, previous, replacement):
    review = {
        "coordinatorDecision": "review_required",
        "deliveryControls": {
            "writeCalendar": False,
            "notifyParticipants": False,
            "outcomeBoundary": {
                "decisionType": "meeting_logistics",
                "automatedHiringDecisionAllowed": False,
            },
        },
    }
    invalidated = {
        "removedStaffResourceId": incident["staffResourceId"],
        "remediation": {
            "plannerAction": "recommended_schedule_comparison",
            "proposedAppointment": replacement,
            "reviewGates": review,
        },
    }
    impact = {
        "affectedRequestId": previous["requestId"],
        "previousAppointment": {
            "previousStartUtc": previous["startUtc"],
            "previousPanel": previous["panelists"],
            "invalidatedCoverage": invalidated,
        },
    }
    return {
        "productionInputsUsed": False,
        "replanReview": {
            "scenario": "disruption",
            "calendarChange": {
                "eventType": "availability_removed",
                "affectedWindow": {**incident, "requestImpact": impact},
            },
        },
    }


def operational(visual):
    incident = {"staffResourceId": DISRUPTION["person"], **spans_to_json([(DISRUPTION["start"], DISRUPTION["end"])])[0],
                "incidentKind": "calendar_window_removed"}
    staff = [{"staffResourceId": person["id"], "panelRole": person["role"], "displayTimeZone": person["zone"],
              "availableWindows": spans_to_json(person["available"]), "calendarBusyWindows": spans_to_json(person["busy"]),
              "minimumGapMinutes": BUFFER, "dailyInterviewLimit": DAILY_CAP} for person in STAFF]
    requests = [{"requestId": req["id"], "stageCode": req["stage"], "requiredRoles": ROLES[req["stage"]],
                 "durationMinutes": DURATIONS[req["stage"]], "offeredStartsUtc": [utc(start) for start in req["starts"]],
                 "attendeeWindows": spans_to_json(req["available"]), "attendeeParticipates": req["stage"] != "debrief",
                 "requiresCompletedScheduleStage": req["prerequisite"],
                 "confirmation": {"locked": bool(req["locked"]), "startUtc": utc(req["locked"][0]) if req["locked"] else None,
                                  "staffResourceIds": req["locked"][1] if req["locked"] else []}} for req in REQUESTS]
    recommended, disrupted = visual["plans"][1:]
    incident_id = incident["staffResourceId"]
    previous = next(x for x in recommended["rows"] if incident_id in x["panelists"] and
                    incident["fromUtc"] <= x["startUtc"] < incident["untilUtc"])
    replacement = next(x for x in disrupted["rows"] if x["requestId"] == previous["requestId"])
    return {
        "caseId": "hiring-operations-interview-planner",
        "scope": {"workflow": "meeting_logistics_only", "manualReviewRequired": True, "invitesSent": False,
                  "candidateEvaluationPerformed": False, "privateRecordsIncluded": False},
        "calendar": {"horizonStartUtc": utc(0), "horizonEndUtc": utc(3*1440), "timeZone": "UTC",
                     "calendarWindowConvention": "half_open_intervals", "staffResources": staff},
        "interviewRequests": requests,
        "planningPolicy": {"minimumStaffBufferMinutes": BUFFER, "maximumInterviewsPerStaffDay": DAILY_CAP,
                           "debriefDelayAfterPanelMinutes": DEPENDENCY_GAP,
                           "confirmationLocksStartAndStaff": True,
                           "objectiveOrder": ["fewest_unscheduled", "fewest_changes_to_previous_schedule",
                                              "earliest_total_starts", "stable_technical_tie_break"],
                           "calendarWritesEnabled": False, "excludedData": ["application_content", "ratings", "identity_attributes"]},
        "whatIfEvent": incident,
        "comparison": visual["plans"],
        "scheduleAudit": disruption_audit(incident, previous, replacement),
    }


def build():
    baseline, base_options = solve("baseline")
    recommended, planned_options = solve("recommended", baseline)
    disruption, recovery_options = solve("disruption", recommended, DISRUPTION)
    visual = {"timeZone": "UTC", "plans": [
        display("baseline", baseline, base_options, {}),
        display("recommended", recommended, planned_options, baseline),
        display("disruption", disruption, recovery_options, recommended),
    ]}
    return {"visual.json": visual, "operational-case.json": operational(visual)}


if __name__ == "__main__":
    parser = ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true", help="regenerate local replay output")
    args = parser.parse_args()
    for name, payload in build().items():
        target = HERE / name
        output = json.dumps(payload, indent=2, ensure_ascii=False) + "\n"
        if args.write:
            target.write_text(output, encoding="utf-8")
        elif target.read_text(encoding="utf-8") != output:
            raise SystemExit(f"Replay differs: {name}")
    print("Verified baseline, recommended interview schedule, and disruption replay.")
