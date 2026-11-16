"""Reproducible nonclinical hospital planning worklists (no external connections)."""
import argparse
import json
from pathlib import Path

HERE=Path(__file__).resolve().parent
SOURCES = {
    "elective-access": {"title":"NHS England: National elective access policy","url":"https://www.england.nhs.uk/long-read/national-elective-access-policy/","scope":"Illustrates administrative booking capacity and wait-list review; no local rule is imported."},
    "cleanliness": {"title":"NHS England: National standards of healthcare cleanliness 2025","url":"https://www.england.nhs.uk/long-read/national-standards-of-healthcare-cleanliness-2025/","scope":"Illustrates cleaning responsibility and coordination; does not certify any sample room."},
    "instrument-handling": {"title":"NHS England: Health Technical Memorandum 01-01","url":"https://www.england.nhs.uk/publication/decontamination-of-surgical-instruments-htm-01-01/","scope":"Illustrates the importance of traceability and handoff controls; not a process instruction."},
}
# task tuple: title, duration, finish-by, follows previous, evidence missing
WORKFLOWS = [
 ("new-york","Appointment backlog administration","Booking desk","Schedule pre-reviewed administrative work, without ranking people or booking appointments.","elective-access","booking workstation",[
  ("Check completeness of already reviewed requests",25,40,False,False),
  ("Match administrative workload to offered session inventory",20,70,True,False),
  ("Prepare an exception list for the booking supervisor",15,95,True,False),
  ("Reserve callback administration batch",30,120,False,False),
  ("Prepare a contact batch with no authorization record",15,140,False,True),
 ]),
 ("rome","Diagnostic session support","Diagnostic logistics","Reserve room setup, capacity reconciliation and handoff work, not tests or appointments.","elective-access","room setup station",[
  ("Reconcile accepted session inventory with room availability",20,40,False,False),
  ("Reserve equipment set-up and room readiness time",30,80,True,False),
  ("Send the availability draft to the session coordinator",15,110,True,False),
  ("Fit an additional setup batch before the room locks",40,90,False,False),
  ("Reserve a room without facilities clearance",15,145,False,True),
 ]),
 ("tirana","Internal supplies transport","Hospital porter dispatch","Coordinate sealed supply carts and receiving teams; no patient movement or building-route instructions.",None,"supply cart",[
  ("Check sealed-cart destination and receiving window",15,40,False,False),
  ("Reserve staff and cart for scheduled material transfer",30,80,True,False),
  ("Hand off the shipment to an available receiving desk",15,110,True,False),
  ("Collect a spare cart before the receiving desk closes",40,95,False,False),
  ("Dispatch a cart without receiving confirmation",20,150,False,True),
  ("Record delivery receipt after unconfirmed dispatch",10,160,False,False),
 ]),
 ("london","Room cleaning and bed-turn logistics","Facilities coordination","Schedule room access, facilities work and documented handoff; no return-to-care clearance.","cleanliness","room service work area",[
  ("Confirm the sample room is released to facilities",20,40,False,False),
  ("Reserve a cleaning-team work window",30,85,True,False),
  ("Present readiness documentation to the room coordinator",15,110,True,False),
  ("Add a second work request to the same facilities window",45,100,False,False),
  ("Queue preparation work without room-access approval",15,150,False,True),
 ]),
 ("paris","Instrument set turnaround logistics","Sterile-services coordination","Schedule transport and readiness paperwork for sealed sets; instrument processing and clinical release are not modeled.","instrument-handling","instrument logistics station",[
  ("Reconcile sample handling labels against the receiving list",20,45,False,False),
  ("Reserve sealed-set transport and desk handoff",30,85,True,False),
  ("Queue ready-state paperwork for an authorized reviewer",15,110,True,False),
  ("Arrange a second set transfer within the same dock window",45,95,False,False),
  ("Propose movement of a set missing the traceability reference",20,145,False,True),
 ]),
 ("munich","Facilities shift and service handoff","Facilities duty desk","Move open service work from the outgoing shift to named receiving capacity.",None,"handoff workstation",[
  ("Collect outstanding facilities service requests",25,45,False,False),
  ("Prepare a shift handoff and reserve receiving capacity",25,85,True,False),
  ("Assign administrative follow-up to the incoming duty shift",20,115,True,False),
  ("Reserve field follow-up preparation before duty shift ends",15,85,False,False),
  ("Book controlled access without a recorded permission",15,145,False,True),
 ]),
]


def fixtures():
    cases=[]
    for id,title,function,scope,reference,asset,steps in WORKFLOWS:
        tag=id.upper()
        resources=[
            {"resourceId":f"{tag}-R1","role":"Duty coordinator","window":[0,125 if id=="new-york" else 105],"capabilities":["coordination","handling"]},
            {"resourceId":f"{tag}-R2","role":"Receiving coordinator","window":[55,165],"capabilities":["coordination","handoff"]},
        ]
        equipment={"assetId":f"{tag}-A1","assetType":asset,"window":[0,125],"unavailable":[[39,44]]}
        jobs=[]
        for n,(label,length,latest,after,missing) in enumerate(steps,1):
            jobs.append({"workId":f"{tag}-{n:02}","name":label,"durationMinutes":length,"window":[0,latest],
             "requiredCapability": ("coordination" if n==1 else "handoff" if n==3 else "handling"), "requiresAsset":n!=1 and not (id=="munich" and n==4),
             "prerequisiteIds":[f"{tag}-{n-1:02}"] if after or (id=="tirana" and n==6) else [],
             "entryEvidence":"missing" if missing else "asserted_in_sample","administrativeOrder":n})
        cases.append({"id":id,"title":title,"function":function,"workflow":scope,
                      "publicWorkflowReference":SOURCES.get(reference),"resources":resources,"sharedAsset":equipment,"work":jobs})
    return cases


