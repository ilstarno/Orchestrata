#!/usr/bin/env python3
"""Rebuild and verify this synthetic fixture with Python's standard library and native ABI v1."""

import argparse
import ctypes
import hashlib
import itertools
import json
from math import prod
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent
ABI_VERSION = 1
SEARCH_LIMIT = 2
SCOPE_EXCLUSIONS = [
    "actual stowage, stability, trim, deck or axle loading",
    "dangerous-goods segregation and cargo securing",
    "reefer slot counts, temperature compliance, and cold-chain assurance",
    "voyage scheduling, berth availability, handling, and actual booking",
]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read_json(name):
    return json.loads((ROOT / name).read_bytes())


def content_sha256(name):
    return hashlib.sha256((ROOT / name).read_bytes()).hexdigest()


def normalize(raw):
    vessels = raw["vessels"]
    consignments = raw["consignments"]
    vessel_by_id = {vessel["id"]: vessel for vessel in vessels}
    require(list(v["resource_index"] for v in vessels) == list(range(len(vessels))), "vessel indices must be dense and ordered")
    require(len(vessel_by_id) == len(vessels), "duplicate vessel id")
    require([c["variable_index"] for c in consignments] == list(range(len(consignments))), "consignment indices must be dense and ordered")
    require(len({c["id"] for c in consignments}) == len(consignments), "duplicate consignment id")
    variables, items, preferences = [], [], []
    for c in consignments:
        eligibility = c["eligibility"]
        by_vessel = {decision["vessel_id"]: decision for decision in eligibility}
        require(len(eligibility) == len(by_vessel) and by_vessel.keys() == vessel_by_id.keys(), f"{c['id']}: one eligibility decision required per vessel")
        require(all(type(decision["allowed"]) is bool for decision in eligibility), f"{c['id']}: eligibility must be boolean")
        require(all(decision["reason_code"] and decision["reason"] for decision in eligibility), f"{c['id']}: missing review rationale")
        domain = [v["resource_index"] for v in vessels if by_vessel[v["id"]]["allowed"]]
        preference = c["preference"]
        preferred = vessel_by_id[preference["vessel_id"]]["resource_index"]
        require(preferred in domain, f"{c['id']}: preferred vessel is not eligible")
        variables.append({"name": c["id"], "domain": domain})
        items.append({"variable": c["variable_index"], "load": c["gross_mass_t"]})
        preferences.append({
            "kind": "prefer_value",
            "variable": c["variable_index"],
            "value": preferred,
            "soft_weight": preference["missed_penalty"],
        })
    limits = [{"resource": v["resource_index"], "capacity": v["weight_allowance_t"]} for v in vessels]
    for key, values in (("load", items), ("capacity", limits), ("soft_weight", preferences)):
        require(all(type(v[key]) is int and v[key] > 0 for v in values), f"{key} must be a positive integer")
    return {
        "variables": variables,
        "constraints": [{"kind": "resource_capacity", "items": items, "limits": limits}, *preferences],
    }


def model_totals(problem, values):
    loads = [0 for _ in problem["constraints"][0]["limits"]]
    for value, item in zip(values, problem["constraints"][0]["items"], strict=True):
        loads[value] += item["load"]
    return loads


def missed_penalty(problem, values):
    return sum(rule["soft_weight"] for rule in problem["constraints"][1:] if values[rule["variable"]] != rule["value"])


def exhaustive_oracle(problem):
    best_penalty = None
    best = []
    feasible_count = 0
    examined = 0
    limits = problem["constraints"][0]["limits"]
    for values in itertools.product(*(v["domain"] for v in problem["variables"])):
        examined += 1
        loads = model_totals(problem, values)
        if any(load > limit["capacity"] for load, limit in zip(loads, limits, strict=True)):
            continue
        feasible_count += 1
        penalty = missed_penalty(problem, values)
        if best_penalty is None or penalty < best_penalty:
            best_penalty, best = penalty, [values]
        elif penalty == best_penalty:
            best.append(values)
    return examined, feasible_count, best_penalty, best


def native_solve(library_path, problem_bytes, cap):
    library = ctypes.CDLL(str(library_path.resolve()))
    library.orchestrata_abi_version.argtypes = []
    library.orchestrata_abi_version.restype = ctypes.c_uint32
    library.orchestrata_solve_json_v1.argtypes = [
        ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.c_uint64, ctypes.POINTER(ctypes.c_void_p),
    ]
    library.orchestrata_solve_json_v1.restype = ctypes.c_int32
    library.orchestrata_free_json_v1.argtypes = [ctypes.c_void_p]
    library.orchestrata_free_json_v1.restype = None
    abi_version = library.orchestrata_abi_version()
    require(abi_version == ABI_VERSION, f"unexpected ABI version {abi_version}")
    request = (ctypes.c_uint8 * len(problem_bytes)).from_buffer_copy(problem_bytes)

    def run(limit):
        output = ctypes.c_void_p()
        status = library.orchestrata_solve_json_v1(request, len(problem_bytes), limit, ctypes.byref(output))
        try:
            require((status == 0) == bool(output), f"unexpected native result pointer for status {status}")
            return status, json.loads(ctypes.string_at(output)) if output else None
        finally:
            if output:
                library.orchestrata_free_json_v1(output)

    status, result = run(cap)
    require(status == 0, f"native solve failed with status {status}")
    lower_cap_status, lower_cap_result = run(cap - 1)
    require(lower_cap_status == SEARCH_LIMIT and lower_cap_result is None, "smaller cap did not produce a distinct native search-limit response")
    return abi_version, result, lower_cap_status


