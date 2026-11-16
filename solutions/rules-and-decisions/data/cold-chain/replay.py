"""Offline advisory planning for a bounded cold-chain loading example."""
import argparse
from datetime import datetime, timedelta, timezone
from collections import Counter, defaultdict
from pathlib import Path
import json

ROOT=Path(__file__).resolve().parent
DAY=datetime(2026,11,16,tzinfo=timezone.utc)
SOURCE="https://www.fda.gov/food/food-safety-modernization-act-fsma/fsma-final-rule-sanitary-transportation-human-and-animal-food"


def ts(minute):
    return (DAY+timedelta(minutes=minute)).strftime("%Y-%m-%dT%H:%M:%SZ")


def case_inputs():
    waves=[]
    requests=[]
    for number,start in enumerate((14*60, 16*60, 18*60),1):
        wave=f"WAVE-{number}"
        bins=[
            {"compartmentId":f"C-{number}-A","classCode":"profile-a","ratedKg":570+number*10,"reserveKg":50,"maxKg":520+number*10,
             "ratedLitres":8500,"reserveLitres":500,"maxLitres":8000,
             "sealedCompatible":True,"prepared":True,"preloadCheckId":f"PC-{number}-A","compartmentWaitingAtMinute":start,
             "compartmentClosesAtMinute":start+75,"preparedForClasses":["profile-a"]},
            {"compartmentId":f"C-{number}-B","classCode":"profile-b","ratedKg":635,"reserveKg":50,"maxKg":585,
             "ratedLitres":8000,"reserveLitres":500,"maxLitres":7500,
             "sealedCompatible":True,"prepared":True,"preloadCheckId":f"PC-{number}-B","compartmentWaitingAtMinute":start,
             "compartmentClosesAtMinute":start+75,"preparedForClasses":["profile-b"]},
        ]
        windows=[{"dockWindowId":f"S-{number}-{slot+1}","dockId":"EAST" if slot<6 else "WEST",
                  "startMinute":start+(slot if slot<6 else 3)*10,
                  "endMinute":start+(slot+1 if slot<6 else 4)*10,"simultaneousLoadLimit":1,
                  "stagingEvidenceRef":f"LOG-{number}-{slot+1}"} for slot in range(7)]
        waves.append({"waveId":wave,"opensUtc":ts(start),"warehouseZone":"US-PA sample cross-dock","departureUtc":ts(start+85),
                      "departureMinute":start+85,"compartments":bins,"stagingWindows":windows,"allowCrossWave":False})
        table=[
            ("profile-a",230,3100,(1,2),0,26,"verified"),
            ("profile-b",250,2950,(2,3),0,40,"verified"),
            ("profile-a",160,2400,(1,),0,10,"verified"),
            ("profile-b",170,2100,(4,),10,50,"verified"),
            ("profile-b",190 if number==1 else 145,2700,(5,7),20,60,"verified"),
            ("profile-a",80,1200,(6,),20,65,"verified" if number==2 else "missing" if number==1 else "stale"),
            ("profile-b",120,1200,(3,),38,51,"verified"),
            ("profile-a",640 if number==1 else 120,5000 if number!=2 else 16000,(2,6),0,65,"verified"),
        ]
        for serial,(profile,kg,volume,slots,ready,cutoff,evidence) in enumerate(table,1):
            load=f"L-{number}-{serial:02}"
            allowed=[bin["compartmentId"] for bin in bins if bin["classCode"]==profile]
            if number==3 and serial==8:
                allowed=[]
            requests.append({"loadId":load,"waveId":wave,"massKg":kg,"volumeLitres":volume,"profileCode":profile,
             "readyMinute":start+ready,"latestStageFinishMinute":start+cutoff,"receiverAppointmentRef":f"REC-{number}",
             "readyAtUtc":ts(start+ready),"stageByUtc":ts(start+cutoff),"approvedCompartmentIds":allowed,
             "allowedDockWindowIds":[f"S-{number}-{value}" for value in slots],"sealedLoad":True,
             "approvalRef":f"SPEC-{profile}-{number}" if allowed else None,
             "temperatureEvidence":{"label":"shipper profile recorded by trusted preparer","state":evidence,
              "lotObservationRef":f"OBS-{number}-{serial:02}" if evidence!="missing" else None,
              "profileRevision":"2026-11-fixture-v1","permitsShipping":False}})
    return {"caseId":"cold-chain-staging-and-compartments","dateUtc":"2026-11-16","units":{"mass":"kg","volume":"litres","time":"integer minutes after 00:00 UTC"},
        "scope":{"transport":"US food road transport context; bounded sample only","doesNotReleaseShipments":True,
                 "noLiveTelemetry":True,"noUniversalTemperatureBands":True,"temperatureClassesAreShipperOwnedLabels":True},
        "policy":{"approval":"allowed compartment and matching shipper class, sealed condition","requiresObservedThermalEvidence":True,
            "allowedEvidenceState":"verified","exclusiveDockWindow":True,"keepsLoadWhole":True,"compartmentCapacityAfterReserve":True,
            "objectiveOrder":["most-placed-loads","fewest-unstable-assignments-on-disruption","minimum-staging-wait","stable-id-tie"]},
        "lawContext":{"fdaUrl":SOURCE,"regulation":"21 CFR Part 1, Subpart O, including §1.908 as applicable",
            "verifiedOn":"2026-11-16","appliesTo":"U.S. food transportation operations within rule scope; roles and requirements vary",
            "whatIsNotEncoded":"Coverage, exceptions, exemptions, written agreements and release authority require review"},
        "waves":waves,"loadRequests":requests,
        "disruption":{"unavailableCompartments":["C-2-B"],"unavailableDockWindows":["S-3-2"],
                      "reason":"compartment preloading readiness lost; staging slot withdrawn","observationUtc":"2026-11-16T13:00:00Z"}}


