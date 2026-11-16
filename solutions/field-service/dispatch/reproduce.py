"""Reproduce the three fixed-wave sample dispatches against the native solver."""
from ctypes import CDLL, POINTER, byref, c_char_p, c_int32, c_size_t, c_uint32, c_uint64, c_void_p, string_at
from itertools import product
import argparse
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise ValueError(message)


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
    require(result is not None, "the all-exception assignment should be feasible")
    return result


def evaluate_pair(source, job, tech, absent):
    roundtrip = job["travelMinutes"][tech["id"]] * 2
    arrival = tech["ready"] + roundtrip // 2
    service_start = max(arrival, job["earliest"])
    return_at = service_start + job["serviceMinutes"] + roundtrip // 2
    overtime = max(0, return_at - tech["shiftEnd"])
    excluded = []
    if tech["id"] in absent:
        excluded.append("Called out")
    if job["skill"] not in tech["skills"]:
        excluded.append("Required skill unavailable")
    if service_start > job["latest"]:
        excluded.append("Misses service window")
    if overtime > source["overtimeLimitMinutes"]:
        excluded.append("Exceeds overtime cap")
    return {
        "technician": tech["name"], "arrival": arrival, "start": service_start, "returnAt": return_at,
        "travelMinutes": roundtrip, "overtimeMinutes": overtime, "excluded": excluded,
    }


def compute(source, scenario, library):
    people = source["technicians"]
    jobs = source["jobs"] if scenario["urgent"] else source["jobs"][:-1]
    require([p["id"] for p in people] == list(range(len(people))), "technician indices must be dense")
    cost_weight = source["overtimeWeight"]
    pairs = [{p["id"]: evaluate_pair(source, job, p, set(scenario["absent"])) for p in people} for job in jobs]
    domains = [[p["id"] for p in people if not row[p["id"]]["excluded"]] + [100 + index]
               for index, row in enumerate(pairs)]
    rules = [{"kind": "all_different", "variables": list(range(len(jobs))), "hard_weight": 1}]
    for index, (job, domain) in enumerate(zip(jobs, domains, strict=True)):
        for candidate in domain[:-1]:
            pair = pairs[index][candidate]
            cost = pair["travelMinutes"] + pair["overtimeMinutes"] * cost_weight
            require(job["unservedPenalty"] > cost, "unserved penalty must dominate one assignment cost")
            rules.append({"kind": "prefer_value", "variable": index, "value": candidate,
                          "soft_weight": job["unservedPenalty"] - cost})
    problem = {"variables": [{"name": job["id"], "domain": domain}
                             for job, domain in zip(jobs, domains, strict=True)], "constraints": rules}
    budget = 1000
    native = native_solve(library, problem, budget)

    # Independent business-model oracle: enumerate every technician or exception
    # from the raw candidates (not the normalized domains or solver scores).
    def oracle_options():
        options = list(range(len(people))) + [None]
        for values in product(options, repeat=len(jobs)):
            used = [p for p in values if p is not None]
            if len(set(used)) != len(used):
                continue
            if any(p is not None and pairs[i][p]["excluded"] for i, p in enumerate(values)):
                continue
            objective = sum(jobs[i]["unservedPenalty"] if p is None else
                            pairs[i][p]["travelMinutes"] + pairs[i][p]["overtimeMinutes"] * cost_weight
                            for i, p in enumerate(values))
            yield objective, values

    expected_objective, expected = min(oracle_options(), key=lambda item: item[0])
    require(native["values"] == [100 + i if p is None else p for i, p in enumerate(expected)],
            "native assignment does not match independent business-model minimum")
    require(native["score"]["hard"] == 0 and native["evaluated"] == len(list(product(*domains))),
            "hard score or exact solver search count mismatch")
    objective_offset = sum(rule["soft_weight"] for rule in rules[1:]) - sum(job["unservedPenalty"] for job in jobs)
    require(-native["score"]["soft"] == expected_objective + objective_offset,
            "native score does not reconcile to business objective")

    rows = []
    for index, (job, selected) in enumerate(zip(jobs, expected, strict=True)):
        alternatives = domains[index][:-1]
        used_by = {value: jobs[i]["name"] for i, value in enumerate(expected) if value is not None and i != index}
        reasons = [{"technician": people[p]["name"], "selectedFor": used_by[p]} for p in alternatives if p in used_by]
        rows.append({"jobId": job["id"], "job": job["name"], "skill": job["skill"], "window": [job["earliest"], job["latest"]],
                     "assignment": None if selected is None else pairs[index][selected],
                     "qualifiedAndAvailable": [people[p]["name"] for p in alternatives],
                     "alreadyAssigned": reasons, "screening": list(pairs[index].values())})
    total = {key: sum(row["assignment"][key] for row in rows if row["assignment"]) for key in ("travelMinutes", "overtimeMinutes")}
    total.update({"assignedJobs": sum(row["assignment"] is not None for row in rows), "requestedJobs": len(rows),
                  "businessScore": expected_objective})
    return {"id": scenario["id"], "label": scenario["label"], "summary": total, "assignments": rows,
            "problem": problem, "native": native, "independentEnumerationMatches": True}


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("--library", type=Path, required=True)
    cli.add_argument("--write", action="store_true")
    args = cli.parse_args()
    source = json.loads((HERE / "input.json").read_text())
    results = [compute(source, scenario, args.library) for scenario in source["scenarios"]]
    output = json.dumps({"scenarios": results}, ensure_ascii=False, indent=2) + "\n"
    path = HERE / "result.json"
    if args.write:
        path.write_text(output)
    else:
        require(path.read_text() == output, "recorded result differs from verification")
    print("; ".join(f"{row['id']}: {row['summary']['assignedJobs']}/{row['summary']['requestedJobs']} assigned, "
                    f"{row['summary']['travelMinutes']} travel, {row['summary']['overtimeMinutes']} overtime; "
                    f"{row['native']['evaluated']} exact choices" for row in results))


if __name__ == "__main__":
    main()
