"""Deterministic, fixed-route Tirana dispatch calculation."""
import argparse
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
DEFAULT_MAP = HERE / "tirana-courier-streets.geojson"
RULES = [
    ("R01", "Mapped pickup and delivery must both match; sequence 2 precedes 3.", "company"),
    ("R02", "The route mode must be among the modes allowed for the shipment.", "company"),
    ("R03", "The courier must be available and their assigned asset in service.", "company"),
    ("R04", "At pickup, shipment plus current load cannot exceed payload capacity.", "company"),
    ("R05", "Wait if early; both pickup and delivery service must finish by each window's end.", "company"),
    ("R06", "The planned route must finish by the courier's shift end.", "company"),
    ("R07", "Only one request per fixed route; choose feasible requests by priority, then stable request ID.", "company"),
    ("R09", "Operator must be qualified for the requested handling; cargo must fit the configured internal dimensions.", "company"),
    ("R10", "Borrowing another team’s operator requires simulated cross-team approval.", "company"),
    ("R11", "Missing scenario road or curb evidence holds the request for review. Sample evidence is not a permit.", "company"),
    ("R08", "Travel follows the already mapped OSM profile; segment durations are estimates, not live traffic.", "map_model"),
]
CAP = [200000, 120000, 160000, 180000, 140000, 160000, 18000, 6000]
MASS = [24000, 62000, 18000, 37000, 84000, 21000, 5000, 1200]
MODE_ASSETS = {"vehicle": "Compact cargo van", "bicycle": "Cargo bicycle", "walking": "Insulated backpack"}
KINDS = ("start", "pickup", "delivery", "end")


def load_map(path):
    features = json.loads(path.read_text())["features"]
    routes = {}
    for feature in features:
        p = feature["properties"]
        route = routes.setdefault(p["courierId"], {"points": {}, "line": None})
        if feature["geometry"]["type"] == "LineString":
            assert route["line"] is None
            route["line"] = p
            route["geometry"] = feature["geometry"]["coordinates"]
        else:
            assert feature["geometry"]["type"] == "Point" and p["sequence"] not in route["points"]
            route["points"][p["sequence"]] = (p["kind"], feature["geometry"]["coordinates"], p["osmNodeId"], p["mode"])
    for cid, route in routes.items():
        points, line, geo = route["points"], route["line"], route["geometry"]
        assert list(sorted(points)) == [1, 2, 3, 4]
        assert [points[i][0] for i in range(1, 5)] == list(KINDS)
        assert all(point[-1] == line["mode"] for point in points.values())
        assert len(line["legs"]) == 3
        for i, leg in enumerate(line["legs"], 1):
            assert [leg["fromSequence"], leg["toSequence"]] == [i, i + 1]
            for seq, direction, index in [(i, "from", "Start"), (i + 1, "to", "End")]:
                assert points[seq][1] == geo[leg[f"vertex{index}Index"]] == leg[f"{direction}Coordinate"]
        assert sum(leg["durationSeconds"] for leg in line["legs"]) == line["durationSeconds"]
    assert list(sorted(routes)) == [f"TIA-{i:03}" for i in range(1, 9)]
    return routes


