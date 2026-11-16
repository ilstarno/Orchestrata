"""Offline comparison of generic gate and ground-crew decisions."""
from pathlib import Path
from itertools import product
import argparse
import json

ROOT = Path(__file__).resolve().parent
BUFFER = 5
GATES = [
    {"id": "G01", "size": "single", "kind": "contact", "international": False, "zone": "west"},
    {"id": "G02", "size": "wide", "kind": "contact", "international": True, "zone": "west"},
    {"id": "G03", "size": "single", "kind": "contact", "international": True, "zone": "east"},
    {"id": "R01", "size": "single", "kind": "remote", "international": True, "zone": "east"},
]
FLIGHTS = [
    {"id": f"OP-{i + 1:02}", "start": start, "end": end, "size": size,
     "international": intl, "passengers": pax, "needsStepFreeContact": contact}
    for i, (start, end, size, intl, pax, contact) in enumerate([
        (0, 45, "single", True, 140, False),
        (5, 50, "wide", True, 260, True),
        (10, 50, "single", False, 115, False),
        (20, 60, "single", True, 160, False),
        (50, 95, "single", False, 108, True),
        (55, 100, "wide", True, 248, False),
        (65, 110, "single", True, 175, False),
        (75, 115, "single", False, 90, False),
        (82, 120, "single", True, 102, True),
    ])
]
CREWS = [
    {"id": "RAMP-A", "skills": ["unload", "load"], "availability": [[0, 41], [53, 120]], "homeZone": "west"},
    {"id": "RAMP-B", "skills": ["unload", "load"], "availability": [[4, 78], [92, 120]], "homeZone": "east"},
    {"id": "MULTI-A", "skills": ["unload", "load", "cabin"], "availability": [[5, 120]], "homeZone": "west"},
    {"id": "CABIN-A", "skills": ["cabin"], "availability": [[0, 35], [47, 120]], "homeZone": "east"},
    {"id": "SERVICE-A", "skills": ["service"], "availability": [[0, 40], [51, 120]], "homeZone": "west"},
    {"id": "SERVICE-B", "skills": ["service"], "availability": [[8, 120]], "homeZone": "east"},
]
GATE_BY_ID = {g["id"]: g for g in GATES}
FLIGHT_BY_ID = {f["id"]: f for f in FLIGHTS}


def separated(a, b, gap):
    return a["end"] + gap <= b["start"] or b["end"] + gap <= a["start"]


def compatible(flight, gate):
    return ((flight["size"] != "wide" or gate["size"] == "wide")
            and (not flight["international"] or gate["international"])
            and (not flight["needsStepFreeContact"] or gate["kind"] == "contact"))


def gate_ok(board, flight, gate):
    return compatible(flight, gate) and all(
        other_gate != gate["id"] or separated(flight, other, BUFFER) for other, other_gate in board
    )


def gate_scores(board):
    unplaced = sum(1 for _, gate in board if gate is None)
    remote = sum(f["passengers"] for f, gate in board if gate and GATE_BY_ID[gate]["kind"] == "remote")
    return unplaced, remote


def gate_plan(proposed):
    if not proposed:
        board = []
        for f in FLIGHTS:
            selected = next((g["id"] for g in GATES if gate_ok(board, f, g)), None)
            board.append((f, selected))
        return board
    candidates = [[None] + [g["id"] for g in GATES if compatible(f, g)] for f in FLIGHTS]
    best = None
    for assignment in product(*candidates):
        board = []
        for f, gid in zip(FLIGHTS, assignment):
            if gid and not gate_ok(board, f, GATE_BY_ID[gid]):
                break
            board.append((f, gid))
        else:
            score = gate_scores(board)
            key = score + (tuple(g or "ZZZ" for _, g in board),)
            if best is None or key < best[0]:
                best = (key, board)
    return best[1]


def gate_rows(board):
    return [{"flight": f["id"], "gate": gate, "start": f["start"], "end": f["end"],
             "status": "held" if gate is None else GATE_BY_ID[gate]["kind"]} for f, gate in board]


def crew_board(gates, flexible):
    occupied = {c["id"]: [] for c in CREWS}
    result = []
    for f, gate in sorted(gates, key=lambda item: (item[0]["end"], item[0]["start"], item[0]["id"])):
        zone = GATE_BY_ID[gate]["zone"] if gate else None
        prior = {}
        for name, duration, earliest, parents in (
            ("unload", 7, 2, ()), ("cabin", 10, 5, ("unload",)),
            ("service", 8, 8, ()), ("load", 8, 13, ("unload", "cabin")),
        ):
            row = {"flight": f["id"], "task": name, "crew": None, "gate": gate,
                   "start": None, "end": None, "status": "held"}
            if not gate:
                row["reason"] = "gate_unassigned"
            elif any(prior.get(parent) is None for parent in parents):
                row["reason"] = "prerequisite_unstaffed"
            else:
                lower = max([f["start"] + earliest] + [prior[parent] for parent in parents])
                options = []
                for c in CREWS:
                    if name not in c["skills"] or (not flexible and c["homeZone"] != zone):
                        continue
                    for start in range(lower, f["end"] - BUFFER - duration + 1):
                        end = start + duration
                        if not any(a <= start and end <= b for a, b in c["availability"]):
                            continue
                        interval = {"start": start, "end": end}
                        if all(separated(interval, old, 4 if old["zone"] != zone else 0)
                               for old in occupied[c["id"]]):
                            options.append((start, c["homeZone"] != zone, c["id"], end))
                            break
                if options:
                    start, _, cid, end = min(options)
                    row.update(crew=cid, start=start, end=end, status="staffed")
                    occupied[cid].append({"start": start, "end": end, "zone": zone})
                else:
                    row["reason"] = "no_qualified_crew_in_window"
            prior[name] = row["end"]
            result.append(row)
    return result