def option_table(wave,load,event):
    veto=set()
    options=[]
    evidence=load["temperatureEvidence"]
    if evidence["state"]!="verified":
        return [],["thermal-evidence-"+evidence["state"]]
    if not evidence["lotObservationRef"]:
        return [],["thermal-evidence-missing"]
    if not load["approvalRef"] or not load["approvedCompartmentIds"]:
        return [],["shipper-approval"]
    for compartment in wave["compartments"]:
        cid=compartment["compartmentId"]
        if cid not in load["approvedCompartmentIds"]:
            continue
        if cid in event.get("unavailableCompartments",[]) or not compartment["prepared"] or not compartment["preloadCheckId"]:
            veto.add("compartment-unavailable")
            continue
        if load["profileCode"] not in compartment["preparedForClasses"] or not (load["sealedLoad"] and compartment["sealedCompatible"]):
            veto.add("compatibility")
            continue
        too_heavy = load["massKg"] > compartment["maxKg"]
        too_large = load["volumeLitres"] > compartment["maxLitres"]
        if too_heavy:
            veto.add("max-mass")
        if too_large:
            veto.add("max-volume")
        if too_heavy or too_large:
            continue
        for slot in wave["stagingWindows"]:
            sid=slot["dockWindowId"]
            if sid not in load["allowedDockWindowIds"]:
                continue
            if sid in event.get("unavailableDockWindows",[]):
                veto.add("dock-unavailable")
                continue
            first,last=slot["startMinute"],slot["endMinute"]
            if not (max(load["readyMinute"],compartment["compartmentWaitingAtMinute"])<=first<last<=
                    min(load["latestStageFinishMinute"],compartment["compartmentClosesAtMinute"],wave["departureMinute"])):
                veto.add("staging-window")
                continue
            options.append((cid,sid,first-load["readyMinute"],first,last))
    if not options and not veto:
        veto.add("shipper-approval" if not any(item["compartmentId"] in load["approvedCompartmentIds"]
                                              for item in wave["compartments"]) else "staging-window")
    return sorted(options,key=lambda t:(t[2],t[0],t[1])),sorted(veto)


def optimize(wave,loads,kind,event,reference):
    limits={row["compartmentId"]:row for row in wave["compartments"]}
    dock={row["dockWindowId"]:row["dockId"] for row in wave["stagingWindows"]}
    options={row["loadId"]:option_table(wave,row,event) for row in loads}
    stores=defaultdict(lambda:[0,0])
    occupied=defaultdict(list)
    board={}
    winner=winner_score=None

    def solve(index):
        nonlocal winner,winner_score
        if index==len(loads):
            selected=[item for item in board.values() if item]
            changed=sum(reference[identifier]!=assignment for identifier,assignment in board.items()
                        if reference.get(identifier) is not None)
            score=(len(loads)-len(selected),changed if kind=="disruption" else 0,
                   sum(item[2] for item in selected),tuple(item[:2] if item else ("~","~") for item in board.values()))
            if winner_score is None or score<winner_score:
                winner,winner_score=board.copy(),score
            return
        item=loads[index]
        identifier=item["loadId"]
        viable=[]
        for option in options[identifier][0]:
            cid,sid,_,begin,end=option
            used_mass,used_volume=stores[cid]
            if (used_mass+item["massKg"]<=limits[cid]["maxKg"] and
                used_volume+item["volumeLitres"]<=limits[cid]["maxLitres"]
                and all(end<=prior_begin or prior_end<=begin for prior_begin,prior_end in occupied[dock[sid]])):
                viable.append(option)
        proposals=(viable[:1] or [None]) if kind=="baseline" else [*viable,None]
        for assignment in proposals:
            board[identifier]=assignment
            if assignment:
                cid,sid,_,begin,end=assignment
                stores[cid][0]+=item["massKg"]
                stores[cid][1]+=item["volumeLitres"]
                occupied[dock[sid]].append((begin,end))
            solve(index+1)
            if assignment:
                occupied[dock[sid]].pop()
                stores[cid][0]-=item["massKg"]
                stores[cid][1]-=item["volumeLitres"]
    solve(0)
    return winner,options


