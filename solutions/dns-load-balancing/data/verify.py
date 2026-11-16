"""Audit the published routing and capacity decisions without calling the generator."""
from datetime import datetime, timedelta
from fractions import Fraction
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent


def load(name):
    return json.loads((HERE / name).read_text(encoding="utf-8"))


def indexed(items, key):
    result = {item[key]: item for item in items}
    assert len(result) == len(items), f"Duplicate {key}"
    return result


def verify_utc(value):
    stamp = datetime.fromisoformat(value)
    assert stamp.utcoffset() == timedelta(0), value
    return stamp


def observation_status(observation, now, expiry):
    state, stamp = observation["reportedState"], observation["observedAtUtc"]
    if state not in ("healthy", "draining", "down") or stamp is None:
        return "unknown"
    age = now - verify_utc(stamp)
    if age < timedelta(0):
        return "unknown"
    return "stale" if age > timedelta(seconds=expiry) else state


def proportional_targets(resources, total):
    if not resources:
        assert total == 0
        return {}
    exact = {label: Fraction(total*amount, sum(resources.values())) for label, amount in resources.items()}
    result = {label: fraction.numerator // fraction.denominator for label, fraction in exact.items()}
    order = sorted(exact, key=lambda label: (result[label]-exact[label], label))
    for label in order[:total-sum(result.values())]:
        result[label] += 1
    return result


def audit_regional(source, row, now, freshness):
    edge_health = observation_status(source["edgeObservation"], now, freshness)
    assert row["status"] == edge_health
    assert row["demand"] == source["offeredDemandRps"] >= 0
    assert source["demandSource"] == "independent_regional_scenario"
    originals = indexed(source["backendInventory"], "backendId")
    outputs = indexed(row["backends"], "id")
    assert outputs.keys() == originals.keys()
    usable = {}
    domains = set()
    for label, original in originals.items():
        item = outputs[label]
        status = observation_status(original["healthObservation"], now, freshness)
        assert item["failureDomain"] == original["failureDomain"]
        assert item["status"] == status
        assert item["capacity"] == original["ratedCapacityRps"]
        assert item["reserve"] == original["requiredReserveRps"]
        assert 0 <= item["reserve"] <= item["capacity"]
        cap = item["capacity"]-item["reserve"]
        available = cap if edge_health == status == "healthy" else 0
        assert item["usable"] == available
        assert 0 <= item["allocated"] <= available
        if available:
            usable[label] = available
            domains.add(item["failureDomain"])
    assert row["usableCapacity"] == sum(usable.values())
    assert row["healthyDomains"] == len(domains)
    assert row["allocated"] == sum(item["allocated"] for item in outputs.values())
    assert row["allocated"] == min(row["usableCapacity"], row["demand"])
    assert row["unserved"] == row["demand"]-row["allocated"]
    assert row["reason"] == ("region-unavailable" if edge_health != "healthy" else
                             "capacity-limit" if row["unserved"] else "covered")
    targets = proportional_targets(usable, row["allocated"])
    assert all(outputs[label]["allocated"] == amount for label, amount in targets.items())


def audit_dns(contract, row, viable, ttl, budget):
    assert row["lookupWeight"] == budget and row["ttlSeconds"] == ttl
    assert sum(item["weightPercent"] for item in row["routes"]) + row["unassignedWeight"] == budget
    routes = {item["region"]: item["weightPercent"] for item in row["routes"]}
    assert len(routes) == len(row["routes"])
    assert routes.keys() <= (set(contract["hardAllowlist"]) & viable)
    tiers = contract["signedOffTiers"]
    allowed = set(contract["hardAllowlist"])
    assert allowed == {choice["regionId"] for tier in tiers for choice in tier["destinations"]}
    tier = next((entry for entry in tiers if any(choice["regionId"] in viable for choice in entry["destinations"])), None)
    if tier is None:
        assert row["unassignedWeight"] == budget and not routes
        assert row["reason"] == "no-eligible-region"
    else:
        weights = {item["regionId"]: item["configuredWeight"] for item in tier["destinations"] if item["regionId"] in viable}
        assert routes == proportional_targets(weights, budget)
        assert row["unassignedWeight"] == 0
        assert row["reason"] == ("preferred" if tier["tierIndex"] == 0 else "fallback")


def audit(case, visual):
    assert all(case["scope"][flag] is False for flag in (
        "networkCallsMade", "dnsWritesEnabled", "loadBalancerWritesEnabled", "actualTrafficInferred", "perUserGeoInference"
    ))
    assert case["scope"]["planOnly"] is True
    inputs = indexed(case["scenarioInputs"], "scenarioId")
    result = indexed(visual["scenarios"], "id")
    assert set(inputs) == set(result) == {"normal", "peak", "region-unavailable"}
    assert case["scenarioOutput"] == visual["scenarios"]
    dns_contract = case["policies"]["dnsPolicy"]
    assert dns_contract["target"] == "new_dns_lookups"
    assert case["policies"]["backendPolicy"]["trafficSourceIndependentFromDns"] is True
    known = indexed(dns_contract["markets"], "marketId")
    for label, replay in result.items():
        original = inputs[label]
        now = verify_utc(original["plannedAtUtc"])
        source_regions = indexed(original["regions"], "regionId")
        planned_regions = indexed(replay["regions"], "region")
        assert source_regions.keys() == planned_regions.keys()
        for name, region in planned_regions.items():
            audit_regional(source_regions[name], region, now, dns_contract["healthMaxAgeSeconds"])
        viable = {name for name, region in planned_regions.items() if region["status"] == "healthy"
                  and region["healthyDomains"] >= dns_contract["requiredHealthyFailureDomains"]}
        routing = indexed(replay["dns"], "market")
        assert routing.keys() == known.keys()
        for market, decision in routing.items():
            audit_dns(known[market], decision, viable, dns_contract["recordTtlSeconds"], dns_contract["lookupWeightBudget"])
        held = sum(decision["unassignedWeight"] == dns_contract["lookupWeightBudget"] for decision in routing.values())
        metrics = replay["metrics"]
        assert metrics["dnsMarkets"] == len(routing)
        assert metrics["dnsHeld"] == held
        for metric, source in (("offered", "demand"), ("allocated", "allocated"), ("unserved", "unserved")):
            assert metrics[metric] == sum(row[source] for row in planned_regions.values())
    outage = result["region-unavailable"]
    ap_south = next(region for region in outage["regions"] if region["region"] == "ap-south")
    assert ap_south["allocated"] > 0 and all(row["region"] != "ap-south"
                                            for market in outage["dns"] for row in market["routes"])
    eu_resident = next(market for market in outage["dns"] if market["market"] == "eu-resident")
    assert eu_resident["unassignedWeight"] == dns_contract["lookupWeightBudget"]


if __name__ == "__main__":
    audit(load("operational-case.json"), load("visual.json"))
    print("Independent audit verified both calculations, locality, fresh health, reserve, counts and unassigned demand.")