def metrics(board, staff):
    gate_held, remote_passengers = gate_scores(board)
    ready = sum(all(row["status"] == "staffed" for row in staff if row["flight"] == f["id"]) for f in FLIGHTS)
    return {"gatesAssigned": len(board) - gate_held, "gatesHeld": gate_held, "remotePassengers": remote_passengers,
            "tasksStaffed": sum(row["status"] == "staffed" for row in staff), "tasksTotal": len(staff),
            "turnsReady": ready, "flightCount": len(board)}


def validate(board, staff):
    assert [f for f, _ in board] == FLIGHTS
    for f, g in board:
        if g:
            assert gate_ok([(other, other_gate) for other, other_gate in board if other != f], f, GATE_BY_ID[g])
    crew_directory = {c["id"]: c for c in CREWS}
    gate_assignments = dict((f["id"], g) for f, g in board)
    assert len({(r["flight"], r["task"]) for r in staff}) == len(FLIGHTS) * 4
    for row in staff:
        assert row["gate"] == gate_assignments[row["flight"]]
        if row["status"] == "held":
            assert row["crew"] is row["start"] is row["end"] is None
            continue
        f = FLIGHT_BY_ID[row["flight"]]
        crew = crew_directory[row["crew"]]
        assert f["start"] <= row["start"] < row["end"] <= f["end"] - BUFFER
        assert row["task"] in crew["skills"]
        assert any(start <= row["start"] < row["end"] <= stop for start, stop in crew["availability"])
        predecessors = {"cabin": ["unload"], "load": ["unload", "cabin"]}.get(row["task"], [])
        for parent in predecessors:
            prev = next(r for r in staff if r["flight"] == row["flight"] and r["task"] == parent)
            assert prev["end"] is not None and prev["end"] <= row["start"]
        for other in staff:
            if row != other and row["crew"] == other["crew"]:
                assert separated(row, other, 4 if GATE_BY_ID[row["gate"]]["zone"] != GATE_BY_ID[other["gate"]]["zone"] else 0)


def build():
    traffic = json.loads((ROOT / "traffic.json").read_text())
    assert {r["code"] for r in traffic} == {"JFK", "CDG", "MUC", "FRA", "TIA", "FCO", "LHR"}
    assert all(r["year"] == 2025 and r["passengers"] > 10000000 for r in traffic)
    before, after = (gate_plan(proposed) for proposed in (False, True))
    baseline, proposed = crew_board(before, False), crew_board(after, True)
    for board, staff in ((before, baseline), (after, proposed)):
        validate(board, staff)
    scores = dict(baseline=metrics(before, baseline), proposed=metrics(after, proposed))
    assert scores["proposed"]["gatesAssigned"] > scores["baseline"]["gatesAssigned"]
    assert scores["proposed"]["tasksStaffed"] > scores["baseline"]["tasksStaffed"]
    view = {"traffic": traffic, "metrics": scores, "scenes": [
        {"id": "gates", "baseline": gate_rows(before), "proposed": gate_rows(after)},
        {"id": "turnaround", "baseline": baseline, "proposed": proposed},
    ]}
    sample = next(r for r in proposed if r["task"] == "load" and r["status"] == "staffed")
    crew = next(c for c in CREWS if c["id"] == sample["crew"])
    f = FLIGHT_BY_ID[sample["flight"]]
    evidence = {
        "modeledFacility": "generic-terminal", "window": {
            "clockStart": "09:00", "windowMinutes": 120, "dispatch": {
                "flightReference": f["id"], "gateAllocation": {
                    "gateReference": sample["gate"], "turnaroundReview": {
                        "reviewMode": "recommendation_only", "serviceWorkOrder": {
                            "activityCode": sample["task"], "plannedInterval": {
                                "offsetStart": sample["start"], "offsetEnd": sample["end"], "qualificationProof": {
                                    "requiredSkill": sample["task"], "assignedCrew": {
                                        "crewId": crew["id"], "availabilityProof": {
                                            "availabilityWindows": crew["availability"], "releaseContext": {
                                                "endBeforeOffset": f["end"] - BUFFER, "markReadyOnlyIfAllStaffed": True,
                                            },
                                        },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        },
    }
    case = {"id": "generic-airport-gates-and-turnaround", "publicTrafficContext": traffic,
            "caseScope": {"liveFeed": False, "realGateLayout": False, "usesPassengerTotalsAsDemand": False,
                          "controlSystemWrites": False, "purpose": "human-reviewed planning example"},
            "planningInputs": {"flights": FLIGHTS, "gates": GATES, "crewRoster": CREWS,
                               "gateSeparationMinutes": BUFFER, "interZoneTransferMinutes": 4,
                               "proposedObjectives": ["minimize held flights", "minimize travelers on remote stands"],
                               "crewDecision": "earliest eligible crew per required task in departure order"},
            "checkedEvidence": evidence, "comparison": view["scenes"], "summary": scores}
    return {"visual.json": view, "operational-case.json": case}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    results = build()
    for name, data in results.items():
        path = ROOT / name
        body = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
        if args.write:
            path.write_text(body)
        elif path.read_text() != body:
            raise SystemExit(f"Replayed result differs: {name}")
    print(json.dumps(results["visual.json"]["metrics"]))


if __name__ == "__main__":
    main()