def simulate(inp, kind, reference):
    rows=[]
    choices={}
    event=inp["disruption"] if kind=="disruption" else {}
    compartment_totals=[]
    for wave in inp["waves"]:
        loads=[load for load in inp["loadRequests"] if load["waveId"]==wave["waveId"]]
        board,options=optimize(wave,loads,kind,event,reference)
        choices.update(board)
        dock={slot["dockWindowId"]:slot["dockId"] for slot in wave["stagingWindows"]}
        for load in loads:
            identifier=load["loadId"]
            choice=board[identifier]
            possible,block=options[identifier]
            reasons=["compatible-window"] if choice else block if not possible else ["shared-capacity-or-dock"]
            projected=[]
            for option in possible:
                cid,sid,wait,start,end=option
                peers=[(other,board[other["loadId"]]) for other in loads if other["loadId"]!=identifier and board[other["loadId"]]]
                occupants=[other for other,seat in peers if seat[0]==cid]
                limit=next(bin for bin in wave["compartments"] if bin["compartmentId"]==cid)
                violations=[]
                if sum(other["massKg"] for other in occupants)+load["massKg"]>limit["maxKg"]:
                    violations.append("max-mass")
                if sum(other["volumeLitres"] for other in occupants)+load["volumeLitres"]>limit["maxLitres"]:
                    violations.append("max-volume")
                if any(dock[sid]==dock[seat[1]] and start<seat[4] and seat[3]<end for _,seat in peers):
                    violations.append("dock-in-use")
                projected.append({"compartmentId":cid,"dockWindowId":sid,"fromUtc":ts(start),"untilUtc":ts(end),
                                  "boardVetoCodes":violations,"waitMinutes":wait})
            if not choice and possible:
                reasons+=sorted({v for row in projected for v in row["boardVetoCodes"]})
            rows.append({"loadId":identifier,"waveId":load["waveId"],"status":"loaded" if choice else "held",
                         "compartmentId":choice[0] if choice else None,"dockWindowId":choice[1] if choice else None,
                         "startUtc":ts(choice[3]) if choice else None,"endUtc":ts(choice[4]) if choice else None,
                         "waitMinutes":choice[2] if choice else None,"massKg":load["massKg"],"volumeLitres":load["volumeLitres"],
                         "reasonCodes":reasons,"evaluatedOptions":projected})
        for cabin in wave["compartments"]:
            assigned=[row for row in rows if row["compartmentId"]==cabin["compartmentId"]]
            compartment_totals.append({"compartmentId":cabin["compartmentId"],"usedKg":sum(row["massKg"] for row in assigned),
               "maxKg":cabin["maxKg"],"usedLitres":sum(row["volumeLitres"] for row in assigned),"maxLitres":cabin["maxLitres"],
               "loadIds":[row["loadId"] for row in assigned]})
    assigned=[row for row in rows if row["status"]=="loaded"]
    metrics={"requested":len(rows),"loaded":len(assigned),"held":len(rows)-len(assigned),
             "waitMinutes":sum(row["waitMinutes"] for row in assigned),
             "changed":sum(choices.get(k)!=v for k,v in reference.items() if v is not None),
             "loadedKg":sum(row["massKg"] for row in assigned)}
    return {"id":kind,"rows":rows,"metrics":metrics,"compartmentTotals":compartment_totals},choices


def documents():
    inp=case_inputs()
    baseline,pre=simulate(inp,"baseline",{})
    recommendation,planned=simulate(inp,"recommended",pre)
    disrupted,_=simulate(inp,"disruption",planned)
    plans=[baseline,recommendation,disrupted]
    hold=next(row for row in recommendation["rows"] if "thermal-evidence-missing" in row["reasonCodes"])
    source=next(row for row in inp["loadRequests"] if row["loadId"]==hold["loadId"])
    approval={"requiresOperatorReview":True,"authority":{"role":"designated food safety reviewer","releaseAllowed":False}}
    dispatch={"reviewDisposition":"hold","authorizationRecord":approval}
    trace={"departureWave":{"waveId":hold["waveId"],"loadReview":{"loadId":hold["loadId"],"plannerDecision":{
        "status":hold["status"],"constraintEvaluation":{"blockingCode":hold["reasonCodes"][0],"shipperPolicy":{
        "approvalRef":source["approvalRef"],"thermalEvidence":{"lotState":source["temperatureEvidence"]["state"],
        "comparison":{"requiredState":"verified","downstreamDecision":dispatch}}}}}}}}
    case={"caseId":inp["caseId"],"inputs":inp,"scenarioResults":plans,"decisionEvidence":trace}
    visual={"scenarios":[{**{k:v for k,v in plan.items() if k!="rows"},"rows":[{k:v for k,v in row.items() if k!="evaluatedOptions"} for row in plan["rows"]]} for plan in plans]}
    return {"inputs.json":inp,"operational-case.json":case,"visual.json":visual}


if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write",action="store_true")
    args=parser.parse_args()
    for name,data in documents().items():
        path=ROOT/name
        rendered=json.dumps(data,indent=2,ensure_ascii=False)+"\n"
        if args.write:
            path.write_text(rendered,encoding="utf-8")
        elif rendered!=path.read_text(encoding="utf-8"):
            raise SystemExit(f"Replay differs: {name}")
    print("Verified cold-chain baseline, recommended and disruption load plans.")
