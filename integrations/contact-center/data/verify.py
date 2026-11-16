"""Offline provider-labelled contact-center coverage and fallback replay."""
from datetime import datetime
from pathlib import Path
import argparse
import gzip
import itertools
import json

ROOT = Path(__file__).resolve().parent
NETWORK = ROOT.parent / "call-center-network"
SCENARIO = "NYC-early-baseline"
FRESHNESS = 300
PROVIDERS = {
    "genesys": ("Genesys Cloud CX", "queues", "queueReference", "media"),
    "ringcx": ("RingCentral RingCX", "queueReports", "workgroupReference", "mediaType"),
    "asterisk": ("Asterisk PBX", "planningRows", "queueLabel", "channel"),
}


def read(path):
    return json.loads(path.read_text())


def source(name):
    with gzip.open(NETWORK / name, "rt") as stream:
        return next(row for line in stream if (row := json.loads(line))["scenario_id"] == SCENARIO)


def snapshots(raw):
    return [{
        "providerId": key, "providerLabel": label, "format": "Illustrative envelope; not a vendor payload or schema",
        "capturedAt": "2026-11-16T08:57:00-05:00", "decisionTime": "2026-11-16T09:00:00-05:00",
        "branchId": "NYC", "window": raw["coverage_window"],
        rows: [{
            queue_key: queue["queue_id"], channel_key: queue["channel"], "plannedPositions": 1,
            "inputOrigin": "independent_chat_planning_input" if key == "asterisk" and queue["channel"] == "chat"
                           else "illustrative_queue_export",
        } for queue in raw["queues"]],
    } for key, (label, rows, queue_key, channel_key) in PROVIDERS.items()]


def time(value):
    point = datetime.fromisoformat(value)
    if point.utcoffset() is None:
        raise ValueError("Explicit UTC offset required")
    return point


def normalize(snapshot, raw):
    if snapshot is None:
        return None, "snapshot_missing"
    try:
        _, rows, queue_key, channel_key = PROVIDERS[snapshot["providerId"]]
        age = (time(snapshot["decisionTime"]) - time(snapshot["capturedAt"])).total_seconds()
        if age < 0:
            return None, "snapshot_invalid"
        if age > FRESHNESS:
            return None, "snapshot_stale"
        lookup = {q["queue_id"]: q["channel"] for q in raw["queues"]}
        if snapshot["branchId"] != raw["branch_id"] or snapshot["window"] != raw["coverage_window"] or time(snapshot["decisionTime"]) != time(snapshot["window"]["start"]):
            return None, "snapshot_invalid"
        ids = []
        for row in snapshot[rows]:
            q, channel = row[queue_key], row[channel_key]
            if lookup.get(q) != channel or type(row["plannedPositions"]) is not int or row["plannedPositions"] != 1 or q in ids:
                return None, "snapshot_invalid"
            ids.append(q)
        if set(ids) != set(lookup):
            return None, "snapshot_invalid"
        return {"branchId": snapshot["branchId"], "window": snapshot["window"], "queueIds": list(lookup),
                "snapshotAgeSeconds": age}, None
    except (KeyError, TypeError, ValueError):
        return None, "snapshot_invalid"


def checks(operator, queue, start, end):
    shift = operator["shift"]
    return {
        "inHomeDepartment": operator["home_department"] == queue["department_id"],
        "shiftCoversWindow": time(shift["start"]) <= start and end <= time(shift["end"]),
        "noBreakOverlap": not any(time(b["start"]) < end and start < time(b["end"]) for b in operator["breaks"]),
        "roleCapabilityMet": queue["capability"] in operator["capability_tags"],
        "taskProficiencyMet": operator["task_skills"].get(queue["skill"], 0) >= queue["min_skill"],
        "serviceLanguageMet": operator["language_proficiency"].get(queue["language"], 0) >= queue["min_language"],
        "experienceMet": operator["role_months"] >= queue["min_role_months"],
        "channelCapacityMet": operator["parallel_contact_limit"].get(queue["channel"], 0) >= queue["min_parallel_contacts"],
    }