def build_input(routes):
    couriers, requests, hubs = [], [], []
    for i, (cid, route) in enumerate(sorted(routes.items()), 1):
        mode = route["line"]["mode"]
        hub = f"HUB-{i:02}"
        cell = "northwest" if i in (1, 4) else "southwest" if i in (2, 6) else "east" if i in (3, 5) else "central"
        hubs.append({"id": hub, "name": f"{cell.capitalize()} launch point {i:02}", "courierId": cid, "sequence": 1, "coordinate": route["points"][1][1]})
        couriers.append({
            "courierId": cid, "driverId": f"OP-{i:02}", "assetId": f"ASSET-{i:02}", "assetType": MODE_ASSETS[mode],
            "hubId": hub, "mode": mode, "available": True, "assetInService": True, "capacityGrams": CAP[i - 1],
            "teamId": "TEAM-W" if cell.endswith("west") else "TEAM-C", "experienceMonths": 15+i*4,
            "qualifiedHandling": ["standard"] + (["secure_handoff"] if i in (2, 5) else []),
            "interiorDimensionsMm": {"vehicle": [1400, 1100, 1000], "bicycle": [600, 400, 350], "walking": [400, 300, 200]}[mode],
            "startingLoadGrams": 0, "shiftStartSeconds": 8 * 3600 + (i - 1) * 60,
            "shiftEndSeconds": 8 * 3600 + (26 if i in (5, 7, 8) else 75) * 60,
        })
        requests.append({
            "requestId": f"REQ-{i:03}", "cargo": ["Printed forms", "Market crates", "Retail tote", "Office supplies",
                                                "Packaged pantry goods", "Repair parts", "Books", "Document pouch"][i - 1],
            "priority": 10, "teamId": ("TEAM-C" if i == 1 else couriers[-1]["teamId"]), "crossTeamApproved": i == 1,
            "handling": "secure_handoff" if i == 2 else "standard", "packedDimensionsMm": [300, 200, 150],
            "scenarioAccessEvidence": {"road": "sample_confirmed", "curb": "sample_confirmed"}, "allowedModes": [mode], "massGrams": MASS[i - 1],
            "pickupOsmNodeId": route["points"][2][2], "deliveryOsmNodeId": route["points"][3][2],
            "pickupWindowSeconds": [8 * 3600 + i * 60, 8 * 3600 + 20 * 60],
            "deliveryWindowSeconds": [8 * 3600 + (25 if i == 1 else 8) * 60, 8 * 3600 + 45 * 60],
            "pickupServiceSeconds": 90, "deliveryServiceSeconds": 120,
        })
    def alt(request_id, base, **changes):
        return requests[base - 1] | {"requestId": f"REQ-{request_id:03}", "priority": 5} | changes
    requests += [
        alt(9, 7, cargo="Oversize cartons", massGrams=19000),
        alt(10, 8, cargo="Asset parts", allowedModes=["vehicle"]),
        alt(11, 1, cargo="Missed handoff window", pickupWindowSeconds=[28800, 28860]),
        alt(12, 4, cargo="Unmapped address", deliveryOsmNodeId=None),
        alt(13, 6, cargo="Deferred batch", deliveryWindowSeconds=[33300, 33600]),
        alt(14, 2, cargo="Second request for same mapped stops"),
        alt(15, 4, cargo="Sealed collection", handling="secure_handoff"),
        alt(16, 7, cargo="Long package", packedDimensionsMm=[650, 200, 150]),
        alt(17, 1, cargo="Inter-team collection", crossTeamApproved=False),
        alt(18, 2, cargo="Pickup pending curb confirmation", priority=20, scenarioAccessEvidence={"road": "sample_confirmed", "curb": "unknown"}),
    ]
    return couriers, requests, hubs


def evaluate(courier, request, route):
    cid, mode = courier["courierId"], courier["mode"]
    errors = []
    if mode not in request["allowedModes"]:
        errors.append("mode_not_allowed")
    if not (courier["available"] and courier["assetInService"]):
        errors.append("courier_or_asset_unavailable")
    if request["handling"] not in courier["qualifiedHandling"]:
        errors.append("handling_qualification_missing")
    if any(a > b for a, b in zip(request["packedDimensionsMm"], courier["interiorDimensionsMm"], strict=True)):
        errors.append("cargo_dimensions_exceeded")
    if request["teamId"] != courier["teamId"] and not request["crossTeamApproved"]:
        errors.append("cross_team_approval_missing")
    if any(x != "sample_confirmed" for x in request["scenarioAccessEvidence"].values()):
        errors.append("access_evidence_missing")
    t, load, stops = courier["shiftStartSeconds"], courier["startingLoadGrams"], []
    kinds = [(None, 0, 0), ("pickup", request["massGrams"], request["pickupServiceSeconds"]),
             ("delivery", -request["massGrams"], request["deliveryServiceSeconds"]), (None, 0, 0)]
    for seq, (window, delta, service) in enumerate(kinds, 1):
        leg = route["line"]["legs"][seq - 2] if seq > 1 else None
        arrival = t + (leg["durationSeconds"] if leg else 0)
        start = max(arrival, request[f"{window}WindowSeconds"][0]) if window else arrival
        t, load = start + service, load + delta
        if window and t > request[f"{window}WindowSeconds"][1]:
            errors.append(f"{window}_window_missed")
        if not 0 <= load <= courier["capacityGrams"] and "capacity_exceeded" not in errors:
            errors.append("capacity_exceeded")
        kind, coordinate, node, _ = route["points"][seq]
        stops.append({
            "sequence": seq, "kind": kind, "coordinate": coordinate, "osmNodeId": node,
            "arrivalSeconds": arrival, "serviceStartSeconds": start, "departureSeconds": t,
            "waitSeconds": start - arrival, "loadGrams": load,
            "travelFromPreviousSeconds": leg["durationSeconds"] if leg else 0,
        })
    if t > courier["shiftEndSeconds"]:
        errors.append("shift_end_exceeded")
    return {"courierId": cid, "requestId": request["requestId"], "mode": mode, "stops": stops,
            "distanceMeters": route["line"]["distanceMeters"], "travelSeconds": route["line"]["durationSeconds"],
            "waitSeconds": sum(stop["waitSeconds"] for stop in stops), "totalSeconds": t - courier["shiftStartSeconds"],
            "finishSeconds": t, "peakLoadGrams": max(stop["loadGrams"] for stop in stops)}, errors


