"""Build a workforce operations handoff from the verified roster evidence."""
from collections import Counter
from json import dumps, loads
from pathlib import Path
from sys import argv

ROOT = Path(__file__).resolve().parent


def main():
    data = loads((ROOT / "input.json").read_text())
    replay = loads((ROOT / "result.json").read_text())
    scenarios = []
    for input_case, result in zip(data["scenarios"], replay["scenarios"], strict=True):
        assert input_case["id"] == result["id"]
        scenarios.append({
            "scenarioId": result["id"],
            "businessEvent": result["label"],
            "staffAbsences": input_case["absences"],
            "incrementalDemandSeatIds": [s["id"] for s in input_case["extraCoverage"]],
            "decisionActorGrants": input_case["actorGrants"],
            "assignmentDecisions": result["coverage"],
            "exceptionQueue": result["holds"],
            "reconciliation": result["metrics"],
        })
    sites = []
    for location in data["locations"]:
        key = location["id"]
        all_seats = [r for r in data["coverage"] if r["locationId"] == key]
        role_requirements = []
        for role in sorted({s["role"] for s in all_seats}):
            seats = [s for s in all_seats if s["role"] == role]
            skill = {s["requiredSkill"] for s in seats}.pop()
            grant = {s["requiredGrant"] for s in seats}.pop()
            role_requirements.append({
                "roleName": role,
                "demandSeatIds": [s["id"] for s in seats],
                "eligibilityRule": {
                    "decision": "all predicates required",
                    "predicate": {
                        "siteAccessCheck": {"sourceField": "authorizedLocations", "mustContain": key},
                        "skillCheck": {"sourceField": "skills", "mustContain": skill},
                        "roleGrantCheck": {"sourceField": "roleGrants", "mustContain": grant},
                        "availabilityCheck": {"sourceField": "availability", "mustCover": "entire seat interval on the local date"},
                    },
                },
            })
        counts = Counter((s["date"], s["shift"]) for s in all_seats)
        sites.append({
            "siteId": key, "siteName": location["name"],
            "homeRosterWorkerIds": [w["id"] for w in data["workers"] if w["homeLocation"] == key],
            "crossSiteAuthorizedWorkerIds": [
                w["id"] for w in data["workers"] if w["homeLocation"] != key and key in w["authorizedLocations"]
            ],
            "department": {
                "functionName": "Warehouse operations",
                "workstream": {
                    "activityName": "Shift handover, dispatch and controlled-goods handling",
                    "coverageSpecification": {
                        "plannedSeatTotal": len(all_seats),
                        "shiftDemand": [
                            {"localDate": day, "shiftCode": shift, "requiredHeadcount": total}
                            for (day, shift), total in sorted(counts.items())
                        ],
                        "roleRequirements": role_requirements,
                    },
                },
            },
        })
    payload = {
        "evidenceVersion": 1,
        "sourceRecordType": "Standalone sample workforce planning data",
        "sourceArtifacts": {
            "demandAndWorkerRules": "input.json",
            "checkedSeatAssignments": "result.json",
            "verificationCommand": "python3 reproduce.py --check",
            "compositionCommand": "python3 compose_operations.py --check",
        },
        "planningCalendar": {
            "weekStarting": data["metadata"]["weekStarting"],
            "timeZone": data["metadata"]["timeZone"],
            "coveredLocalDates": sorted({s["date"] for s in data["coverage"]}),
        },
        "organization": {
            "organizationScope": "Two-site warehouse roster",
            "division": {"divisionScope": "Distribution operations", "sites": sites},
        },
        "workerDirectory": data["workers"],
        "baselineDemand": data["coverage"],
        "workingTimeAndApprovalPolicy": data["policy"],
        "scenarioDecisions": scenarios,
        "limits": {
            "policyScope": "Illustrative operating rules; not employment law or a compliance claim.",
            "vendorAccess": "No Workday account, tenant, API, or employee records were used.",
            "writebackResult": "No source system writes were attempted.",
            "optimizationResult": replay["metadata"]["optimizationClaim"],
        },
    }
    assert sum(s["department"]["workstream"]["coverageSpecification"]["plannedSeatTotal"]
               for s in sites) == 62
    assert all(s["reconciliation"]["assignedSeats"] == sum(r["workerId"] is not None for r in s["assignmentDecisions"])
               for s in scenarios)
    keys, paths = set(), []
    def walk(node, depth, path):
        if isinstance(node, dict):
            places = depth + 1
            paths.append((places, path))
            for k, v in node.items():
                keys.add(k)
                walk(v, places, path + "." + k)
        elif isinstance(node, list):
            for v in node:
                walk(v, depth, path + "[]")
    walk(payload, 0, "$")
    depth, path = max(paths)
    assert len(keys) >= 50 and depth >= 10
    target = ROOT / "operations.json"
    formatted = dumps(payload, indent=2, ensure_ascii=False) + "\n"
    if "--check" in argv:
        assert target.read_text() == formatted
    else:
        target.write_text(formatted)
    print(f"{len(keys)} unique property names, {depth} nested JSON objects (array wrappers excluded); {path}")


if __name__ == "__main__":
    main()