def calculate(raw, roster, archived, prepared):
    branch = next(b for b in roster["branches"] if b["branch_id"] == raw["branch_id"])
    operators = [p for d in branch["departments"] for p in d["operators"]]
    queues = raw["queues"]
    assert not raw["temporary_unavailability"]
    # This NYC base window has no cross-department pool; exact domains are
    # checked against the existing normalized archive, not inferred globally.
    start, end = (time(raw["coverage_window"][key]) for key in ("start", "end"))
    domains, proof = [], {}
    for q in queues:
        evidence = [checks(p, q, start, end) for p in operators]
        proof[q["queue_id"]] = evidence
        eligible = [i for i, checks_ in enumerate(evidence) if all(checks_.values())]
        key = lambda i: (-operators[i]["task_skills"][q["skill"]], -operators[i]["language_proficiency"][q["language"]],
                         -operators[i]["role_months"], operators[i]["operator_id"])
        eligible.sort(key=key)
        domains.append(eligible)
    assert [x["domain"] for x in prepared["problem"]["variables"]] == domains
    assert [p["operator_id"] for p in operators] == prepared["operator_index_order"]
    checked, feasible, best = 0, 0, None
    for vector in itertools.product(*domains):
        checked += 1
        if len(set(vector)) != len(vector):
            continue
        feasible += 1
        penalty = sum(q["preference_weight"] for idx, q in enumerate(queues) if vector[idx] != domains[idx][0])
        option = (penalty, vector)
        best = option if best is None else min(best, option)
    penalty, sol = best
    assert list(sol) == archived["native_solution"]["values"]
    assert penalty == -archived["native_solution"]["score"]["soft"]
    assert checked == archived["proof"]["evaluated_combinations"]
    assert feasible == archived["proof"]["hard_feasible_combinations"]
    positions = []
    for idx, q in enumerate(queues):
        p = operators[sol[idx]]
        assert p["operator_id"] == archived["assignments"][idx]["assigned_operator_id"]
        positions.append({
            "queueId": q["queue_id"], "departmentId": q["department_id"], "channel": q["channel"],
            "proposedOperatorId": p["operator_id"], "eligibleCandidateCount": len(domains[idx]),
            "requirements": {k: q[k] for k in ("skill","min_skill","language","min_language","capability","min_role_months","min_parallel_contacts")},
            "selectedProfile": {k: p[k] for k in ("task_skills","language_proficiency","role_months","parallel_contact_limit","shift","breaks")},
            "hardRuleResults": proof[q["queue_id"]][sol[idx]], "preferredOperatorRetained": sol[idx] == domains[idx][0],
        })
    return {"branchId": "NYC", "profilesScreened": len(operators), "positionsCovered": len(positions),
            "positionsRequested": len(queues), "positions": positions, "allDistinct": len(set(sol)) == len(sol),
            "exactScoredCombinations": checked, "hardFeasibleCombinations": feasible, "preferencePenalty": penalty,
            "nativeArchiveMatches": True, "nativeEngineRunDuringReplay": False}


def preview(snapshot, raw, decision):
    normalized, reason = normalize(snapshot, raw)
    return {"status": "review_only" if normalized is not None else "manual_review",
            "reason": reason, "normalized": normalized, "proposedPositions": decision["positionsCovered"] if normalized else 0,
            "reusePreviousDraft": False, "writeback": False}


