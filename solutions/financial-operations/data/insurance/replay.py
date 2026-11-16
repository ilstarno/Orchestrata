"""Deterministic, offline scheduling of approved insurance operations work items."""
from functools import lru_cache
import json
from pathlib import Path
import sys

ROOT = Path(__file__).parent


def unpack(problem, scenario):
    slots = sorted(problem["slotInventory"], key=lambda item: item["slotIndex"])
    staff = sorted(problem["approvedRoster"], key=lambda agent: agent["staffId"])
    jobs = [job for job in problem["approvedQueue"] if job["arrivalWave"] in scenario["includedWaves"]]
    return slots, staff, jobs


def choices_for(problem, scenario, slots, staff, job):
    """Return independent legal pairs and the reason for an empty option set."""
    policy = problem["workbook"]["operatingScope"]["policyBundle"]["companyDispatchPolicy"]["conditions"]
    if job["reviewState"] != policy["approvedQueueState"]:
        return [], "work_scope_review"
    if job["region"] not in policy["dispatchScope"]["approvedZones"]:
        return [], "region_scope_review"
    field = job["workType"] == "field_inspection"
    if field and job["siteAccess"] != policy["fieldAccessConfirmedState"]:
        return [], "site_access_review"

    # Authorizations cover duties and zones, are externally confirmed, and
    # must remain valid on the service day. This is an input check, not licensing.
    day = problem["workbook"]["serviceDate"]
    authorized = [
        index
        for index, resource in enumerate(staff)
        if any(
            grant["dutyCode"] == job["workType"]
            and job["region"] in grant["approvedZones"]
            and grant["verificationState"] == policy["currentScopeState"]
            and grant["validFrom"] <= day <= grant["validThrough"]
            for grant in resource["scopeGrants"]
        )
    ]
    if not authorized:
        return [], "authorization_review"

    slot_id = {slot["slotId"]: slot["slotIndex"] for slot in slots}
    cutoff = slot_id[job["internalTargetSlot"]]
    offered = [slot for slot in slots if slot["slotId"] in job["offeredSlotIds"] and slot["slotIndex"] <= cutoff]
    if not offered:
        return [], "no_slot_before_target"
    outages = {(loss["staffId"], slot) for loss in scenario["calendarChanges"] for slot in loss["blockedSlotIds"]}
    options = []
    count = len(slots)
    for slot in offered:
        if field and not slot["fieldSupportOpen"]:
            continue
        for index in authorized:
            agent = staff[index]
            if slot["slotId"] not in agent["freeSlotIds"] or (agent["staffId"], slot["slotId"]) in outages:
                continue
            occupied = 1 << (index * count + slot["slotIndex"])
            if field:
                # One remote shared support seat. It cannot support two inspections
                # in the same window even when two field staff are free.
                occupied |= 1 << (len(staff) * count + slot["slotIndex"])
            options.append((slot["slotIndex"], index, occupied))
    return options, "no_staffed_window" if not options else None


