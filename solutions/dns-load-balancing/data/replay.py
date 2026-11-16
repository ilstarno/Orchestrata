"""Replay two bounded, independent traffic planning calculations offline."""
import argparse
from datetime import datetime, timedelta, timezone
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
TTL_SECONDS = 60
HEALTH_FRESHNESS_SECONDS = 30
MINIMUM_DNS_DOMAINS = 2
MARKET_WEIGHT = 100
OBSERVATION_BASE = datetime(2026, 11, 16, 12, tzinfo=timezone.utc)

REGIONS = {
    "us-east": [("US-A", "az-a", 180, 30), ("US-B", "az-b", 160, 20), ("US-C", "az-c", 70, 10)],
    "eu-west": [("EU-A", "az-a", 160, 30), ("EU-B", "az-b", 120, 20), ("EU-C", "az-c", 105, 15)],
    "ap-south": [("AP-A", "az-a", 110, 25), ("AP-B", "az-b", 100, 15), ("AP-C", "az-b", 55, 10)],
}
MARKETS = {
    "americas": [{"us-east": 1}, {"eu-west": 1}],
    "eu-resident": [{"eu-west": 1}],
    "eu-flex": [{"eu-west": 1}, {"us-east": 1}],
    "asia-pacific": [{"ap-south": 1}, {"us-east": 1}],
    "global": [{"us-east": 50, "eu-west": 35, "ap-south": 15}],
}
SCENARIOS = [
    ("normal", (225, 180, 105), {}, {}),
    ("peak", (420, 315, 240), {}, {"EU-C": ("healthy", 31), "AP-C": (None, None)}),
    ("region-unavailable", (390, 130, 130), {"eu-west": ("down", 2)}, {
        "AP-B": ("draining", 6), "AP-C": ("down", 3),
    }),
]


def timestamp(value):
    return value.isoformat(timespec="seconds").replace("+00:00", "Z")


def observation(now, state, age):
    return {"reportedState": state,
            "observedAtUtc": timestamp(now-timedelta(seconds=age)) if age is not None else None}


def health(now, report):
    raw, observed = report["reportedState"], report["observedAtUtc"]
    if raw not in {"healthy", "down", "draining"} or observed is None:
        return "unknown"
    age = (now - datetime.fromisoformat(observed)).total_seconds()
    if age < 0:
        return "unknown"
    return raw if age <= HEALTH_FRESHNESS_SECONDS else "stale"


def apportion(weights, total):
    """Allocate indivisible units by largest remainder with stable ID tie breaks."""
    if not weights:
        if total:
            raise ValueError("Cannot assign a positive total without eligible resources")
        return {}
    if any(weight <= 0 for weight in weights.values()) or total < 0:
        raise ValueError("Weights must be positive and totals cannot be negative")
    denominator = sum(weights.values())
    whole = {}
    remainders = []
    for identifier, weight in weights.items():
        whole[identifier], residual = divmod(total*weight, denominator)
        remainders.append((-residual, identifier))
    for _, identifier in sorted(remainders)[:total-sum(whole.values())]:
        whole[identifier] += 1
    return whole


def make_inputs(index, scenario):
    name, demands, region_signals, backend_signals = scenario
    now = OBSERVATION_BASE + timedelta(minutes=index*5)
    regions = []
    for (region, inventory), demand in zip(REGIONS.items(), demands, strict=True):
        replicas = [
            {
                "backendId": ident, "failureDomain": zone, "ratedCapacityRps": capacity,
                "requiredReserveRps": reserve,
                "healthObservation": observation(now, *backend_signals.get(ident, ("healthy", 8))),
            }
            for ident, zone, capacity, reserve in inventory
        ]
        regions.append({
            "regionId": region,
            "edgeObservation": observation(now, *region_signals.get(region, ("healthy", 4))),
            "offeredDemandRps": demand,
            "demandSource": "independent_regional_scenario",
            "backendInventory": replicas,
        })
    return {"scenarioId": name, "plannedAtUtc": timestamp(now), "regions": regions}


def load_split(region, now):
    edge_status = health(now, region["edgeObservation"])
    replicas = region["backendInventory"]
    state = {item["backendId"]:health(now, item["healthObservation"]) for item in replicas}
    usable = {item["backendId"]: item["ratedCapacityRps"]-item["requiredReserveRps"]
              for item in replicas if edge_status == "healthy" and state[item["backendId"]] == "healthy"
              and item["ratedCapacityRps"] > item["requiredReserveRps"]}
    for item in replicas:
        if not 0 <= item["requiredReserveRps"] <= item["ratedCapacityRps"]:
            raise ValueError(f"Invalid capacity or reserve: {item['backendId']}")
    demand = region["offeredDemandRps"]
    allocated = min(demand, sum(usable.values()))
    allocation = apportion(usable, allocated)
    backends = [{"id": item["backendId"], "status": state[item["backendId"]],
                 "failureDomain": item["failureDomain"], "capacity": item["ratedCapacityRps"],
                 "reserve": item["requiredReserveRps"], "usable": usable.get(item["backendId"], 0),
                 "allocated": allocation.get(item["backendId"], 0)}
                for item in replicas]
    return {"region": region["regionId"], "status": edge_status, "demand": demand,
            "allocated": allocated, "unserved": demand-allocated, "backends": backends,
            "healthyDomains": len({item["failureDomain"] for item in backends if item["usable"] > 0}),
            "usableCapacity": sum(usable.values()),
            "reason": ("region-unavailable" if edge_status != "healthy" else
                       "capacity-limit" if demand > allocated else "covered")}


