"""Generate or verify all sample artifacts against a real Orchestrata native library."""

import argparse
from ctypes import CDLL, POINTER, byref, c_char_p, c_int32, c_size_t, c_uint32, c_uint64, c_void_p, string_at
from itertools import product
import json
from pathlib import Path

from make_sample import make_sample
from normalize import normalize, require

HERE = Path(__file__).resolve().parent


def native_solve(path, problem, budget):
    library = CDLL(str(path.resolve(strict=True)))
    abi = library.orchestrata_abi_version
    abi.argtypes, abi.restype = [], c_uint32
    require(abi() == 1, "requires native ABI v1")
    solve = library.orchestrata_solve_json_v1
    solve.argtypes = [c_char_p, c_size_t, c_uint64, POINTER(c_void_p)]
    solve.restype = c_int32
    free = library.orchestrata_free_json_v1
    free.argtypes, free.restype = [c_void_p], None
    request = json.dumps(problem, separators=(",", ":")).encode("utf-8")
    out = c_void_p()
    status = solve(request, len(request), budget, byref(out))
    try:
        require(status == 0 and bool(out), f"native solve failed (status {status})")
        result = json.loads(string_at(out))["solution"]
    finally:
        free(out)
    require(result is not None, "native solver proved this sample has no feasible coverage")
    return result


def oracle(problem):
    """Verify the exact fixture using an independent enumeration and objective."""
    best, hard_feasible = None, 0
    for values in product(*(item["domain"] for item in problem["variables"])):
        if len(set(values)) != len(values):
            continue
        hard_feasible += 1
        score = -sum(
            rule["soft_weight"] for rule in problem["constraints"][1:]
            if values[rule["variable"]] != rule["value"]
        )
        if not best or score > best[1]:
            best = (values, score)
    return best, hard_feasible


