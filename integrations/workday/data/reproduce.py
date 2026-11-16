"""Build and replay a synthetic, vendor-neutral workforce roster."""
from collections import Counter, defaultdict
from datetime import date, datetime, time, timedelta
from json import dumps, loads
from pathlib import Path

ROOT = Path(__file__).resolve().parent
DAYS = [(date(2026, 11, 16) + timedelta(days=i)).isoformat() for i in range(5)]
SHIFT_HOURS = {"early": ["06:00", "14:00"], "late": ["14:00", "22:00"]}
ROLES = {
    "lead": ("handover", "schedule:lead"),
    "operator": ("dispatch", "schedule:dispatch"),
    "security": ("controlled-goods", "schedule:controlled"),
}


def build_input():
    workers, coverage = [], []
    groups = [("NW", "early"), ("NW", "late"), ("SE", "early"), ("SE", "late")]
    for group, (site, shift) in enumerate(groups):
        for index in range(12 + group % 2):
            role = "lead" if index < 3 else "operator"
            roles = [role]
            if (group, index) in {(0, 3), (2, 3)}:
                roles += ["security"]
            other_site = "NW" if site == "SE" else "SE"
            workers.append({
                "id": f"W{len(workers) + 1:03}",
                "homeLocation": site,
                "authorizedLocations": [site, other_site] if index in {5, 9} else [site],
                "skills": [ROLES[r][0] for r in roles],
                "roleGrants": [ROLES[r][1] for r in roles],
                "availability": [
                    {"date": day, "start": SHIFT_HOURS[shift][0], "end": SHIFT_HOURS[shift][1]}
                    for d, day in enumerate(DAYS) if d != (index + group) % 7
                ],
                "weeklyTargetHours": 24 if index < 5 else 16,
                "weeklyCapHours": 40 if index < 8 else 24,
                "preferredShift": shift,
            })
    for day in DAYS:
        for site, shift in groups:
            for role, count in [("lead", 1), ("operator", 2)]:
                for number in range(1, count + 1):
                    coverage.append(seat(day, site, shift, role, number))
    coverage += [seat(DAYS[0], site, "early", "security", 1) for site in ("NW", "SE")]
    assert len(workers) == 50
    return {
        "metadata": {
            "title":"Warehouse staffing across two sites", "schemaVersion": 1,
            "weekStarting": DAYS[0], "timeZone": "Europe/London",
            "provenance":"Sample data. No Workday tenant, API, or worker records were used.",
        },
        "locations":[{"id":"NW","name":"North Warehouse"},{"id":"SE","name":"South Exchange"}],
        "policy":{
            "minimumRestHours": 12, "maxShiftsPerDate": 1, "defaultWeeklyCapHours": 40,
            "paidHoursPerShift": 8, "publishedMinimumCoveragePerSiteShift": 3,
            "editingGrant": "roster:publish", "planningGrant": "roster:propose",
            "notes":"Example site policy, not employment law. Meals and intrashift relief are scheduled separately and not validated here.",
        },
        "workers":workers, "coverage":coverage,
        "scenarios":[
            {"id":"baseline", "label":"Staff both sites", "absences":[], "extraCoverage":[], "actorGrants":["roster:propose"]},
            {"id":"absence-and-demand", "label":"Specialist absent; late demand climbs",
             "absences":[{"workerId":"W029","date":DAYS[0]}],
             "extraCoverage":[seat(DAYS[0], "NW", "late", "operator", 3), seat(DAYS[0], "SE", "late", "security", 1)],
             "actorGrants":["roster:propose"]},
        ],
    }


def seat(day, site, shift, role, number):
    skill, grant = ROLES[role]
    start,end = SHIFT_HOURS[shift]
    return {"id":f"{day}/{site}/{shift}/{role}/{number}", "date":day,"locationId":site,
            "shift":shift,"start":start,"end":end,"role":role,"requiredSkill":skill,"requiredGrant":grant}


def instant(day, clock):
    return datetime.combine(date.fromisoformat(day), time.fromisoformat(clock))