def generate_report(raw, problem, expected, engine_revision, native_evidence):
    require(normalize(raw) == problem, "operating input does not normalize to the exact checked problem")
    solution = expected["solution"]
    values = solution["values"]
    variables = problem["variables"]
    require(len(values) == len(variables), "invalid assignment length")
    require(all(type(value) is int and value in variable["domain"] for value, variable in zip(values, variables, strict=True)), "assignment violates eligibility")
    examined, feasible_count, optimal_penalty, optimum = exhaustive_oracle(problem)
    require(tuple(values) in optimum, "expected result is not optimal in the independent oracle")
    require(examined == solution["evaluated"] == prod(len(v["domain"]) for v in variables), "evaluation count mismatch")
    require(solution["score"] == {"hard": 0, "soft": -optimal_penalty}, "score mismatch")
    abi, native_result, lower_cap_status = native_evidence
    require(native_result == expected, "native returned a result different from expected.json")
    vessels = raw["vessels"]
    loads = model_totals(problem, values)
    by_shipment = []
    for c, value in zip(raw["consignments"], values, strict=True):
        vessel_id = vessels[value]["id"]
        preference = c["preference"]
        selected = vessel_id == preference["vessel_id"]
        decisions = c["eligibility"]
        by_shipment.append({
            "consignment_id": c["id"],
            "lot_ref": c["supporting"]["lot_ref"],
            "assigned_vessel_id": vessel_id,
            "assigned_mass_t": c["gross_mass_t"],
            "eligible_vessel_ids": [d["vessel_id"] for d in decisions if d["allowed"]],
            "eligibility_detail": [
                {**d, "selected": d["vessel_id"] == vessel_id} for d in decisions
            ],
            "preference": {
                "preferred_vessel_id": preference["vessel_id"],
                "result": "met" if selected else "missed",
                "missed_penalty": preference["missed_penalty"],
                "applied_penalty": 0 if selected else preference["missed_penalty"],
                "reason": preference["reason"],
            },
        })
    by_vessel = [
        {
            "vessel_id": v["id"],
            "label": v["label"],
            "allowance_t": v["weight_allowance_t"],
            "assigned_mass_t": load,
            "residual_t": v["weight_allowance_t"] - load,
            "consignment_ids": [c["consignment_id"] for c in by_shipment if c["assigned_vessel_id"] == v["id"]],
            "within_modeled_allowance": load <= v["weight_allowance_t"],
        }
        for v, load in zip(vessels, loads, strict=True)
    ]
    return {
        "fixture_id": raw["fixture_id"],
        "status": "optimal_for_checked_model",
        "units": {"assigned_mass": "metric tonne (t)", "preference_penalty": "integer points"},
        "totals": {
            "consignments": len(by_shipment),
            "vessels": len(by_vessel),
            "assigned_mass_t": sum(loads),
            "allowance_t": sum(v["allowance_t"] for v in by_vessel),
            "residual_t": sum(v["residual_t"] for v in by_vessel),
        },
        "shipments": by_shipment,
        "vessels": by_vessel,
        "model_checks": {
            "hard": {
                "all_consignments_assigned_once": len(values) == len(variables),
                "all_assignments_in_eligible_domains": True,
                "all_vessels_within_weight_allowance": all(v["within_modeled_allowance"] for v in by_vessel),
                "weight_total_reconciles": sum(loads) == sum(c["gross_mass_t"] for c in raw["consignments"]),
                "score": solution["score"]["hard"],
            },
            "soft": {
                "preferences_met": sum(c["preference"]["result"] == "met" for c in by_shipment),
                "preferences_missed": sum(c["preference"]["result"] == "missed" for c in by_shipment),
                "total_missed_penalty": missed_penalty(problem, values),
                "score": solution["score"]["soft"],
            },
        },
        "proof": {
            "native": {
                "engine_revision": engine_revision,
                "abi_version": abi,
                "operation": "orchestrata_solve_json_v1",
                "evaluation_cap": examined,
                "evaluated": native_result["solution"]["evaluated"],
                "response_matches_expected": True,
                "smaller_cap": examined - 1,
                "smaller_cap_status": lower_cap_status,
                "smaller_cap_meaning": "ORCHESTRATA_SEARCH_LIMIT; no partial response",
            },
            "independent_enumeration": {
                "candidate_assignments": examined,
                "hard_feasible_assignments": feasible_count,
                "hard_infeasible_assignments": examined - feasible_count,
                "optimal_assignments": len(optimum),
                "optimal_missed_penalty": optimal_penalty,
            },
            "source_sha256": {name: content_sha256(name) for name in ("input.json", "problem.json", "expected.json")},
            "scope": "Only this synthetic finite booking-allocation model was solved and verified.",
            "not_verified": SCOPE_EXCLUSIONS,
        },
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library", type=Path, required=True, help="locally built native Orchestrata cdylib")
    parser.add_argument("--engine-revision", required=True, help="Git revision used to build the native library")
    parser.add_argument("--write", action="store_true", help="replace allocation-report.json; otherwise compare with it")
    args = parser.parse_args()
    require(re.fullmatch(r"[a-f0-9]{40}", args.engine_revision), "record the full 40-digit Git revision")
    raw, problem, expected = (read_json(name) for name in ("input.json", "problem.json", "expected.json"))
    require(normalize(raw) == problem, "operating input does not normalize to the exact checked problem")
    cap = prod(len(variable["domain"]) for variable in problem["variables"])
    native = native_solve(args.library, (ROOT / "problem.json").read_bytes(), cap)
    report = generate_report(raw, problem, expected, args.engine_revision, native)
    if args.write:
        (ROOT / "allocation-report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    else:
        require(report == read_json("allocation-report.json"), "committed allocation report disagrees with verified evidence")
    print(f"Cargo allocation verified: {cap} candidates, {report['totals']['assigned_mass_t']} t assigned.")


if __name__ == "__main__":
    main()
