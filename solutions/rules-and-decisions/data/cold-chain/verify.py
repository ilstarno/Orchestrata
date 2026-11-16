"""Verify the published cold-chain instance with independent Cartesian search."""
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from itertools import combinations, product
from pathlib import Path
import json

ROOT=Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT/name).read_text(encoding="utf-8"))


def timestamp(date,offset):
    return (datetime.fromisoformat(date).replace(tzinfo=timezone.utc)+timedelta(minutes=offset)).isoformat().replace("+00:00","Z")


def by_id(rows,key):
    index={row[key]:row for row in rows}
    assert len(index)==len(rows)
    return index


def eligible(request,cabin,window,departure,event):
    if cabin["compartmentId"] in event.get("unavailableCompartments",()) or window["dockWindowId"] in event.get("unavailableDockWindows",()):
        return False
    return all((
        request["temperatureEvidence"]["state"]=="verified",
        bool(request["temperatureEvidence"]["lotObservationRef"]),
        bool(request["approvalRef"]),
        cabin["compartmentId"] in request["approvedCompartmentIds"],
        request["profileCode"] in cabin["preparedForClasses"],
        cabin["prepared"],bool(cabin["preloadCheckId"]),
        request["sealedLoad"],cabin["sealedCompatible"],
        request["massKg"]<=cabin["maxKg"],request["volumeLitres"]<=cabin["maxLitres"],
        window["dockWindowId"] in request["allowedDockWindowIds"],
        request["readyMinute"]<=window["startMinute"],
        cabin["compartmentWaitingAtMinute"]<=window["startMinute"],
        window["startMinute"]<window["endMinute"]<=request["latestStageFinishMinute"],
        window["endMinute"]<=min(cabin["compartmentClosesAtMinute"],departure),
    ))


def audit():
    data,case,visual=map(read,["inputs.json","operational-case.json","visual.json"])
    assert data==case["inputs"]
    assert data["scope"]["doesNotReleaseShipments"] and data["scope"]["noLiveTelemetry"]
    assert all(not row["temperatureEvidence"]["permitsShipping"] for row in data["loadRequests"])
    assert len(data["loadRequests"])==24 and len(data["waves"])==3
    previous={}
    previous_loads=set()
    for plan,shown in zip(case["scenarioResults"],visual["scenarios"],strict=True):
        scenario=plan["id"]
        assert scenario==shown["id"]
        event=data["disruption"] if scenario=="disruption" else {}
        rows=by_id(plan["rows"],"loadId")
        assert rows.keys()==by_id(data["loadRequests"],"loadId").keys()
        assert shown["rows"]==[{k:v for k,v in row.items() if k!="evaluatedOptions"} for row in plan["rows"]]
        assert plan["metrics"]==shown["metrics"]
        assert shown["compartmentTotals"]==plan["compartmentTotals"]
        selected={}
        aggregate_mass=aggregate_wait=0
        for wave in data["waves"]:
            bins=by_id(wave["compartments"],"compartmentId")
            windows=by_id(wave["stagingWindows"],"dockWindowId")
            requests=[item for item in data["loadRequests"] if item["waveId"]==wave["waveId"]]
            assert not wave["allowCrossWave"]
            assert all(item["simultaneousLoadLimit"]==1 for item in windows.values())
            for cabin in bins.values():
                assert cabin["maxKg"]==cabin["ratedKg"]-cabin["reserveKg"]>0
                assert cabin["maxLitres"]==cabin["ratedLitres"]-cabin["reserveLitres"]>0
            independent_options=[[
                (cabin,window) for cabin,window in product(bins,windows)
                if eligible(load,bins[cabin],windows[window],wave["departureMinute"],event)
            ] for load in requests]

            def result(values):
                dock=defaultdict(list)
                used=defaultdict(list)
                held=wait=changes=0
                for load,item in zip(requests,values,strict=True):
                    identifier=load["loadId"]
                    if previous.get(identifier) is not None and item!=previous[identifier]:
                        changes+=1
                    if item is None:
                        held+=1
                        continue
                    cid,wid=item
                    slot=windows[wid]
                    wait+=slot["startMinute"]-load["readyMinute"]
                    dock[slot["dockId"]].append((slot["startMinute"],slot["endMinute"]))
                    used[cid].append(load)
                if any(min(first[1],second[1])>max(first[0],second[0])
                       for records in dock.values() for first,second in combinations(records,2)):
                    return None
                if any(sum(row[field] for row in contents)>bins[key][limit]
                       for key,contents in used.items() for (field,limit) in (("massKg","maxKg"),("volumeLitres","maxLitres"))):
                    return None
                return (held, changes if scenario=="disruption" else 0, wait,
                        tuple(value if value else ("~","~") for value in values)),used

            computed=[]
            for idx,load in enumerate(requests):
                row=rows[load["loadId"]]
                assert row["waveId"]==wave["waveId"]
                assert row["massKg"]==load["massKg"] and row["volumeLitres"]==load["volumeLitres"]
                pair=(row["compartmentId"],row["dockWindowId"]) if row["status"]=="loaded" else None
                assert row["reasonCodes"]
                if pair:
                    assert pair in independent_options[idx]
                    window=windows[pair[1]]
                    assert row["startUtc"]==timestamp(data["dateUtc"],window["startMinute"])
                    assert row["endUtc"]==timestamp(data["dateUtc"],window["endMinute"])
                    assert row["waitMinutes"]==window["startMinute"]-load["readyMinute"]
                    aggregate_mass+=row["massKg"]
                    aggregate_wait+=row["waitMinutes"]
                    selected[load["loadId"]]=pair
                else:
                    assert row["status"]=="held"
                    assert all(row[field] is None for field in ("compartmentId","dockWindowId","startUtc","endUtc","waitMinutes"))
                computed.append(pair)
            measured,contents=result(computed)
            if scenario=="baseline":
                greedy=[]
                for load,options in zip(requests,independent_options,strict=True):
                    options.sort(key=lambda item:(windows[item[1]]["startMinute"]-load["readyMinute"],*item))
                    choice=next((item for item in options if result([*greedy,item,*([None]*(len(requests)-len(greedy)-1))]) is not None),None)
                    greedy.append(choice)
                assert greedy==computed
            else:
                optimum=min(out[0] for choices in product(*[[None,*item] for item in independent_options])
                            if (out:=result(choices)) is not None)
                assert optimum==measured
            for cid,cabin in bins.items():
                used=contents[cid]
                record=next(row for row in plan["compartmentTotals"] if row["compartmentId"]==cid)
                assert record["loadIds"]==[item["loadId"] for item in used]
                assert (record["usedKg"],record["usedLitres"])==(sum(item["massKg"] for item in used),sum(item["volumeLitres"] for item in used))
                assert (record["maxKg"],record["maxLitres"])==(cabin["maxKg"],cabin["maxLitres"])
        assert len(plan["compartmentTotals"])==6
        metrics=plan["metrics"]
        assert (metrics["requested"],metrics["loaded"],metrics["held"])==(len(rows),len(selected),len(rows)-len(selected))
        assert (metrics["waitMinutes"],metrics["loadedKg"])==(aggregate_wait,aggregate_mass)
        assert metrics["changed"]==sum(selected.get(key)!=value for key,value in previous.items())
        previous=selected


if __name__=="__main__":
    audit()
    print("Independent verifier reproduced capacity, dock conflicts, evidence checks and exact scenario optima.")