def dns_steering(regions):
    choices = []
    eligible = {item["region"] for item in regions if item["status"] == "healthy" and
                item["healthyDomains"] >= MINIMUM_DNS_DOMAINS}
    for market, tiers in MARKETS.items():
        selected, tier_index = {}, None
        for idx, group in enumerate(tiers):
            allowed = {region: weight for region, weight in group.items() if region in eligible}
            if allowed:
                selected, tier_index = apportion(allowed, MARKET_WEIGHT), idx
                break
        choices.append({
            "market": market, "lookupWeight": MARKET_WEIGHT, "ttlSeconds": TTL_SECONDS,
            "routes": [{"region": region, "weightPercent": weight} for region, weight in selected.items()],
            "unassignedWeight": MARKET_WEIGHT-sum(selected.values()),
            "reason": "no-eligible-region" if tier_index is None else "preferred" if tier_index == 0 else "fallback",
        })
    return choices


def solve(scenario):
    now = datetime.fromisoformat(scenario["plannedAtUtc"])
    regions = [load_split(region, now) for region in scenario["regions"]]
    dns = dns_steering(regions)
    return {"id": scenario["scenarioId"], "dns": dns, "regions": regions, "metrics": {
        "dnsMarkets": len(dns),
        "dnsHeld": sum(item["unassignedWeight"] > 0 for item in dns),
        "offered": sum(item["demand"] for item in regions),
        "allocated": sum(item["allocated"] for item in regions),
        "unserved": sum(item["unserved"] for item in regions),
    }}


def outage_trace(inputs, result):
    outage = inputs["regions"][1]
    target = outage["regionId"]
    region_plan = next(item for item in result["regions"] if item["region"] == target)
    restricted = next(item for item in result["dns"] if item["market"] == "eu-resident")
    observation_audit = {
        "recordedSignal": outage["edgeObservation"],
        "freshnessEvaluation": {
            "maximumAgeSeconds": HEALTH_FRESHNESS_SECONDS, "computedStatus": region_plan["status"],
            "eligibilityGate": {
                "freshHealthyEdgeRequired": True,
                "minimumDistinctHealthyDomains": MINIMUM_DNS_DOMAINS,
                "healthyDomainsCounted": region_plan["healthyDomains"],
                "outcome": {
                    "admitted": False, "unassignedPolicyWeight": restricted["unassignedWeight"],
                    "reviewDisposition": {"publishAllowed": False, "action": "send_to_operator_review"},
                },
            },
        },
    }
    return {
        "scenarioId": inputs["scenarioId"], "reviewLayer": "dns_new_lookup_policy",
        "marketEvaluation": {
            "marketId": restricted["market"],
            "localityEvidence": {
                "hardAllowlist": [target], "crossRegionFallbackAllowed": False,
                "priorityTier": {
                    "tierIndex": 0, "candidate": {"regionId": target, "signalAudit": observation_audit},
                },
            },
        },
    }


def build():
    inputs = [make_inputs(idx, item) for idx, item in enumerate(SCENARIOS)]
    planned = [solve(scenario) for scenario in inputs]
    visual = {"scenarios": planned}
    operational = {
        "caseId": "regional-traffic-planning",
        "scope": {"planOnly": True, "networkCallsMade": False, "dnsWritesEnabled": False,
                  "loadBalancerWritesEnabled": False, "actualTrafficInferred": False,
                  "destinationIdsAreLabels": True, "perUserGeoInference": False},
        "policies": {
            "dnsPolicy": {
                "target": "new_dns_lookups", "recordTtlSeconds": TTL_SECONDS,
                "healthMaxAgeSeconds": HEALTH_FRESHNESS_SECONDS, "lookupWeightBudget": MARKET_WEIGHT,
                "requiredHealthyFailureDomains": MINIMUM_DNS_DOMAINS, "preferFirstEligibleTier": True,
                "unassignedRequiresReview": True,
                "markets": [{"marketId": market, "hardAllowlist": list(dict.fromkeys(key for tier in tiers for key in tier)),
                             "signedOffTiers": [{"tierIndex": idx, "destinations": [
                                 {"regionId": region, "configuredWeight": weight} for region, weight in group.items()
                             ]} for idx, group in enumerate(tiers)]} for market, tiers in MARKETS.items()],
            },
            "backendPolicy": {
                "units": "requests_per_second", "trafficSourceIndependentFromDns": True,
                "hardReserveRequired": True, "allocateOnlyWithinDemandRegion": True,
                "preferFullFeasibleCoverage": True, "ratioBasis": "usable_after_reserve",
                "roundingRule": "largest_fraction_then_backend_id",
                "residualMustBeVisible": True, "acceptUnknownHealth": False,
            },
        },
        "scenarioInputs": inputs,
        "scenarioOutput": planned,
        "decisionAudit": outage_trace(inputs[-1], planned[-1]),
    }
    return {"visual.json": visual, "operational-case.json": operational}


if __name__ == "__main__":
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("--write", action="store_true", help="regenerate the local JSON fixtures")
    parsed = args.parse_args()
    for name, content in build().items():
        target = HERE / name
        encoded = json.dumps(content, ensure_ascii=False, indent=2) + "\n"
        if parsed.write:
            target.write_text(encoded, encoding="utf-8")
        elif target.read_text(encoding="utf-8") != encoded:
            raise SystemExit(f"Replay differs: {name}")
    print("Verified normal, peak and region-unavailable DNS and capacity plans.")
