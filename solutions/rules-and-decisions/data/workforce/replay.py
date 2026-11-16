"""Reproduce the bounded, review-only Thanksgiving retail roster."""
import argparse
from collections import defaultdict
from datetime import datetime
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
WEEKLY_OT_MINUTES = 40*60
REST_MINUTES = 24*60


def tstamp(stamp):
    return datetime.fromisoformat(stamp)


def span(item):
    return tuple(tstamp(item[field]) for field in ("startLocal", "endLocal"))


def within(a, b):
    return b[0] <= a[0] < a[1] <= b[1]


def overlaps(a, b):
    return max(a[0], b[0]) < min(a[1], b[1])


def minutes(a, b):
    return int((b-a).total_seconds() // 60)


def worker_minutes(assigned):
    return sum(worked(shift) for shift in assigned)


def worked(shift):
    return minutes(*span(shift)) - sum(meal["endMinuteLocal"]-meal["startMinuteLocal"]
                                       for meal in shift["meals"] if not meal["paid"])


def day_minute(value):
    return value.hour*60+value.minute


def meal_requirements(shift):
    start, end = map(day_minute,span(shift))
    result = []
    if start <= 660 and end >= 840 and end-start > 360:
        result.append(("NY-162-NOON","noon",30,660,840))
    if start < 660 and end > 1140:
        result.append(("NY-162-EVENING","additional-evening",20,1020,1140))
    if 780 < start and end-start > 360:
        result.append(("NY-162-LATE","midpoint",45,(start+end)/2,(start+end)/2))
    return result


def shape_rules(shift, site):
    block = span(shift)
    reqs = meal_requirements(shift)
    rules = []
    if not any(within(block, span(window)) for window in site[(shift["siteId"],shift["serviceDate"])]):
        rules.append("CO-HOLIDAY-HOURS")
    start, end = map(day_minute, block)
    if minutes(*block) > 600 or start >= end:
        rules.append("CO-ONE-SHIFT")
    all_meals = sorted(shift["meals"],key=lambda m:m["startMinuteLocal"])
    limit = start
    for meal in all_meals:
        x, y = meal["startMinuteLocal"], meal["endMinuteLocal"]
        if not meal["reliefRef"] or not limit <= x < y <= end:
            rules.append("CO-MEAL-RELIEF")
        limit = y
    for identifier, label, minimum, first, last in reqs:
        def meal_fits(meal):
            x, y = meal["startMinuteLocal"], meal["endMinuteLocal"]
            return (meal["mealCode"] == label and y-x >= minimum
                    and (first <= x and y <= last if first != last else x <= first <= y))
        if not any(meal_fits(meal) for meal in all_meals):
            rules.append(identifier)
    return list(dict.fromkeys(rules))


def person_rules(shift, worker, board, disruptions):
    rules = []
    block = span(shift)
    if shift["requestedRole"] not in worker["roleCodes"]:
        rules.append("CO-QUALIFICATION")
    if shift["siteId"] not in worker["approvedSiteIds"]:
        rules.append("CO-SITE")
    if not worker["protectedRest"]["designationRecorded"] or overlaps(block,span(worker["protectedRest"]["window"])):
        rules.append("NY-161-REST")
    for leave in worker["approvedBlocks"]:
        if leave["approvalRecorded"] and overlaps(block,span(leave)):
            rules.append("NY-196B-APPROVED" if leave["category"]=="protected-paid" else "CO-APPROVED-PTO")
    if (not any(within(block,span(day)) for day in worker["availabilityByDate"])
            or any(overlaps(block,span(item)) for item in worker["otherUnavailability"])
            or any(worker["workerId"]==x["workerId"] and overlaps(block,span(x)) for x in disruptions)):
        rules.append("CO-AVAILABILITY")
    if any(row["serviceDate"]==shift["serviceDate"] for row in board[worker["workerId"]]):
        rules.append("CO-ONE-SHIFT")
    return list(dict.fromkeys(rules))


def score(plan, staff, board, fixed_ot, reference):
    held = sum(worker is None for worker in plan.values())
    extra = sum(max(0,worker_minutes(board[worker])+staff[worker]["assumedHoursOutsideFixture"]*60-WEEKLY_OT_MINUTES)
                for worker in board)-fixed_ot
    changes = sum(plan.get(shift)!=worker for shift,worker in reference.items() if worker is not None)
    tie = tuple(value or "~" for value in plan.values())
    return held, extra, changes, tie


def advance(inp, scenario, reference=None):
    reference=reference or {}
    staff = {person["workerId"]:person for person in inp["workerRoster"]}
    site = {(item["siteId"],day["serviceDate"]):day["openWindows"]
            for item in inp["companyCalendar"]["weeklySiteHours"] for day in item["hours"]}
    disruptions = inp["disruptionSnapshot"] if scenario=="disruption" else []
    stock, pending = inp["fixedShiftDemand"], inp["unresolvedShiftDemand"]
    board={person:[] for person in staff}
    for shift in stock:
        worker = shift["fixedWorkerId"]
        blocked = shape_rules(shift,site)+person_rules(shift,staff[worker],board,disruptions)
        if blocked:
            raise ValueError(f"Locked assignment needs supervisor review: {shift['shiftId']} {blocked}")
        board[worker].append(shift)
    fixed_ot = sum(max(0,worker_minutes(board[worker])+staff[worker]["assumedHoursOutsideFixture"]*60-WEEKLY_OT_MINUTES)
                   for worker in board)
    shapes={shift["shiftId"]:shape_rules(shift,site) for shift in pending}
    best_score=best=None
    choice={}
    branches=0

    def search(i):
        nonlocal best_score,best,branches
        if i==len(pending):
            branches+=1
            check=score(choice,staff,board,fixed_ot,reference)
            if best_score is None or check<best_score:
                best_score,best=check,choice.copy()
            return
        shift=pending[i]
        sid=shift["shiftId"]
        available=[worker for worker in shift["namedWorkerPool"] if not shapes[sid]
                   and not person_rules(shift,staff[worker],board,disruptions)]
        candidates=(available[:1] or [None]) if scenario=="baseline" else [*available,None]
        for worker in candidates:
            choice[sid]=worker
            if worker:
                board[worker].append(shift)
            search(i+1)
            if worker:
                board[worker].pop()
            del choice[sid]

    search(0)
    assert best is not None
    for shift in pending:
        if worker:=best[shift["shiftId"]]:
            board[worker].append(shift)
    result=[]
    for shift in [*stock,*pending]:
        sid=shift["shiftId"]
        worker=shift["fixedWorkerId"] or best.get(sid)
        chosen_meals=[key for key,*_ in meal_requirements(shift)]
        rejected=[]
        for alternative in shift["namedWorkerPool"]:
            if alternative==worker:
                continue
            others={**board,alternative:[item for item in board[alternative] if item["shiftId"]!=sid]}
            veto=list(dict.fromkeys(shapes.get(sid,[])+person_rules(shift,staff[alternative],others,disruptions)))
            overtime_if_picked=max(0,worker_minutes(others[alternative])+worked(shift)-WEEKLY_OT_MINUTES)
            rejected.append({"workerId":alternative,"blockingRules":veto,"projectedOvertimeMinutesIfAssigned":overtime_if_picked,
                             "status":"constraint_failed" if veto else "feasible_not_selected"})
        if worker:
            explanations=(["CO-LOCKED"] if shift["fixedWorkerId"] else
                          ["CO-QUALIFICATION","CO-SITE","CO-AVAILABILITY","NY-161-REST",
                           "CO-BASELINE-FIRST-FIT" if scenario=="baseline" else "CO-PLAN-OBJECTIVE"])
        else:
            explanations=list(dict.fromkeys(rule for item in rejected for rule in item["blockingRules"]))
            if any(item["status"]=="feasible_not_selected" for item in rejected):
                explanations.append("CO-PLAN-OBJECTIVE")
        result.append({"shiftId":sid,"serviceDate":shift["serviceDate"],"siteId":shift["siteId"],
                       "role":shift["requestedRole"],"startLocal":shift["startLocal"],"endLocal":shift["endLocal"],
                       "workerId":worker,"locked":bool(shift["fixedWorkerId"]),"status":"covered" if worker else "held",
                       "requiredMealRuleIds":chosen_meals,"mealPlan":shift["meals"],"workedMinutes":worked(shift) if worker else 0,
                       "reasonRuleIds":explanations,"alternativeDecisions":rejected})
    roster=[]
    for identifier,person in staff.items():
        rest=person["protectedRest"]["window"]
        total_worked=worker_minutes(board[identifier])+person["assumedHoursOutsideFixture"]*60
        paid_absence=sum(item["paidMinutes"] for item in person["approvedBlocks"] if item["approvalRecorded"])
        roster.append({"workerId":identifier,"workedMinutes":total_worked,"approvedPaidAbsenceMinutes":paid_absence,
                       "overtimeWorkedMinutes":max(0,total_worked-WEEKLY_OT_MINUTES),
                       "preservedDesignatedRestMinutes":minutes(*span(rest)),"restWindow":rest,
                       "scheduledShiftIds":[item["shiftId"] for item in board[identifier]]})
    metrics={"requested":len(result),"covered":len(result)-best_score[0],"held":best_score[0],
             "overtimeMinutes":sum(item["overtimeWorkedMinutes"] for item in roster),
             "incrementalOvertimeMinutes":best_score[1],"changedPriorAssignments":best_score[2],
             "confirmed":len(stock),"openCovered":len(pending)-best_score[0],
             "optimizerEvaluatedFeasibleLeaves":branches}
    return {"id":scenario,"metrics":metrics,"shifts":result,"workerTotals":roster},best


def evidence(inp, scenarios):
    recommended=scenarios[1]
    hold=next(x for x in recommended["shifts"] if x["shiftId"]=="O-10")
    source=next(item for item in inp["ruleRegister"] if item["ruleId"]=="NY-162-NOON")
    constrained=hold["alternativeDecisions"][0]
    clearance={"attemptedWorkerId":constrained["workerId"],
               "blockingRuleRegisterIds":constrained["blockingRules"],"actualDisposition":constrained["status"]}
    proof={"mealWindowStartMinuteLocal":660,"mealWindowEndMinuteLocal":840,"minimumEligibleMinutes":30,
           "requestPlan":hold["mealPlan"],"candidateReview":clearance}
    trace={"id":"rest-and-meal-review","caseDistrict":"Albany County","plannedScenario":{
        "id":"recommended","workday":{"serviceDate":hold["serviceDate"],"site":{"id":hold["siteId"],
        "coverage":{"demandId":hold["shiftId"],"decision":{"status":hold["status"],
        "veto":{"ruleId":source["ruleId"],"provenance":{"checkedOn":source["accessedOn"],
        "officialSource":source["source"],"application":{"scope":source["applicability"],
        "decisionEvidence":proof}}}}}}}}}
    return {"caseId":"ny-retail-holiday-staffing","scope":inp["scope"],"calendar":inp["companyCalendar"],
            "dateScope":{"fromLocal":inp["calendarWeekStartLocal"],"untilLocal":inp["calendarWeekEndLocal"],
                         "planningAsOfLocal":inp["asOfLocal"],"zone":inp["timeZone"]},
            "ruleRegister":inp["ruleRegister"],"rawTables":inp,"scenarioResults":scenarios,"decisionAudit":trace}


def build():
    inp=json.loads((HERE/"inputs.json").read_text(encoding="utf-8"))
    baseline,baseline_picks=advance(inp,"baseline")
    recommended,picks=advance(inp,"recommended",baseline_picks)
    disruption,_=advance(inp,"disruption",picks)
    scenarios=[baseline,recommended,disruption]
    visual={"timeZone":inp["timeZone"],"weekStart":"2026-11-22","weekEnd":"2026-11-28","holiday":"2026-11-26",
            "scenarios":[{"id":plan["id"],"metrics":plan["metrics"],"shifts":[
                {key:row[key] for key in ("shiftId","serviceDate","siteId","role","workerId","locked","status","reasonRuleIds")}
                for row in plan["shifts"] if not row["locked"]]} for plan in scenarios]}
    return {"visual.json":visual,"operational-case.json":evidence(inp,scenarios)}


if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write",action="store_true",help="regenerate public JSON")
    args=parser.parse_args()
    for name,content in build().items():
        target=HERE/name
        payload=json.dumps(content,indent=2,ensure_ascii=False)+"\n"
        if args.write:
            target.write_text(payload,encoding="utf-8")
        elif target.read_text(encoding="utf-8")!=payload:
            raise SystemExit(f"Replay differs: {name}")
    print("Verified NY retail staffing baseline, recommended plan, and disruption.")