def calculate(routes):
    couriers, requests, hubs = build_input(routes)
    reserved, runs, decisions = set(), [], []
    for request in sorted(requests, key=lambda r: (-r["priority"], r["requestId"])):
        candidates = []
        chosen = None
        for courier in couriers:
            cid = courier["courierId"]
            route = routes[cid]
            if (route["points"][2][2], route["points"][3][2]) != (request["pickupOsmNodeId"], request["deliveryOsmNodeId"]):
                continue
            draft, errors = evaluate(courier, request, route)
            if not errors and cid in reserved:
                errors.append("fixed_route_already_committed")
            candidates.append({"courierId": cid, "reasonCodes": errors})
            if not errors and chosen is None:
                chosen = draft
        if chosen:
            reserved.add(chosen["courierId"])
            runs.append(chosen)
        decisions.append({"requestId": request["requestId"], "status": "assigned" if chosen else "not_assigned",
                          "reviewRequired": not chosen and any("access_evidence_missing" in c["reasonCodes"] for c in candidates),
                          "courierId": chosen["courierId"] if chosen else None,
                          "reasonCodes": [] if chosen else sorted({e for c in candidates for e in c["reasonCodes"]}) if candidates else ["no_mapped_pickup_delivery_pair"],
                          "candidates": candidates})
    return {
        "metadata": {"caseId": "TIA-FIXED-01", "organization": "Tirana City Parcel (illustrative dispatch data)", "timezone": "Europe/Tirane",
                     "clock": "Seconds since local midnight; illustrative day, no real roster.",
                     "roadSource": "OpenStreetMap contributors, offline Geofabrik Albania extract 2026-11-13",
                     "osmLicenseUrl": "https://www.openstreetmap.org/copyright",
                     "scope": "Eight fixed street geometries, 32 mapped example stops; no rerouting, live dispatch or global route optimization.",
                     "rulesScope": "Company rules are scenario assumptions. OSM profile/access is a routing model only; legal compliance and current road restrictions were not evaluated."},
        "organization": {"id": "ORG-TIA", "name": "Tirana City Parcel", "dataClass": "Illustrative dispatch data",
                         "function": "Local parcel collection and delivery",
                         "teams": [{"id": "TEAM-W", "function": "West launch points"}, {"id": "TEAM-C", "function": "Central/east launch points"}]},
        "externalReview": {
            "status": "not_verified", "requiredBeforeRealDispatch": True,
            "sources": [
                {"id": "SRC-ACCESS", "name": "Municipality of Tirana: ring circulation authorization", "url": "https://tirana.al/Bashkia-Tiran%C3%AB/detaje-sherbimi/Transporti/753", "check": "Verify whether the route, vehicle and time require authorization inside the city ring."},
                {"id": "SRC-CURB", "name": "Tirana Parking: on-street paid parking", "url": "https://tiranaparking.al/parkime-ne-rruge/", "check": "Parking tariffs do not confirm lawful loading; check actual signs and allowed handoff location."},
                {"id": "SRC-RESERVED", "name": "Tirana Parking: reserved spaces", "url": "https://tiranaparking.al/parkim-i-rezervuar-me-pagese/", "check": "Check any sign, activity, applicable hours and permission for the specific bay."},
                {"id": "SRC-LAW", "name": "Official Publications Centre: Road Code register", "url": "https://qbz.gov.al/eli/ligj/1998/07/22/8378", "check": "Registry reference only; the case does not determine which provisions apply."}
            ]
        },
        "rules": [{"id": i, "description": txt, "kind": typ} for i, txt, typ in RULES],
        "hubs": hubs, "couriers": couriers, "requests": requests, "decisions": decisions, "runs": runs,
        "summary": {"requestCount": len(requests), "assignedCount": len(runs), "notAssignedCount": len(requests)-len(runs),
                    "reviewCount": sum(d["reviewRequired"] for d in decisions), "routeCount": len(routes), "stopCount": sum(len(run["stops"]) for run in runs),
                    "distanceMeters": round(sum(run["distanceMeters"] for run in runs), 1),
                    "travelSeconds": sum(run["travelSeconds"] for run in runs), "waitSeconds": sum(run["waitSeconds"] for run in runs)},
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--map", type=Path, default=DEFAULT_MAP)
    args = parser.parse_args()
    print(json.dumps(calculate(load_map(args.map)), indent=2, ensure_ascii=False))
