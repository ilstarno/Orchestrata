"""Reconcile a local print-manifest fixture; do not read carrier records or write a release."""
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import sys

HERE = Path(__file__).resolve().parent
RULES = (
    ("PRINT-MANIFEST-01", "manifestMembership", "Piece ID appears in the local batch manifest"),
    ("PRINT-MANIFEST-02", "firstRelease", "Piece has no prior internal match in this run"),
    ("PRINT-MANIFEST-03", "readablePacket", "Component observations are present"),
    ("PRINT-MANIFEST-04", "distinctComponents", "Observed component IDs contain no repeats"),
    ("PRINT-MANIFEST-05", "orderedMatch", "Observed component IDs equal the manifest sequence"),
)
CARRIER = {
    "program": "USPS Seamless Acceptance",
    "status": "not-assessed",
    "sourceUrl": "https://pe.usps.com/text/DMM300/705.htm",
    "sourceLocator": "Domestic Mail Manual 705 §22.0",
    "resourcesUrl": "https://postalpro.usps.com/resources/mailing/seamless-acceptance",
    "sourceAuthority": "carrier-program-reference",
    "sourcePublisher": "United States Postal Service",
    "lastCheckedUtcDate": "2026-11-16",
    "appliesTo": "Potentially eligible program mailings; not inferred for training pieces",
    "programChecks": ["mailer participation", "mail eligibility and preparation", "Intelligent Mail barcodes", "electronic documentation", "USPS scan reconciliation"],
    "notAssessedReason": "No physical mailing, carrier barcode, electronic documentation, participation evidence or carrier observations were supplied.",
    "eligibilityDetermined": False,
    "carrierResponseReceived": False,
}
ID = re.compile(r"^[A-Z][A-Za-z0-9-]{0,31}$")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def timestamp(value):
    out = datetime.fromisoformat(value)
    require(out.tzinfo == timezone.utc and value.endswith("Z"), "Expected UTC with Z")
    return out


def valid_id(value):
    return isinstance(value, str) and bool(ID.fullmatch(value))


def packet(value):
    return isinstance(value, list) and 1 <= len(value) <= 16 and all(valid_id(part) for part in value)


