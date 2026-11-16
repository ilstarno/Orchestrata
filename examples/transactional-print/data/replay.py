"""Deterministic, offline replay of a transactional printing shift."""
from itertools import groupby
from pathlib import Path
from math import ceil
import argparse
import json

ROOT = Path(__file__).resolve().parent
STEPS = ("intake", "compose", "batch", "print", "insert", "quality", "dispatch")
KINDS = {
    "invoices": ("white-duplex", 2, "standard-envelope"),
    "statements": ("white-duplex", 4, "window-envelope"),
    "notices": ("secure-simplex", 3, "window-envelope"),
}


def source():
    rows = []
    faults = {2, 11, 20, 31, 46, 61, 68}
    for i in range(72):
        kind = tuple(KINDS)[i % 3]
        stock, sheets, envelope = KINDS[kind]
        rows.append({
            "pieceId": f"PRINT-{i + 1:03}",
            "documentType": kind,
            "paperProfile": stock,
            "documentSheets": sheets,
            "envelopeProfile": envelope,
            "templateRevision": "v3" if i not in {9, 17, 23} else "unsupported",
            "dataComplete": i not in {12, 42, 52},
            "markedSheetFault": i in faults,
        })
    return rows


def run(rows, proposed, policy):
    holds, eligible = [], []
    for row in rows:
        if row["templateRevision"] != "v3" or not row["dataComplete"]:
            holds.append({"pieceId": row["pieceId"], "documentType": row["documentType"],
                          "reason": "template_revision" if row["templateRevision"] != "v3" else "incomplete_record"})
        else:
            eligible.append(row)
    ordered = sorted(eligible, key=lambda row: (row["paperProfile"], row["envelopeProfile"], row["pieceId"])) if proposed else eligible
    groups = [(profile, list(pieces)) for profile, pieces in groupby(
        ordered, key=lambda row: (row["paperProfile"], row["envelopeProfile"]))]
    batches = []
    trace = []
    setup_waste = len(groups) * policy["setupSheets"]
    spoiled = 0
    for i, ((paper, envelope), pieces) in enumerate(groups):
        batch_id = f"B-{i + 1:03}"
        batches.append({"batchId": batch_id, "paperProfile": paper, "envelopeProfile": envelope,
                        "pieces": len(pieces), "setupSheets": policy["setupSheets"]})
        for row in pieces:
            failed = row["markedSheetFault"]
            replacement = (1 if proposed else row["documentSheets"]) if failed else 0
            spoiled += replacement
            trace.append({
                "pieceId": row["pieceId"], "documentType": row["documentType"], "batchId": batch_id,
                "expectedGoodSheets": row["documentSheets"], "replacementSheets": replacement,
                "firstFailureDetectedAt": ("insert" if proposed else "quality") if failed else None,
                "reprintMatch": "passed" if failed else "not_required",
                "dispatchCount": 1, "manifestStatus": "matched",
            })
    faults = sum(row["markedSheetFault"] for row in eligible)
    total_sheets = sum(row["documentSheets"] for row in eligible)
    seq = {
        "intake": 4 + ceil(len(rows) / 24),
        "compose": 3 + ceil(len(eligible) / 20),
        "batch": 3 + len(groups) * policy["batchSetupMinutes"],
        "print": 3 + ceil((total_sheets + setup_waste) / policy["pressSheetsPerMinute"]),
        "insert": 4 + ceil(len(eligible) / policy["insertPiecesPerMinute"]) + (faults * 2 if proposed else 0),
        "quality": 3 + ceil(len(eligible) / policy["qualityPiecesPerMinute"]) + (0 if proposed else faults * 4),
        "dispatch": 5 + ceil(len(eligible) / 20),
    }
    assert len({row["pieceId"] for row in trace + holds}) == len(rows)
    assert sum(row["dispatchCount"] for row in trace) == len(rows) - len(holds)
    return {
        "stageMinutes": seq, "turnaroundMinutes": sum(seq.values()), "received": len(rows),
        "accepted": len(trace), "exceptions": len(holds), "replacementSheets": spoiled,
        "misfedPieces": faults, "fullPieceReprints": faults if not proposed else 0,
        "setupWasteSheets": setup_waste, "wasteSheets": setup_waste + spoiled,
        "goodSheets": total_sheets, "totalProducedSheets": total_sheets + setup_waste + spoiled,
        "changeovers": len(groups) - 1,
        "batches": batches, "holds": holds, "pieceDecisions": trace,
    }