def structured(samples, result):
    return {
        "id": "contact-center-provider-review",
        "title": "Three provider-labeled inputs, one coverage review",
        "dataClassification": "Illustrative sample envelopes and existing verified staffing evidence",
        "evidenceFiles": {"intakeSamples": "provider-samples.json", "outcomes": "decisions.json",
                          "verification": "verify.py", "sourceScenario": SCENARIO},
        "operatingOrganization": {
            "operationPurpose": "Review half-hour contact-center coverage before supervisors act",
            "businessUnit": {
                "unitName": "Customer service operations",
                "serviceFunction": {
                    "functionName": "Shift and queue coverage",
                    "workstream": {
                        "workstreamName": "Provider-labelled queue-intake comparison",
                        "planningRun": {
                            "scopeReview": result["networkContext"],
                            "inputVariants": [{"providerId": p["providerId"],"providerLabel": p["providerLabel"],
                                               "envelopeType": p["format"], "inputCapturedAt": p["capturedAt"]} for p in samples],
                            "feedPolicy": {"freshnessLimitSeconds": FRESHNESS, "missingFeedAction": "manual_review",
                                           "staleFeedAction": "manual_review", "previousPlanSubstitution": False,
                                           "productionWritebackEnabled": False},
                            "decisionLedger": {
                                "providerComparisons": result["providerComparisons"],
                                "commonCoverage": {
                                    "runEvidence": {k: v for k,v in result["commonDecision"].items() if k != "positions"},
                                    "proposedAssignments": [{
                                        "assignmentContext": {k:v for k,v in assignment.items() if k not in ("requirements","selectedProfile","hardRuleResults")},
                                        "reviewBasis": {"configuredRequirements": assignment["requirements"],
                                                        "operatorInputSnapshot": assignment["selectedProfile"],
                                                        "qualificationEvidence": {"checks": assignment["hardRuleResults"]}},
                                    } for assignment in result["commonDecision"]["positions"]],
                                },
                            },
                        },
                    },
                },
            },
        },
    }


def dimensions(node, keys, depth=0):
    if isinstance(node, list):
        return max([depth] + [dimensions(x, keys, depth) for x in node])
    if isinstance(node, dict):
        keys.update(node)
        return max([depth + 1] + [dimensions(x, keys, depth + 1) for x in node.values()])
    return depth


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    raw = source("raw-scenarios.ndjson.gz")
    roster = read(NETWORK / "roster.json")
    samples = snapshots(raw)
    decision = calculate(raw, roster, source("verified-output.ndjson.gz"), source("normalized-problems.ndjson.gz"))
    comparisons = []
    for p in samples:
        _, rows, _, _ = PROVIDERS[p["providerId"]]
        cases = {
            "current": p, "missing": None,
            "stale": p | {"capturedAt": "2026-11-16T08:45:00-05:00"},
            "invalid": p | {rows: p[rows][:-1]},
        }
        previews = {key: preview(value, raw, decision) for key, value in cases.items()}
        assert previews["current"]["proposedPositions"] == 8
        assert [previews[key]["reason"] for key in ("missing","stale","invalid")] == ["snapshot_missing","snapshot_stale","snapshot_invalid"]
        assert all(not v["writeback"] and not v["reusePreviousDraft"] for v in previews.values())
        assert all(previews[key]["proposedPositions"] == 0 for key in ("missing","stale","invalid"))
        comparisons.append({"providerId":p["providerId"], "providerLabel":p["providerLabel"], "variants":previews})
    assert comparisons[0]["variants"]["current"]["normalized"] == comparisons[1]["variants"]["current"]["normalized"] == comparisons[2]["variants"]["current"]["normalized"]
    network = {"branches":len(roster["branches"]),
               "operators":sum(len(d["operators"]) for b in roster["branches"] for d in b["departments"]),
               "coverageScope":"Existing five-branch roster, this replay uses only the NYC early window"}
    assert list(network.values())[:2] == [5,500]
    result = {"schemaVersion":1, "sourceScenarioId": SCENARIO, "networkContext": network,
              "commonDecision":decision, "providerComparisons":comparisons}
    case = structured(samples, result)
    keys = set()
    depth = dimensions(case, keys)
    assert depth >= 10 and len(keys) >= 50
    for name, data in [("provider-samples.json",{"schemaVersion":1,"snapshots":samples}), ("decisions.json",result),
                       ("operational-case.json",case)]:
        serialized = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
        if args.write:
            (ROOT / name).write_text(serialized)
        else:
            assert (ROOT / name).read_text() == serialized, name
    print(f"Verified 3 provider samples × 4 states, {decision['positionsCovered']}/8 positions, {depth} object levels, {len(keys)} keys.")


if __name__ == "__main__":
    main()