def allocate(problem, scenario, prior):
    slots, staff, jobs = unpack(problem, scenario)
    options, unavailable = zip(*(choices_for(problem, scenario, slots, staff, job) for job in jobs))
    late = []
    used = 0
    selected = {}
    for index, job in enumerate(jobs):
        key = job["workId"]
        if key not in scenario["lockedWorkIds"]:
            late.append(index)
            continue
        pair = prior.get(key)
        valid = next(
            (choice for choice in options[index] if pair == (slots[choice[0]]["slotId"], staff[choice[1]]["staffId"])),
            None,
        )
        if valid is None or used & valid[2]:
            raise ValueError("A locked booking is no longer valid; coordinator review required.")
        selected[index] = valid
        used |= valid[2]

    if scenario["strategy"] == "first_fit":
        for index in late:
            choice = next((choice for choice in options[index] if not choice[2] & used), None)
            if choice is not None:
                selected[index] = choice
                used |= choice[2]
    else:
        work = sorted((index for index in late if options[index]), key=lambda index: (len(options[index]), jobs[index]["workId"]))

        # Ordered lexicographic objective: scheduled count, retained prior
        # work items, exact prior placements, then negative sum of selected slot indices. All windows are
        # already limited by internal due dates. Keep ties stable by enumeration.
        def points(index, choice):
            slot, member, _ = choice
            old = prior.get(jobs[index]["workId"])
            retained = old == (slots[slot]["slotId"], staff[member]["staffId"])
            return 1, int(old is not None), int(retained), -slot

        def plus(left, right):
            return tuple(a + b for a, b in zip(left, right))

        @lru_cache(maxsize=None)
        def best(position, busy):
            if position == len(work):
                return 0, 0, 0, 0
            index = work[position]
            highest = best(position + 1, busy)
            for choice in options[index]:
                if choice[2] & busy:
                    continue
                highest = max(highest, plus(points(index, choice), best(position + 1, busy | choice[2])))
            return highest

        for position, index in enumerate(work):
            target = best(position, used)
            for choice in options[index]:
                if choice[2] & used:
                    continue
                if plus(points(index, choice), best(position + 1, used | choice[2])) == target:
                    selected[index] = choice
                    used |= choice[2]
                    break
        best.cache_clear()

    reports, schedule = [], {}
    reasons = problem["reasonCatalog"]
    rules = problem["ruleCatalog"]
    for index, job in enumerate(jobs):
        key = job["workId"]
        choice = selected.get(index)
        slot = slots[choice[0]] if choice else None
        agent = staff[choice[1]] if choice else None
        locked = key in scenario["lockedWorkIds"]
        if choice:
            schedule[key] = (slot["slotId"], agent["staffId"])
            if locked:
                reason = "locked_booking_retained"
            elif prior.get(key) and prior[key] != schedule[key]:
                reason = "replacement_capacity_found"
            elif prior.get(key) == schedule[key]:
                reason = "prior_booking_retained"
            else:
                reason = "within_approved_capacity"
        else:
            reason = unavailable[index] or "shared_plan_capacity"
        rule_id = reasons[reason]["ruleId"]
        reports.append({
            "workId": key,
            "workType": job["workType"],
            "region": job["region"],
            "status": reasons[reason]["disposition"],
            "staffId": agent["staffId"] if agent else None,
            "slotId": slot["slotId"] if slot else None,
            "startUtc": slot["startUtc"] if slot else None,
            "endUtc": slot["endUtc"] if slot else None,
            "reason": reason,
            "locked": locked,
            "ruleId": rule_id,
            "ruleRevision": rules[rule_id]["revision"],
            "ruleVerifiedAsOf": rules[rule_id]["verifiedAsOf"],
            "compatiblePairsBeforeSharedBooking": len(options[index]),
            "reviewAction": reasons[reason]["reviewAction"],
        })
    retained = sum(schedule.get(key) == pair for key, pair in prior.items())
    return {
        "id": scenario["id"],
        "requested": len(jobs),
        "scheduled": len(schedule),
        "held": sum(row["status"] == "held" for row in reports),
        "review": sum(row["status"] == "review" for row in reports),
        "changed": len(prior) - retained,
        "retained": retained,
        "locked": len(scenario["lockedWorkIds"]),
        "rows": reports,
    }, schedule


def replay(problem):
    plans, recorded = [], {}
    for scenario in problem["scenarioInputs"]:
        prior = recorded.get(scenario["comparePriorPlan"], {})
        report, schedule = allocate(problem, scenario, prior)
        plans.append(report)
        recorded[scenario["id"]] = schedule
    return plans


def visual_for(problem, plans):
    fields = {"workId", "workType", "region", "status", "staffId", "slotId", "startUtc", "endUtc", "reason", "locked"}
    result = {"timeZone": problem["workbook"]["operatingScope"]["localTimeZone"], "scenarios": []}
    for plan in plans:
        result["scenarios"].append({
            **plan,
            "rows": [{key: value for key, value in row.items() if key in fields} for row in plan["rows"]],
        })
    return result


def main():
    source = ROOT / "operational-case.json"
    item = json.loads(source.read_text(encoding="utf-8"))
    plans = replay(item)
    visual = visual_for(item, plans)
    if sys.argv[1:] == ["--write"]:
        item["recordedPlans"] = plans
        source.write_text(json.dumps(item, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        (ROOT / "visual.json").write_text(json.dumps(visual, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    elif sys.argv[1:]:
        raise SystemExit("Use replay.py or replay.py --write")
    else:
        assert plans == item["recordedPlans"] and visual == json.loads((ROOT / "visual.json").read_text(encoding="utf-8"))
        print("Verified " + ", ".join(f'{s["id"]}: {s["scheduled"]}/{s["requested"]} proposed' for s in plans))


if __name__ == "__main__":
    main()