def build():
    rows = source()
    policy = {"setupSheets": 2, "batchSetupMinutes": 1, "pressSheetsPerMinute": 16,
              "insertPiecesPerMinute": 12, "qualityPiecesPerMinute": 15}
    baseline, proposed = (run(rows, improved, policy) for improved in (False, True))
    assert baseline["holds"] == proposed["holds"]
    assert baseline["misfedPieces"] == proposed["misfedPieces"] == 7
    assert baseline["goodSheets"] == proposed["goodSheets"] == 200
    assert baseline["accepted"] == proposed["accepted"] == 66
    steps = [{"id": step, "baselineMinutes": baseline["stageMinutes"][step],
              "proposedMinutes": proposed["stageMinutes"][step]} for step in STEPS]
    metrics = [
        {"id": ident, "unit": unit, "baseline": baseline[key], "proposed": proposed[key]}
        for ident, unit, key in (
            ("turnaround", "minutes", "turnaroundMinutes"), ("waste", "sheets", "wasteSheets"),
            ("reprints", "sheets", "replacementSheets"), ("exceptions", "pieces", "exceptions"),
        )
    ]
    cases = []
    for kind in KINDS:
        pieces = sum(row["documentType"] == kind for row in rows)
        held = sum(row["documentType"] == kind for row in proposed["holds"])
        cases.append({"id": kind, "pieces": pieces, "accepted": pieces - held, "held": held})
    visual = {"steps": steps, "metrics": metrics, "cases": cases}
    sample = next(row for row in proposed["pieceDecisions"] if row["replacementSheets"])
    first = next(row for row in rows if row["pieceId"] == sample["pieceId"])
    # The nested path follows an individual released piece through site, shift, lane,
    # batch, scan, repair and manifest checks. It is linked to the decision, not padding.
    manifest = {
        "facilityCode": "PLANT-DEMO", "dispatchControl": {
            "collectionCode": "PICKUP-SAMPLE-01", "pickupDeadlineMinute": 180,
            "accountability": {"shiftCode": "SHIFT-DAY-A", "physicalLane": {
                "laneCode": "INSERT-A", "envelopeProfile": first["envelopeProfile"],
                "releasePacket": {"batchId": sample["batchId"], "record": {
                    "pieceId": sample["pieceId"], "documentType": sample["documentType"],
                    "componentInspection": {"expectedGoodSheets": sample["expectedGoodSheets"],
                        "identityCheck": {"sourceIdType": "internal_sample_piece", "logicalPieceKey": sample["pieceId"],
                            "repairCheck": {"reprintMode": "replace_faulty_component",
                                "replacementSheets": sample["replacementSheets"],
                                "barcodeRescan": {"matchedPiece": True,
                                    "releaseLedger": {"manifestStatus": sample["manifestStatus"],
                                                      "approvedDispatchCount": sample["dispatchCount"]}}}}}}}}}}
    }
    operational = {
        "id": "transactional-print-control-and-reconciliation",
        "scope": {
            "caseType": "modeled_shift", "sendMail": False, "runsProductionEquipment": False,
            "sourceSnapshotReference": "PRINT-SHIFT-01", "measurementUnit": "serial_stage_minutes",
            "personalDataIncluded": False, "barcodesPhysical": False, "assumeSameFaultEvents": True,
        },
        "processPolicy": policy,
        "inputRecords": rows,
        "dispatchEvidence": manifest,
        "comparison": {"baseline": baseline, "proposed": proposed},
        "summary": visual,
    }
    return {"visual.json": visual, "operational-case.json": operational}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    for name, value in build().items():
        serialized = json.dumps(value, ensure_ascii=False, indent=2) + "\n"
        path = ROOT / name
        if args.write:
            path.write_text(serialized)
        elif path.read_text() != serialized:
            raise SystemExit(f"Replayed result differs: {name}")
    print("Verified 72 records, 66 released, 6 held, 7 faults in both workflows.")


if __name__ == "__main__":
    main()