def render(fixture):
    policy = fixture["policy"]
    require(policy["holdsOnUnreadableComponents"] is True and policy["preservesManifestOrder"] is True,
            "Unknown manifest policy variant")
    low = timestamp(fixture["windowUtc"]["from"])
    high = timestamp(fixture["windowUtc"]["until"])
    require(timestamp(policy["effectiveFrom"]) <= low < high, "Policy or window out of scope")
    require(1 <= len(fixture["manifest"]) <= 64 and 1 <= len(fixture["observations"]) <= 64, "Unsupported input size")
    manifest = {}
    for entry in fixture["manifest"]:
        piece_id, components = entry["pieceId"], entry["componentIds"]
        require(valid_id(piece_id) and piece_id not in manifest, "Invalid or duplicate manifest piece ID")
        require(packet(components) and len(set(components)) == len(components), "Invalid manifest packet")
        manifest[piece_id] = entry

    events, seen_events, matched = [], set(), {}
    last_time = low
    for row in fixture["observations"]:
        event_id, piece = row["eventId"], row["pieceId"]
        observed, when = row["componentIds"], timestamp(row["observedAt"])
        require(valid_id(event_id) and event_id not in seen_events and valid_id(piece), "Invalid or duplicate scan ID")
        require(low <= when < high and when >= last_time, "Scans must be in the requested chronological window")
        require(observed is None or packet(observed), "Malformed component scan")
        require(row["capture"] in {"reader", "correction-rescan", "reader-unreadable"}
                and (row["capture"] == "reader-unreadable") == (observed is None), "Capture evidence contradicts packet")
        seen_events.add(event_id)
        last_time = when
        expected_entry = manifest.get(piece)
        expected = expected_entry["componentIds"] if expected_entry else None
        prior = matched.get(piece)
        label_info = [
            (expected is not None, "not-on-manifest", expected is not None),
            (prior is None, "duplicate-release", prior),
            (observed is not None, "unreadable-packet", observed is not None),
            (observed is not None and len(observed) == len(set(observed)), "duplicate-component",
                None if observed is None else len(set(observed))),
            (observed == expected, "packet-mismatch", None if observed is None else len(observed)),
        ]
        reason = "internal-match"
        checks = []
        for (rule_id, evaluator, summary), (passed, code, operand) in zip(RULES, label_info):
            status = "not-evaluated"
            if reason == "internal-match":
                status = "pass" if passed else "unknown" if code == "unreadable-packet" else "fail"
                if not passed:
                    reason = code
            checks.append({
                "ruleId": rule_id,
                "ruleRevision": policy["revision"],
                "authority": "sample-facility-policy",
                "enforcement": "hard",
                "evaluation": {
                    "evaluator": evaluator,
                    "outcome": status,
                    "comparison": {
                        "statement": summary,
                        "observedOperand": operand if status != "not-evaluated" else None,
                        "evidence": {
                            "scan": {
                                "identifier": event_id,
                                "capturedAt": row["observedAt"],
                                "capturedBy": policy["scannerId"],
                                "sourceFixture": "inputs.json",
                                "captureMethod": row["capture"],
                                "packetSnapshot": {
                                    "localPieceId": piece,
                                    "expectedRevision": expected_entry["packetRevision"] if expected_entry else None,
                                    "observedComponentIds": observed,
                                    "expectedComponentIds": expected,
                                },
                            },
                            "priorInternalMatchEvent": prior,
                        },
                    },
                },
            })
        state = "matched" if reason == "internal-match" else "held"
        if state == "matched":
            matched[piece] = event_id
        events.append({
            "eventId": event_id, "pieceId": piece, "observedAt": row["observedAt"],
            "state": state, "reasonCode": reason,
            "disposition": {"rules": checks, "internalLedgerCanRecordMatch": state == "matched",
                            "carrierEligibilityStatus": "not-assessed", "externalSubmissionPerformed": False},
        })

    summary = {
        "observed": len(events), "matched": len(matched), "held": sum(e["state"] == "held" for e in events),
        "remaining": len(manifest) - len(matched), "carrierAccepted": None,
    }
    visual = {
        "summary": summary,
        "events": [{key: row[key] for key in ("eventId", "observedAt", "pieceId", "state", "reasonCode")} for row in events],
        "carrier": {key: CARRIER[key] for key in ("program", "status", "sourceUrl")},
    }
    case = {
        "id": fixture["caseId"],
        "schemaVersion": "1.0",
        "scope": {
            "operation": "internal-manifest-check",
            "sampleOnly": True,
            "physicalMailProduced": False,
            "carrierAccountAccessed": False,
            "reportedMetricsAreSample": True,
            "postalBarcodesPresent": False,
        },
        "ruleEvidence": {
            "authority": "sample-facility-policy",
            "policyVersion": policy,
            "externalProgramReference": CARRIER,
        },
        "facility": {
            "siteLabel": fixture["sampleSite"],
            "productionWindow": {
                "timeBasis": "UTC", "bounds": fixture["windowUtc"],
                "batch": {
                    "localManifestId": fixture["localBatch"],
                    "expectedPieces": fixture["manifest"],
                    "captureStream": {
                        "order": "input-time-order", "eventCount": len(events),
                        "decisions": events,
                    },
                    "internalLedger": {
                        "matchingObservationByPiece": matched,
                        "unmatchedPieceIds": sorted(set(manifest) - set(matched)),
                        "matchOperationCanSendMail": False,
                    },
                },
            },
        },
        "summary": summary,
    }
    return case, visual


def main():
    paths = ("operational-case.json", "visual.json")
    fixture = json.loads((HERE / "inputs.json").read_text(encoding="utf-8"))
    results = render(fixture)
    write = sys.argv[1:] == ["--write"]
    require(write or not sys.argv[1:], "Usage: python3 replay.py [--write]")
    for name, result in zip(paths, results):
        text = json.dumps(result, indent=2) + "\n"
        file = HERE / name
        if write:
            file.write_text(text, encoding="utf-8")
        elif file.read_text(encoding="utf-8") != text:
            raise SystemExit(f"{name}: result differs")
    print(f"{results[1]['summary']['matched']} internal matches; carrier acceptance not assessed")


if __name__ == "__main__":
    main()