def solve(data, scenario, previous):
    policy = data["policy"]
    if policy["planningGrant"] not in scenario["actorGrants"]:
        raise PermissionError("Roster planning grant required")
    hours, last_end, worked = Counter(), {}, set()
    unavailable = {(r["workerId"],r["date"]) for r in scenario["absences"]}
    rows, holds = [], []
    for s in sorted(data["coverage"]+scenario["extraCoverage"],
                    key=lambda r:(r["date"],r["start"], r["role"] != "security", r["id"])):
        start, end = instant(s["date"], s["start"]), instant(s["date"], s["end"])
        dur = (end-start).seconds//3600
        eligible, rejected = [], Counter()
        for w in data["workers"]:
            key=w["id"]
            if s["locationId"] not in w["authorizedLocations"]: rejected["location"]+=1
            elif s["requiredSkill"] not in w["skills"]: rejected["skill"]+=1
            elif s["requiredGrant"] not in w["roleGrants"]: rejected["role-grant"]+=1
            elif (key,s["date"]) in unavailable: rejected["absence"]+=1
            elif not any(a["date"]==s["date"] and a["start"]<=s["start"] and a["end"]>=s["end"] for a in w["availability"]): rejected["availability"]+=1
            elif (key,s["date"]) in worked: rejected["daily-shift-cap"]+=1
            elif hours[key]+dur>min(w["weeklyCapHours"],policy["defaultWeeklyCapHours"]): rejected["weekly-hours-cap"]+=1
            elif key in last_end and start-last_end[key]<timedelta(hours=policy["minimumRestHours"]): rejected["minimum-rest"]+=1
            else: eligible.append(w)
        row = dict(s, eligibleWorkers=len(eligible))
        if not eligible:
            reason = "specialist-unavailable" if s["role"]=="security" else "no-eligible-worker"
            row.update(workerId=None,status="hold",reason=reason)
            holds.append({"type":"coverage","id":s["id"],"reason":reason,"rejected":dict(rejected),
                          "nextAction":"Find an authorized specialist or move the controlled work window." if s["role"]=="security" else "Check relief coverage; do not publish an uncovered seat as staffed."})
        else:
            w=min(eligible,key=lambda w:(w["id"]!=previous.get(s["id"]), w["homeLocation"]!=s["locationId"],
                    w["preferredShift"]!=s["shift"], hours[w["id"]]/w["weeklyTargetHours"],w["id"]))
            key=w["id"]
            hours[key]+=dur
            last_end[key]=end
            worked.add((key,s["date"]))
            row.update(workerId=key,status="assigned",reason="reused" if previous.get(s["id"])==key else "eligible")
        rows.append(row)
    if policy["editingGrant"] not in scenario["actorGrants"]:
        holds.append({"type":"publish","id":"roster","reason":"publish-grant-required",
                      "nextAction":"Send to an authorized roster publisher. No source system was changed."})
    assigned=sum(r["status"]=="assigned" for r in rows)
    return {"id":scenario["id"],"label":scenario["label"],"coverage":rows,"holds":holds,
            "metrics":{"requiredSeats":len(rows),"assignedSeats":assigned,"unfilledSeats":len(rows)-assigned,
                       "paidHours":sum(hours.values()),"workersScheduled":len([h for h in hours.values() if h]),
                       "keptAssignments":sum(r["reason"]=="reused" for r in rows),"writebacks":0}}


def verify(data, scenario, output):
    # Independent checks join completed rows to input and reconstruct worker timelines.
    want = {r["id"]:r for r in data["coverage"]+scenario["extraCoverage"]}
    by_id = {w["id"]:w for w in data["workers"]}
    assert len({r["id"] for r in output["coverage"]}) == len(want) == len(output["coverage"])
    timelines = defaultdict(list)
    for r in output["coverage"]:
        s=want[r["id"]]
        assert all(r[key]==v for key,v in s.items())
        if r["status"]=="hold":
            assert not r["workerId"] and any(h["id"]==r["id"] for h in output["holds"])
            continue
        w=by_id[r["workerId"]]
        assert s["locationId"] in w["authorizedLocations"]
        assert s["requiredSkill"] in w["skills"] and s["requiredGrant"] in w["roleGrants"]
        assert {"workerId":w["id"],"date":s["date"]} not in scenario["absences"]
        a=next(a for a in w["availability"] if a["date"]==s["date"])
        assert a["start"]<=s["start"]<s["end"]<=a["end"]
        timelines[w["id"]].append((instant(s["date"],s["start"]),instant(s["date"],s["end"])))
    for worker,timeline in timelines.items():
        timeline.sort()
        assert len({start.date() for start,_ in timeline})==len(timeline)
        assert all(timeline[i+1][0]-timeline[i][1]>=timedelta(hours=data["policy"]["minimumRestHours"]) for i in range(len(timeline)-1))
        assert sum((end-start).total_seconds()/3600 for start,end in timeline)<=min(by_id[worker]["weeklyCapHours"],data["policy"]["defaultWeeklyCapHours"])
    assert output["metrics"]["assignedSeats"]==sum(map(len,timelines.values()))
    assert output["metrics"]["paidHours"]==sum((end-start).total_seconds()/3600 for t in timelines.values() for start,end in t)
    for r in output["coverage"]:
        if r["status"] != "hold":
            continue
        # For this fixture, every held role lacks a worker even before shift caps/rest.
        # This makes the stated exception independent of the algorithm's ordering.
        theoretically_eligible = [
            w for w in data["workers"]
            if r["locationId"] in w["authorizedLocations"]
            and r["requiredSkill"] in w["skills"] and r["requiredGrant"] in w["roleGrants"]
            and not any(a["workerId"]==w["id"] and a["date"]==r["date"] for a in scenario["absences"])
            and any(a["date"]==r["date"] and a["start"]<=r["start"] and a["end"]>=r["end"] for a in w["availability"])
        ]
        assert theoretically_eligible == [], r["id"]
    assert any(h["type"]=="publish" and h["reason"]=="publish-grant-required" for h in output["holds"])
    assert output["metrics"]["writebacks"]==0


def main():
    data=build_input()
    prior, scenarios={},[]
    for scenario in data["scenarios"]:
        result=solve(data,scenario,prior)
        verify(data,scenario,result)
        scenarios.append(result)
        prior={r["id"]:r["workerId"] for r in result["coverage"] if r["workerId"]}
        print(result["id"], result["metrics"])
    assert [r["metrics"]["unfilledSeats"] for r in scenarios]==[0,2]
    files={
        "input.json":data,
        "result.json":{"metadata":{"schemaVersion":1,"algorithm":"deterministic eligible-first roster with published-assignment preference",
                   "optimizationClaim":"Feasibility checks only; no claim of optimality or vendor/engine execution."},"scenarios":scenarios},
    }
    for name,obj in files.items():
        payload=dumps(obj,ensure_ascii=False,indent=2)+"\n"
        if "--check" in __import__("sys").argv:
            assert loads((ROOT/name).read_text())==obj,name
        else:
            (ROOT/name).write_text(payload)


if __name__ == "__main__":
    main()
