"""Verify serialized worklists by minute enumeration, independently of the allocator."""
import json
from pathlib import Path

HERE=Path(__file__).resolve().parent
def read(name):
    return json.loads((HERE/name).read_text())


def verf(case,v):
    assert v["id"]==case["id"]
    occupied={a["resourceId"]:set() for a in case["resources"]}
    asset=case["sharedAsset"]
    asset_minutes={m for start,end in asset["unavailable"] for m in range(start,end)}
    finish={}
    assert len(v["decisions"])==len(case["work"])
    for task,d in zip(case["work"],v["decisions"],strict=True):
        assert task["workId"]==d["workId"]
        requires=task["requiresAsset"]
        possible=[]
        deps=task["prerequisiteIds"]
        available=all(p in finish for p in deps)
        for r in case["resources"]:
            key=r["resourceId"]
            if not available or task["requiredCapability"] not in r["capabilities"]:
                continue
            for begin in range(task["window"][0],task["window"][1]):
                end=begin+task["durationMinutes"]
                interval=set(range(begin,end))
                if (end>task["window"][1] or begin<r["window"][0] or end>r["window"][1]
                    or occupied[key]&interval or any(begin<finish[p] for p in deps)):
                    continue
                if requires and (begin<asset["window"][0] or end>asset["window"][1] or asset_minutes&interval):
                    continue
                possible.append((end,key,begin))
        if task["entryEvidence"]!="asserted_in_sample":
            assert d["status"]=="held" and d["holdCode"]=="required_evidence_missing"
        elif not available:
            assert d["status"]=="held" and d["holdCode"]=="prerequisite_held"
        elif not possible:
            assert d["status"]=="held" and d["holdCode"]=="no_resource_window"
        else:
            assert d["status"]=="scheduled"
            actual=d["endMinute"],d["resourceId"],d["startMinute"]
            assert actual==min(possible)
            assert d["assetId"]==(asset["assetId"] if requires else None)
            used=set(range(d["startMinute"],d["endMinute"]))
            occupied[d["resourceId"]].update(used)
            if requires:
                asset_minutes.update(used)
            finish[d["workId"]]=d["endMinute"]
    assert v["summary"]=={"requested":len(case["work"]),"scheduled":len(finish),"held":len(case["work"])-len(finish)}


cases=read("inputs.json")["cases"]
scenarios=read("visual.json")["scenarios"]
assert len(cases)==len(scenarios)==6
for case,view in zip(cases,scenarios,strict=True):
    verf(case,view)
    print(view["id"],view["summary"],"PASS")
assert sum(s["summary"]["requested"] for s in scenarios)==31
assert sum(s["summary"]["scheduled"] for s in scenarios)==20
