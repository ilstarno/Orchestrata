"""Check recorded coverage upper bounds, capacity constraints, and disruption invariants."""
from copy import deepcopy
import json
from pathlib import Path

from replay import allocate

case = json.loads(Path(__file__).with_name("operational-case.json").read_text(encoding="utf-8"))
plans = {plan["id"]: plan for plan in case["recordedPlans"]}
jobs = {job["workId"]: job for job in case["approvedQueue"]}
staff = {agent["staffId"]: agent for agent in case["approvedRoster"]}
slots = {slot["slotId"]: slot for slot in case["slotInventory"]}
service_day = case["workbook"]["serviceDate"]
sources = {source["sourceId"] for source in case["sourceCatalog"]}
expected = {
    "baseline": (14, 7, 4, 3, 0, 0, 0),
    "recommended": (14, 9, 2, 3, 0, 0, 0),
    "surge": (21, 12, 5, 4, 0, 0, 0),
    "disruption": (21, 12, 5, 4, 2, 10, 2),
}

for fixture in case["scenarioInputs"]:
    ident = fixture["id"]
    plan = plans[ident]
    assert tuple(plan[key] for key in ["requested", "scheduled", "held", "review", "changed", "retained", "locked"]) == expected[ident]
    assert len(plan["rows"]) == plan["scheduled"] + plan["held"] + plan["review"] == plan["requested"]
    assert len({row["workId"] for row in plan["rows"]}) == plan["requested"]
    work_windows, field_windows = set(), set()
    outages = {cal["staffId"]: set(cal["blockedSlotIds"]) for cal in fixture["calendarChanges"]}
    for row in plan["rows"]:
        job = jobs[row["workId"]]
        rule = case["ruleCatalog"][row["ruleId"]]
        assert row["ruleRevision"] == rule["revision"] and row["ruleVerifiedAsOf"] == rule["verifiedAsOf"]
        assert set(rule["sourceIds"]) <= sources and row["reviewAction"]
        assert row["workType"] == job["workType"]
        if row["status"] != "scheduled":
            assert row["staffId"] is None and row["slotId"] is None and row["startUtc"] is None and row["endUtc"] is None
            continue
        assert job["reviewState"] == "approved_for_scheduling"
        slot = slots[row["slotId"]]
        assert slot["startUtc"] == row["startUtc"] and slot["endUtc"] == row["endUtc"]
        assert slot["slotId"] in job["offeredSlotIds"]
        assert slot["slotIndex"] <= slots[job["internalTargetSlot"]]["slotIndex"]
        agent = staff[row["staffId"]]
        assert row["slotId"] in agent["freeSlotIds"] and row["slotId"] not in outages.get(agent["staffId"], set())
        assert any(
            job["workType"] == grant["dutyCode"] and job["region"] in grant["approvedZones"]
            and grant["validFrom"] <= service_day <= grant["validThrough"]
            and grant["verificationState"] == "verified_by_compliance"
            for grant in agent["scopeGrants"]
        )
        pair = (row["staffId"], row["slotId"])
        assert pair not in work_windows
        work_windows.add(pair)
        if row["workType"] == "field_inspection":
            assert job["siteAccess"] == "access_confirmed" and slot["fieldSupportOpen"]
            assert row["slotId"] not in field_windows
            field_windows.add(row["slotId"])

# Independent coverage upper bounds: in normal input only nine items can
# individually be booked (five explicitly identified blockers); all are booked.
# Both high-volume plans book all seven feasible desk items and max out the five
# exclusive field support windows. These bounds do not depend on DP internals.
normal_blockers = {"WORK-10", "WORK-11", "WORK-12", "WORK-13", "WORK-14"}
assert {row["workId"] for row in plans["recommended"]["rows"] if row["status"] == "scheduled"} == {
    key for key, item in jobs.items() if item["arrivalWave"] == "normal"
} - normal_blockers
surge_desk = {"WORK-01", "WORK-02", "WORK-07", "WORK-08", "WORK-15", "WORK-18", "WORK-19"}
for ident in ["surge", "disruption"]:
    booked = [row for row in plans[ident]["rows"] if row["status"] == "scheduled"]
    assert {row["workId"] for row in booked if row["workType"] == "desk_contact_prep"} == surge_desk
    assert sum(row["workType"] == "field_inspection" for row in booked) == len(slots) == 5

prior = {row["workId"]: (row["slotId"], row["staffId"]) for row in plans["surge"]["rows"] if row["status"] == "scheduled"}
after = {row["workId"]: (row["slotId"], row["staffId"]) for row in plans["disruption"]["rows"] if row["status"] == "scheduled"}
assert prior.keys() == after.keys()
assert {key for key in prior if prior[key] != after[key]} == {"WORK-09", "WORK-20"}
assert all(prior[key] == after[key] for key in ["WORK-02", "WORK-05"])
# Two altered bookings are necessary to retain all previously scheduled work:
# WORK-09 can only use WIN-05 after the outage and WIN-05 was held by WORK-20.
assert prior["WORK-09"][0] == "WIN-04" and after["WORK-09"][0] == "WIN-05"
assert prior["WORK-20"][0] == "WIN-05" and after["WORK-20"][0] == "WIN-04"

bad_lock = deepcopy(case["scenarioInputs"][-1])
bad_lock["calendarChanges"].append({"staffId": prior["WORK-02"][1], "blockedSlotIds": [prior["WORK-02"][0]]})
try:
    allocate(case, bad_lock, prior)
except ValueError as issue:
    assert "locked booking" in str(issue)
else:
    raise AssertionError("A locked booking with no assigned staff must not be silently moved")

print("Verified all 70 decisions, two independent coverage upper bounds, two required appointment moves, and refusal to alter an invalid lock.")
