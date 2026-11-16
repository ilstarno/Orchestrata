"""Independent audit of the checked-in case and its hard scheduling constraints."""
from collections import Counter, defaultdict
from datetime import datetime, timedelta
from itertools import combinations, product
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent


def read(name):
    return json.loads((HERE / name).read_text(encoding="utf-8"))


def time(value):
    result = datetime.fromisoformat(value)
    assert result.utcoffset() == timedelta(0), value
    return result


def instant(row, key):
    return time(row[key])


def in_windows(start, end, spans):
    return any(instant(span, "fromUtc") <= start and end <= instant(span, "untilUtc") for span in spans)


def clear(start, end, spans):
    return all(end <= instant(span, "fromUtc") or instant(span, "untilUtc") <= start for span in spans)


def audit(case, visual):
    plans = visual["plans"]
    assert case["comparison"] == plans
    assert [plan["id"] for plan in plans] == ["baseline", "recommended", "disruption"]
    assert visual["timeZone"] == case["calendar"]["timeZone"] == "UTC"
    requests = {row["requestId"]: row for row in case["interviewRequests"]}
    staff = {row["staffResourceId"]: row for row in case["calendar"]["staffResources"]}
    event = case["whatIfEvent"]
    policy = case["planningPolicy"]
    buffer = timedelta(minutes=policy["minimumStaffBufferMinutes"])
    prerequisites = timedelta(minutes=policy["debriefDelayAfterPanelMinutes"])
    assert case["scope"]["candidateEvaluationPerformed"] is False
    assert case["scope"]["privateRecordsIncluded"] is False
    assert case["scope"]["invitesSent"] is False
    assert policy["calendarWritesEnabled"] is False
    last = {}
    for plan in plans:
        rows = {row["requestId"]: row for row in plan["rows"]}
        assert len(rows) == len(plan["rows"]) == len(requests)
        assert rows.keys() == requests.keys()
        commitments = defaultdict(list)
        actual = {}
        forced_holds = set()
        for key, row in rows.items():
            request = requests[key]
            roles = request["requiredRoles"]
            lock = request["confirmation"]
            windows = request["attendeeWindows"]
            takes_part = request["attendeeParticipates"]
            duration = timedelta(minutes=request["durationMinutes"])
            assert row["stage"] == request["stageCode"]
            assert row["locked"] == lock["locked"]
            available = []
            for offered in request["offeredStartsUtc"]:
                start = time(offered)
                end = start + duration
                if takes_part and not in_windows(start, end, windows):
                    continue
                panels = [tuple(person_id for person_id, person in staff.items()
                                    if person["panelRole"] == role
                                    and in_windows(start, end, person["availableWindows"])
                                    and clear(start, end, person["calendarBusyWindows"])
                                    and (plan["id"] != "disruption" or person_id != event["staffResourceId"]
                                         or clear(start, end, [event])))
                          for role in roles]
                available.extend((start, team) for team in product(*panels) if len(team) == len(set(team)))
            dependency = request["requiresCompletedScheduleStage"]
            if not available or dependency in forced_holds:
                forced_holds.add(key)
            if row["status"] == "held":
                assert not lock["locked"], key
                assert row["startUtc"] is row["endUtc"] is None and not row["panelists"], key
                assert row["reason"] in {"no-overlap", "no-panel", "schedule-conflict", "prerequisite"}, key
                continue
            assert row["status"] == "scheduled" and row["reason"] is None, key
            start, end = instant(row, "startUtc"), instant(row, "endUtc")
            team = tuple(row["panelists"])
            assert end - start == duration, key
            assert (start, team) in available, key
            assert instant(case["calendar"], "horizonStartUtc") <= start < end <= instant(case["calendar"], "horizonEndUtc"), key
            if dependency:
                assert rows[dependency]["status"] == "scheduled", key
                assert instant(rows[dependency], "endUtc") + prerequisites <= start, key
            if lock["locked"]:
                assert row["startUtc"] == lock["startUtc"] and list(team) == lock["staffResourceIds"], key
            for person in team:
                commitments[person].append((start, end))
            actual[key] = (start, team)
        for person, times in commitments.items():
            counts = Counter(start.date() for start, _ in times)
            assert max(counts.values()) <= policy["maximumInterviewsPerStaffDay"], person
            for (start, end), (other, stop) in combinations(times, 2):
                assert end + buffer <= other or stop + buffer <= start, person
        assert plan["scheduled"] == len(actual)
        assert plan["held"] == len(rows) - len(actual)
        assert plan["changed"] == sum(key in last and last[key] != choice for key, choice in actual.items())
        if plan["id"] != "baseline":
            # Independent coverage bound from individual calendar impossibility,
            # propagating through dependents, even before shared resources enter.
            assert plan["held"] == len(forced_holds)
        last = actual


if __name__ == "__main__":
    audit(read("operational-case.json"), read("visual.json"))
    print("Independent audit verified coverage bounds, immutable commitments, panel roles, windows, buffers and changes.")
