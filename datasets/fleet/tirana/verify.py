"""Independent checks of serialized decisions, matched route vertices and timelines."""
from pathlib import Path
import json

here = Path(__file__).resolve().parent
d = json.loads((here / "tirana-dispatch.json").read_text())
geo = json.loads((here / "tirana-courier-streets.geojson").read_text())
lines = {}
points = {}
for f in geo["features"]:
    p = f["properties"]
    if "sequence" in p:
        points[(p["courierId"], p["sequence"])] = f
    else:
        lines[p["courierId"]] = f
requests = {r["requestId"]: r for r in d["requests"]}
people = {c["courierId"]: c for c in d["couriers"]}
decisions = {j["requestId"]: j for j in d["decisions"]}
runs = {v["courierId"]: v for v in d["runs"]}
assert set(runs) == set(people) == set(lines)
assert set(requests) == set(decisions)
assert len(points) == d["summary"]["stopCount"] == 32
assert len(runs) == d["summary"]["assignedCount"] == 8
assert len(requests) == d["summary"]["requestCount"] == 18
assert d["summary"]["notAssignedCount"] == 10 and d["summary"]["reviewCount"] == 1

for cid, run in runs.items():
    req, c = requests[run["requestId"]], people[cid]
    assert decisions[req["requestId"]]["courierId"] == cid
    assert decisions[req["requestId"]]["status"] == "assigned"
    assert not decisions[req["requestId"]]["reasonCodes"]
    coords = lines[cid]["geometry"]["coordinates"]
    prop = lines[cid]["properties"]
    start, load, out, end = run["stops"]
    assert (load["osmNodeId"], out["osmNodeId"]) == (req["pickupOsmNodeId"], req["deliveryOsmNodeId"])
    dep0 = c["shiftStartSeconds"]
    assert start["arrivalSeconds"] == start["departureSeconds"] == dep0
    leg0, leg1, leg2 = (leg["durationSeconds"] for leg in prop["legs"])
    expected_pick = max(dep0 + leg0, req["pickupWindowSeconds"][0])
    expected_drop = max(expected_pick + req["pickupServiceSeconds"] + leg1, req["deliveryWindowSeconds"][0])
    assert load["serviceStartSeconds"] == expected_pick
    assert out["serviceStartSeconds"] == expected_drop
    assert load["departureSeconds"] == expected_pick + req["pickupServiceSeconds"] <= req["pickupWindowSeconds"][1]
    assert out["departureSeconds"] == expected_drop + req["deliveryServiceSeconds"] <= req["deliveryWindowSeconds"][1]
    assert end["departureSeconds"] == out["departureSeconds"] + leg2 <= c["shiftEndSeconds"]
    assert [s["loadGrams"] for s in run["stops"]] == [0, req["massGrams"], 0, 0]
    assert req["massGrams"] <= c["capacityGrams"]
    assert c["mode"] in req["allowedModes"] and c["available"] and c["assetInService"]
    assert req["handling"] in c["qualifiedHandling"]
    assert all(a <= b for a, b in zip(req["packedDimensionsMm"], c["interiorDimensionsMm"], strict=True))
    assert c["teamId"] == req["teamId"] or req["crossTeamApproved"]
    assert req["scenarioAccessEvidence"] == {"road": "sample_confirmed", "curb": "sample_confirmed"}
    for a, b, leg in zip(run["stops"][:-1], run["stops"][1:], prop["legs"], strict=True):
        for item in (a, b):
            point = points[(cid, item["sequence"])]
            assert item["coordinate"] == point["geometry"]["coordinates"]
            assert item["osmNodeId"] == point["properties"]["osmNodeId"]
            assert item["kind"] == point["properties"]["kind"]
        assert coords[leg["vertexStartIndex"]] == a["coordinate"] and coords[leg["vertexEndIndex"]] == b["coordinate"]
        assert b["arrivalSeconds"] == a["departureSeconds"] + leg["durationSeconds"]
        assert b["waitSeconds"] == b["serviceStartSeconds"] - b["arrivalSeconds"] >= 0

expected = {
    9: "capacity_exceeded", 10: "mode_not_allowed", 11: "pickup_window_missed",
    12: "no_mapped_pickup_delivery_pair", 13: "shift_end_exceeded",
    14: "fixed_route_already_committed", 15: "handling_qualification_missing",
    16: "cargo_dimensions_exceeded", 17: "cross_team_approval_missing", 18: "access_evidence_missing",
}
for i, reason in expected.items():
    decision = decisions[f"REQ-{i:03d}"]
    assert decision["status"] == "not_assigned" and decision["courierId"] is None
    assert decision["reasonCodes"] == [reason] and decision["reviewRequired"] == (i == 18)
get = lambda i: requests[f"REQ-{i:03d}"]
assert get(9)["massGrams"] > people["TIA-007"]["capacityGrams"]
assert people["TIA-008"]["mode"] not in get(10)["allowedModes"]
assert get(11)["pickupWindowSeconds"][1] < runs["TIA-001"]["stops"][1]["arrivalSeconds"] + get(11)["pickupServiceSeconds"]
assert all(get(12)["deliveryOsmNodeId"] != p["properties"]["osmNodeId"] for p in points.values())
assert get(13)["deliveryWindowSeconds"][0] + get(13)["deliveryServiceSeconds"] > people["TIA-006"]["shiftEndSeconds"]
assert get(14)["priority"] < get(2)["priority"] and runs["TIA-002"]["requestId"] == "REQ-002"
assert get(15)["handling"] not in people["TIA-004"]["qualifiedHandling"]
assert get(16)["packedDimensionsMm"][0] > people["TIA-007"]["interiorDimensionsMm"][0]
assert not get(17)["crossTeamApproved"] and get(17)["teamId"] != people["TIA-001"]["teamId"]
assert get(18)["priority"] > get(2)["priority"] and get(18)["scenarioAccessEvidence"]["curb"] == "unknown"
assert d["externalReview"]["status"] == "not_verified"
assert d["summary"]["travelSeconds"] == sum(l["properties"]["durationSeconds"] for l in lines.values()) == 5074
assert d["summary"]["waitSeconds"] == sum(s["waitSeconds"] for r in runs.values() for s in r["stops"]) == 1082
print("Verified 8 runs × 4 street stops, 18 decisions (10 exceptions; 1 review), 24 exact mapped leg times.")
