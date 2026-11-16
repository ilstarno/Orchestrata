"""Validate and normalize the sample raw schema to native finite-domain JSON."""

from collections import Counter
from datetime import datetime
from math import prod
from zoneinfo import ZoneInfo


def require(condition, explanation):
    if not condition:
        raise ValueError(explanation)


def integer(number, minimum, maximum, field):
    require(type(number) is int and minimum <= number <= maximum, f"invalid {field}")
    return number


def keyset(record, names, field):
    require(isinstance(record, dict) and set(record) == set(names.split()), f"invalid {field} fields")


def period(data, timezone):
    keyset(data, "start end", "time window")
    endpoints = [datetime.fromisoformat(data[key]) for key in ("start", "end")]
    for dt in endpoints:
        require(dt.tzinfo is not None and dt.utcoffset() == dt.astimezone(timezone).utcoffset(),
                "window offset does not match dataset time zone")
    require(endpoints[0] < endpoints[1], "time window must have positive length")
    return endpoints


def mapping_ratings(data, field):
    require(isinstance(data, dict) and data and all(isinstance(k, str) and k for k in data),
            f"invalid {field} keys")
    for rating in data.values():
        integer(rating, 1, 5, field)


def validated(raw):
    keyset(raw, "schema_version dataset_id classification timezone coverage_window metadata policy queues operators", "dataset")
    require(raw["schema_version"] == 1 and raw["classification"] == "sample", "unsupported dataset contract")
    metadata = raw["metadata"]
    keyset(metadata, "generated_by observed_data purpose rating_scale units", "metadata")
    require(metadata["observed_data"] is False, "observed records are not supported")
    keyset(metadata["rating_scale"], "1 2 3 4 5", "rating scale")
    keyset(metadata["units"], "probability handling_time experience arrivals contact_capacity", "units")
    timezone = ZoneInfo(raw["timezone"])
    start, end = period(raw["coverage_window"], timezone)
    p = raw["policy"]
    keyset(p, "max_queues_per_operator evaluation_budget voice_occupancy_target_basis_points demand_scenarios_basis_points", "policy")
    require(p["max_queues_per_operator"] == 1, "this model requires one queue per operator")
    integer(p["evaluation_budget"], 1, 100000, "evaluation budget")
    integer(p["voice_occupancy_target_basis_points"], 1, 10000, "occupancy target")
    demand = p["demand_scenarios_basis_points"]
    keyset(demand, "typical surge", "demand scenarios")
    require(sum(integer(v, 0, 10000, "probability") for v in demand.values()) == 10000,
            "demand probabilities must sum to 10000 basis points")
    queues = raw["queues"]
    require(isinstance(queues, list) and 5 <= len(queues) <= 7, "expected five to seven queues")
    for q in queues:
        keyset(q, "id label channel required_agents skill language minimum_role_months soft_preference_weight arrival_counts average_handle_seconds", "queue")
        require(q["channel"] == "voice" and q["required_agents"] == 1, "each queue must need one voice seat")
        keyset(q["skill"], "id minimum_rating", "required skill")
        keyset(q["language"], "tag minimum_rating", "required language")
        for field in ("skill", "language"):
            integer(q[field]["minimum_rating"], 1, 5, field + " threshold")
        integer(q["minimum_role_months"], 0, 1000, "experience")
        integer(q["soft_preference_weight"], 1, 1000, "soft preference weight")
        integer(q["average_handle_seconds"], 1, 3600, "average handle time")
        keyset(q["arrival_counts"], "typical surge", "queue arrivals")
        for arrivals in q["arrival_counts"].values():
            integer(arrivals, 0, 1000, "arrivals")
        require(q["arrival_counts"]["typical"] <= q["arrival_counts"]["surge"], "surge must not be below typical")
    for name, rows in (("queue", queues), ("operator", raw["operators"])):
        ids = [row["id"] for row in rows]
        require(all(isinstance(i, str) and i for i in ids) and len(set(ids)) == len(ids),
                f"{name} IDs must be nonempty and unique")
    require(len(raw["operators"]) >= 50, "expected at least 50 operators")
    availability = []
    for op in raw["operators"]:
        keyset(op, "id skills languages role_experience_months shift breaks contact_capacity", "operator")
        mapping_ratings(op["skills"], "skill"); mapping_ratings(op["languages"], "language")
        integer(op["role_experience_months"], 0, 1000, "experience")
        cap = op["contact_capacity"]
        keyset(cap, "voice chat", "contact capacity")
        integer(cap["voice"], 0, 1, "voice concurrency"); integer(cap["chat"], 0, 10, "chat concurrency")
        shift = period(op["shift"], timezone)
        breaks = [period(entry, timezone) for entry in op["breaks"]]
        require(all(shift[0] <= bs < be <= shift[1] for bs, be in breaks), "break outside shift")
        availability.append((shift[0] <= start and end <= shift[1],
                             any(start < be and bs < end for bs, be in breaks)))
    return start, end, availability


def screen(queue, operator, availability):
    covers, on_break = availability
    skill = operator["skills"].get(queue["skill"]["id"], 0)
    language = operator["languages"].get(queue["language"]["tag"], 0)
    months = operator["role_experience_months"]
    capacity = operator["contact_capacity"][queue["channel"]]
    reasons = [
        reason for reason, excluded in (
            ("shift_does_not_cover_window", not covers),
            ("break_overlaps_window", on_break),
            ("no_voice_capacity", capacity < 1),
            ("required_skill_below_minimum", skill < queue["skill"]["minimum_rating"]),
            ("required_language_below_minimum", language < queue["language"]["minimum_rating"]),
            ("experience_below_minimum", months < queue["minimum_role_months"]),
        ) if excluded
    ]
    return reasons, {"skill_rating": skill, "language_rating": language, "experience_months": months,
                     "voice_contact_capacity": capacity, "shift_covers_window": covers,
                     "break_overlaps_window": on_break}


def normalize(raw):
    start, end, availability = validated(raw)
    variables, checks, prefs = [], [], []
    for q in raw["queues"]:
        eligible, excluded, evidence = [], {}, {}
        for index, (op, covered) in enumerate(zip(raw["operators"], availability, strict=True)):
            reasons, facts = screen(q, op, covered)
            if reasons:
                excluded[op["id"]] = reasons
            else:
                eligible.append(index)
                evidence[index] = facts
        require(eligible, f"queue {q['id']} has no eligible operators")
        preferred = min(eligible, key=lambda i: (
            -evidence[i]["skill_rating"], -evidence[i]["language_rating"],
            -evidence[i]["experience_months"], raw["operators"][i]["id"],
        ))
        prefs.append({"kind": "prefer_value", "variable": len(variables), "value": preferred,
                      "soft_weight": q["soft_preference_weight"]})
        variables.append({"name": q["id"], "domain": eligible})
        counts = Counter(reason for reasons in excluded.values() for reason in reasons)
        checks.append({"eligible": eligible, "excluded": excluded, "reason_counts": dict(sorted(counts.items())),
                       "evidence": evidence, "preferred": preferred})
    combinations = prod(len(v["domain"]) for v in variables)
    require(combinations <= raw["policy"]["evaluation_budget"], "model exceeds exact search budget")
    return {"variables": variables, "constraints": [
        {"kind": "all_different", "variables": list(range(len(variables))), "hard_weight": 1},
        *prefs,
    ]}, checks, int((end - start).total_seconds()), combinations
