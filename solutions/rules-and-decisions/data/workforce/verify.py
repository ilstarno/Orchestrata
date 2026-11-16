"""Independently audit inputs, assignments and finite search results."""
from collections import Counter, defaultdict
from datetime import datetime
from itertools import product
import json
from pathlib import Path
from zoneinfo import ZoneInfo

HERE=Path(__file__).resolve().parent
OT=2400


def read(filename):
    return json.loads((HERE/filename).read_text(encoding="utf-8"))


def period(row):
    return datetime.fromisoformat(row["startLocal"]),datetime.fromisoformat(row["endLocal"])


def contained(a,b):
    return b[0] <= a[0] < a[1] <= b[1]


def intersect(a,b):
    return not (a[1] <= b[0] or b[1] <= a[0])


def duration(times):
    return int((times[1]-times[0]).total_seconds()/60)


def within_workdate(times, date):
    return all(str(t.date())==date for t in times)


def meal_details(shift):
    times=period(shift)
    local=lambda value:value.hour*60+value.minute
    start, stop=map(local,times)
    if not within_workdate(times,shift["serviceDate"]) or not 0<duration(times)<=600:
        return False,0,set()
    lunch = start <= 11*60 and stop >= 14*60 and stop-start>360
    dinner = start < 11*60 and stop > 19*60
    late = start > 13*60 and stop-start>360
    categories={}
    duty_free=0
    previous_stop=start
    for relief in sorted(shift["meals"],key=lambda row:row["startMinuteLocal"]):
        left,right=relief["startMinuteLocal"],relief["endMinuteLocal"]
        if not relief["reliefRef"] or not previous_stop<=left<right<=stop:
            return False,0,set()
        previous_stop=right
        categories.setdefault(relief["mealCode"],[]).append((left,right))
        if not relief["paid"]:
            duty_free+=right-left
    checks=[
        (lunch,"NY-162-NOON","noon",lambda a,b: 660<=a<=b-30<=810),
        (dinner,"NY-162-EVENING","additional-evening",lambda a,b: 1020<=a<=b-20<=1120),
        (late,"NY-162-LATE","midpoint",lambda a,b: 2*a<=start+stop<=2*b and b-a>=45),
    ]
    valid = all(not applies or any(match(*item) for item in categories.get(kind,[]))
                for applies,_,kind,match in checks)
    return valid,(stop-start)-duty_free,{key for applies,key,_,_ in checks if applies}


def candidates(shift,people,hours,incident):
    bounds=period(shift)
    if not meal_details(shift)[0] or not any(contained(bounds,period(opening))
                                           for opening in hours[(shift["siteId"],shift["serviceDate"])]):
        return []
    choices=[]
    identifiers=([shift["fixedWorkerId"]] if shift["fixedWorkerId"] else shift["namedWorkerPool"])
    for identifier in identifiers:
        worker=people[identifier]
        busy=[worker["protectedRest"]["window"],*worker["otherUnavailability"],
              *(row for row in worker["approvedBlocks"] if row["approvalRecorded"]),
              *(row for row in incident if row["workerId"]==identifier)]
        if (shift["requestedRole"] in worker["roleCodes"] and shift["siteId"] in worker["approvedSiteIds"]
            and any(contained(bounds,period(a)) for a in worker["availabilityByDate"])
            and all(not intersect(bounds,period(a)) for a in busy)):
            choices.append(identifier)
    return choices