def demand_report(q, policy, seconds):
    probability = policy["demand_scenarios_basis_points"]
    planned_seconds = seconds * policy["voice_occupancy_target_basis_points"] // 10000
    scenarios = {
        key: {
            "probability_basis_points": bp,
            "arrivals": q["arrival_counts"][key],
            "required_handling_seconds": q["arrival_counts"][key] * q["average_handle_seconds"],
        }
        for key, bp in probability.items()
    }
    expected_times_10000 = sum(s["probability_basis_points"] * s["required_handling_seconds"]
                               for s in scenarios.values())
    return {
        "scenarios": scenarios,
        "average_handle_seconds": q["average_handle_seconds"],
        "expected_arrivals": sum(s["probability_basis_points"] * s["arrivals"]
                                 for s in scenarios.values()) / 10000,
        "expected_handling_seconds": expected_times_10000 / 10000,
        "expected_occupancy_basis_points_rounded": (expected_times_10000 + seconds // 2) // seconds,
        "planned_handling_seconds": planned_seconds,
        "probability_above_planned_capacity_basis_points": sum(
            s["probability_basis_points"] for s in scenarios.values()
            if s["required_handling_seconds"] > planned_seconds
        ),
    }


def enrich(raw, problem, native, screened, window_secs, combinations):
    (oracle_values, oracle_soft), number_feasible = oracle(problem)
    values = native["values"]
    require(list(oracle_values) == values and native["score"] == {"hard": 0, "soft": oracle_soft},
            "native assignment disagrees with independent enumeration")
    require(native["evaluated"] == combinations, "native did not enumerate the entire search space")
    ids = [operator["id"] for operator in raw["operators"]]
    rows = []
    for index, (queue, selected, audit) in enumerate(zip(raw["queues"], values, screened, strict=True)):
        preferred = audit["preferred"]
        assigned = selected == preferred
        rows.append({
            "queue_id": queue["id"], "queue_label": queue["label"],
            "assigned_operator": ids[selected], "native_value": selected,
            "required": {
                "skill": queue["skill"], "language": queue["language"],
                "minimum_role_months": queue["minimum_role_months"],
                "voice_contact_capacity": 1, "must_cover_entire_window": True, "break_overlap_allowed": False,
            },
            "eligible_operator_count": len(audit["eligible"]),
            "eligible_operators": [ids[value] for value in audit["eligible"]],
            "selected_eligibility": audit["evidence"][selected],
            "excluded_operator_count": len(audit["excluded"]),
            "exclusion_reason_counts": audit["reason_counts"],
            "excluded_operator_reasons": audit["excluded"],
            "preference": {
                "preferred_operator": ids[preferred],
                "rank_order": ["skill_rating_desc", "language_rating_desc", "experience_months_desc", "operator_id_asc"],
                "satisfied": assigned, "weight": queue["soft_preference_weight"],
                "soft_penalty": 0 if assigned else -queue["soft_preference_weight"],
                "preferred_operator_assigned_to": (
                    raw["queues"][values.index(preferred)]["id"] if preferred in values else None
                ),
            },
            "demand_context": demand_report(queue, raw["policy"], window_secs),
        })
    score = native["score"]
    require(sum(row["preference"]["soft_penalty"] for row in rows) == score["soft"], "soft score reconciliation failed")
    return {
        "schema_version": 1, "dataset_id": raw["dataset_id"], "classification": raw["classification"],
        "coverage_window": raw["coverage_window"], "timezone": raw["timezone"], "coverage_window_seconds": window_secs,
        "coverage": {
           "queues_requested": len(rows), "queues_with_an_eligible_assigned_operator": len(rows),
           "distinct_operators_assigned": len(set(values)), "operators_screened_per_queue": len(ids),
           "queues_with_nonzero_demand_risk": [r["queue_id"] for r in rows
               if r["demand_context"]["probability_above_planned_capacity_basis_points"]],
        },
        "constraint_verdicts": {
            "eligible_assignment_per_queue": "satisfied", "simultaneous_operators_distinct": "satisfied",
            "available_for_entire_window_without_overlapping_break": "satisfied",
            "required_skill_language_experience_and_voice_capacity": "satisfied",
            "soft_preferences_met": sum(row["preference"]["satisfied"] for row in rows),
            "soft_preferences_missed": [row["queue_id"] for row in rows if not row["preference"]["satisfied"]],
            "demand_context_used_as_constraint": False,
        },
        "proof": {
            "method": "native_abi_v1_bounded_exact_search", "status": "optimal_within_model",
            "budget_combinations": raw["policy"]["evaluation_budget"],
            "candidate_combinations": combinations, "evaluated_combinations": native["evaluated"],
            "hard_feasible_combinations": number_feasible,
            "independent_enumeration_matches_native": True,
            "selection_policy": "minimum soft penalty magnitude among hard-feasible assignments; first in declared domain order breaks ties",
        },
        "native_score": score,
        "queues": rows,
    }


def encode(value):
    return json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n"


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("--library", type=Path, required=True)
    cli.add_argument("--write", action="store_true", help="write canonical artifacts; default checks current bytes")
    args = cli.parse_args()
    raw = make_sample()
    problem, screened, seconds, combinations = normalize(raw)
    result = native_solve(args.library, problem, raw["policy"]["evaluation_budget"])
    rich = enrich(raw, problem, result, screened, seconds, combinations)
    for name, value in (
        ("raw-input.json", raw), ("problem.json", problem), ("expected.json", result),
        ("enriched-result.json", rich),
    ):
        path = HERE / name
        canonical = encode(value)
        if args.write:
            path.write_text(canonical, encoding="utf-8")
        else:
            require(path.read_text(encoding="utf-8") == canonical, f"{name} diverges from recomputation")
    print(f"{len(raw['operators'])} operators, {len(rich['queues'])} queues, "
          f"{result['evaluated']} assignments, {rich['proof']['hard_feasible_combinations']} feasible; "
          f"hard={result['score']['hard']}, soft={result['score']['soft']}")


if __name__ == "__main__":
    main()