REASONS={"required_evidence_missing":"Required administrative evidence is missing; manual review.",
         "prerequisite_held":"The preceding activity was not scheduled.",
         "no_resource_window":"No eligible operator and asset have a free slot in the supplied window."}


def make_plan(case):
    booked={r["resourceId"]:[] for r in case["resources"]}
    asset=case["sharedAsset"]
    reserved=list(asset["unavailable"])
    planned={}
    records=[]
    for task in sorted(case["work"],key=lambda t:t["administrativeOrder"]):
        wid=task["workId"]
        cap=task["requiredCapability"]
        deps=task["prerequisiteIds"]
        actors=[r for r in case["resources"] if cap in r["capabilities"]]
        code=None
        if task["entryEvidence"]!="asserted_in_sample":
            code="required_evidence_missing"
        elif not all(dep in planned for dep in deps):
            code="prerequisite_held"
        else:
            option=None
            for r in actors:
                rid=r["resourceId"]
                active_from=max(task["window"][0],r["window"][0],*( [planned[p]["endMinute"] for p in deps] or [0] ))
                active_until=min(task["window"][1],r["window"][1])
                if task["requiresAsset"]:
                    active_from=max(active_from,asset["window"][0])
                    active_until=min(active_until,asset["window"][1])
                t=active_from
                occupied=booked[rid]+(reserved if task["requiresAsset"] else [])
                for a,b in sorted(occupied):
                    if t+task["durationMinutes"]<=a:
                        break
                    t=max(t,b)
                end=t+task["durationMinutes"]
                if end<=active_until:
                    option=min(option,(end,rid,t)) if option else (end,rid,t)
            if not option:
                code="no_resource_window"
        if code:
            record={"workId":wid,"activity":task["name"],"status":"held","holdCode":code,"holdReason":REASONS[code],
                    "candidateResourceIds":[r["resourceId"] for r in actors]}
        else:
            end,rid,start=option
            booked[rid].append([start,end])
            if task["requiresAsset"]:
                reserved.append([start,end])
            record={"workId":wid,"activity":task["name"],"status":"scheduled","resourceId":rid,
                    "assetId":asset["assetId"] if task["requiresAsset"] else None,"startMinute":start,"endMinute":end}
            planned[wid]=record
        records.append(record)
    return {"id":case["id"],"title":case["title"],"workflow":case["workflow"],"decisions":records,
            "summary":{"requested":len(records),"scheduled":len(planned),"held":len(records)-len(planned)}}


def depth(value, keys, level=0):
    if isinstance(value,dict):
        keys.update(value)
        return max([level+1]+[depth(v,keys,level+1) for v in value.values()])
    if isinstance(value,list):
        return max([level]+[depth(v,keys,level) for v in value])
    return level


def operation(case, result):
    people={r["resourceId"]:r for r in case["resources"]}
    tasks={j["workId"]:j for j in case["work"]}
    rows=[]
    for d in result["decisions"]:
        t=tasks[d["workId"]]
        evidence=None
        if d["status"]=="scheduled":
            p=people[d["resourceId"]]
            evidence={"assignmentValidation":{"capabilityCheck":{"capabilityRequired":t["requiredCapability"],
                        "configuredCapabilities":p["capabilities"],"capabilitySatisfied":t["requiredCapability"] in p["capabilities"]},
                       "resourceTiming":{"assignedShiftStart":p["window"][0],"assignedShiftEnd":p["window"][1],
                        "workWindowSlackMinutes":t["window"][1]-d["endMinute"]},
                       "evidenceGate":{"evidenceInput":t["entryEvidence"],"verifiedByRealHospital":False},
                       "assetReservation":{"attachedAsset":d["assetId"],"assetUsed":t["requiresAsset"]}}}
        rows.append({"activityInput":t,"draftDecision":d,"modelEvidence":evidence})
    return {"cityExampleId":case["id"],"reference":case["publicWorkflowReference"],
        "operatingOrganization":{"dataClassification":"Illustrative hospital logistics only",
        "operatingUnit":{"unitTitle":case["function"],"serviceFunction":{"operationPurpose":case["workflow"],
        "workstream":{"workflowTitle":case["title"],"planningRun":{"timeBasis":"Relative planning minutes",
        "realFacilityAssociation":False,"liveSystemConnection":False,"resourceRoster":case["resources"],"sharedAsset":case["sharedAsset"],
        "decisionLedger":{"counts":result["summary"],"activityRecords":rows}}}}}}}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check",action="store_true")
    args=parser.parse_args()
    source=fixtures()
    results=[make_plan(x) for x in source]
    records=[operation(c,r) for c,r in zip(source,results,strict=True)]
    scope="Illustrative operational workflows; no actual hospital, patient, care, routing, safety approval or performance is represented."
    companion={"id":"hospital-operations-worklists","title":"Six urban hospital logistics worklists","caseScope":scope,
               "sourceEvidence":{"inputsFile":"inputs.json","viewDigest":"visual.json","verifierFile":"verify.py","realHospitalRecordsUsed":False},
               "cityCases":records}
    for row in records:
        keys=set()
        level=depth(row,keys)
        assert level>=10 and len(keys)>=50, (row["cityExampleId"],level,len(keys))
        print(row["cityExampleId"],level,len(keys))
    for name, obj in {"inputs.json":{"schemaVersion":1,"cases":source},
        "visual.json":{"schemaVersion":1,"scope":scope,"scenarios":results},"operational-case.json":companion}.items():
        serialized=json.dumps(obj,ensure_ascii=False,indent=2)+"\n"
        if args.check:
            assert (HERE/name).read_text()==serialized
        else:
            (HERE/name).write_text(serialized)


if __name__=="__main__":
    main()