def verify():
    inp,case,visual=map(read,["inputs.json","operational-case.json","visual.json"])
    assert inp==case["rawTables"]
    tz=ZoneInfo(inp["timeZone"])
    requested={row["shiftId"]:row for row in [*inp["fixedShiftDemand"],*inp["unresolvedShiftDemand"]]}
    pending=inp["unresolvedShiftDemand"]
    workers={row["workerId"]:row for row in inp["workerRoster"]}
    rules={row["ruleId"] for row in inp["ruleRegister"]}
    week=(datetime.fromisoformat(inp["calendarWeekStartLocal"]),datetime.fromisoformat(inp["calendarWeekEndLocal"]))
    assert duration(week)==7*24*60
    assert week==tuple(datetime.fromisoformat(inp[k]) for k in ("fixedEmployerWorkweekStartLocal","fixedEmployerWorkweekEndLocal"))
    by_site={(site["siteId"],day["serviceDate"]):day["openWindows"] for site in inp["companyCalendar"]["weeklySiteHours"] for day in site["hours"]}
    for staff in workers.values():
        rest=period(staff["protectedRest"]["window"])
        assert duration(rest)==1440 and contained(rest,week) and staff["protectedRest"]["designationRecorded"]
    for shift in requested.values():
        moments=period(shift)
        assert contained(moments,week)
        assert all(moment.utcoffset()==moment.astimezone(tz).utcoffset() for moment in moments)
    ref={}
    assert len(case["scenarioResults"])==len(visual["scenarios"])==3
    for evidence, view in zip(case["scenarioResults"],visual["scenarios"],strict=True):
        scenario=evidence["id"]
        assert scenario==view["id"]
        assert view["metrics"]==evidence["metrics"]
        incident=inp["disruptionSnapshot"] if scenario=="disruption" else []
        possible={key:candidates(shift,workers,by_site,incident) for key,shift in requested.items()}
        preoccupied=set()
        fixed_worked=Counter({person:staff["assumedHoursOutsideFixture"]*60 for person,staff in workers.items()})
        for row in inp["fixedShiftDemand"]:
            person=row["fixedWorkerId"]
            fixed_worked[person]+=meal_details(row)[1]
            assert possible[row["shiftId"]]==[person]
            pair=(person,row["serviceDate"])
            assert pair not in preoccupied
            preoccupied.add(pair)

        def evaluate(grid):
            occupied=set(preoccupied)
            totals=Counter(fixed_worked)
            for shift,person in zip(pending,grid,strict=True):
                if person is None:
                    continue
                pair=(person,shift["serviceDate"])
                if pair in occupied:
                    return None
                occupied.add(pair)
                totals[person]+=meal_details(shift)[1]
            moved=sum(ref.get(shift["shiftId"]) is not None and ref[shift["shiftId"]]!=person
                      for shift,person in zip(pending,grid,strict=True))
            rank=(grid.count(None),sum(max(0,count-OT) for count in totals.values()),moved,tuple(k or "~" for k in grid))
            return rank,totals

        rows={row["shiftId"]:row for row in evidence["shifts"]}
        assert len(evidence["shifts"])==len(rows)==len(requested)
        assert rows.keys()==requested.keys()
        for key,record in rows.items():
            shift=requested[key]
            assert set(record["reasonRuleIds"]) <= rules
            assert record["locked"] == bool(shift["fixedWorkerId"])
            assert record["mealPlan"]==shift["meals"]
            pass_meal,work_min,required=meal_details(shift)
            if record["workerId"]:
                assert record["status"]=="covered"
                assert record["workerId"] in possible[key] and pass_meal
                assert record["workedMinutes"]==work_min
                assert set(record["requiredMealRuleIds"])==required
                if record["locked"]:
                    assert record["workerId"]==shift["fixedWorkerId"]
            else:
                assert record["status"]=="held" and record["workedMinutes"]==0
            assert all(set(alt["blockingRules"])<=rules for alt in record["alternativeDecisions"])
        for row in view["shifts"]:
            assert row=={key:rows[row["shiftId"]][key] for key in row}
        chosen=tuple(rows[slot["shiftId"]]["workerId"] for slot in pending)
        actual,totals=evaluate(chosen)
        if scenario=="baseline":
            occupied=set(preoccupied)
            greedy=[]
            for row in pending:
                person=next((item for item in possible[row["shiftId"]] if (item,row["serviceDate"]) not in occupied),None)
                if person:
                    occupied.add((person,row["serviceDate"]))
                greedy.append(person)
            assert chosen==tuple(greedy)
        else:
            enumerated=[res[0] for grid in product(*[(*possible[shift["shiftId"]],None) for shift in pending])
                        if (res:=evaluate(grid)) is not None]
            assert actual==min(enumerated)
            assert len(enumerated)==view["metrics"]["optimizerEvaluatedFeasibleLeaves"]
        tallies={entry["workerId"]:entry for entry in evidence["workerTotals"]}
        assert tallies.keys()==workers.keys()
        for worker, tally in tallies.items():
            assert tally["workedMinutes"]==totals[worker]
            assert tally["overtimeWorkedMinutes"]==max(0,totals[worker]-OT)
            assert tally["approvedPaidAbsenceMinutes"]==sum(entry["paidMinutes"] for entry in workers[worker]["approvedBlocks"] if entry["approvalRecorded"])
            assert tally["preservedDesignatedRestMinutes"]==1440
        metrics=view["metrics"]
        assert metrics["requested"]==len(requested) and metrics["covered"]+metrics["held"]==len(requested)
        assert (metrics["held"],metrics["overtimeMinutes"],metrics["changedPriorAssignments"])==actual[:3]
        assert metrics["overtimeMinutes"]-sum(max(0,n-OT) for n in fixed_worked.values())==metrics["incrementalOvertimeMinutes"]
        ref={row["shiftId"]:choice for row,choice in zip(pending,chosen,strict=True)}


if __name__=="__main__":
    verify()
    print("Independent audit verified scenario scope, known commitments, frozen time windows and exact model optimum.")
