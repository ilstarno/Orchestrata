"""Independent assertions for the named example; does not import the planner."""
from collections import Counter
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent


def read(name):
    return json.loads((HERE / name).read_text(encoding="utf-8"))


def main():
    inputs = read("inputs.json")
    case = read("operational-case.json")
    visual = read("visual.json")
    batch = case["facility"]["productionWindow"]["batch"]
    decided = batch["captureStream"]["decisions"]
    expected = {
        "SCAN-001": ("held", "not-on-manifest", ["fail"]),
        "SCAN-002": ("held", "packet-mismatch", ["pass", "pass", "pass", "pass", "fail"]),
        "SCAN-003": ("held", "duplicate-component", ["pass", "pass", "pass", "fail"]),
        "SCAN-004": ("held", "packet-mismatch", ["pass", "pass", "pass", "pass", "fail"]),
        "SCAN-005": ("matched", "internal-match", ["pass"] * 5),
        "SCAN-006": ("held", "duplicate-release", ["pass", "fail"]),
        "SCAN-007": ("held", "packet-mismatch", ["pass", "pass", "pass", "pass", "fail"]),
        "SCAN-008": ("matched", "internal-match", ["pass"] * 5),
        "SCAN-009": ("held", "unreadable-packet", ["pass", "pass", "unknown"]),
    }
    for original, recorded, shown in zip(inputs["observations"], decided, visual["events"], strict=True):
        event = original["eventId"]
        assert (event, original["observedAt"], original["pieceId"]) == (
            recorded["eventId"], recorded["observedAt"], recorded["pieceId"]
        )
        assert all(recorded[key] == val for key, val in shown.items())
        state, reason, checks = expected[event]
        assert (recorded["state"], recorded["reasonCode"]) == (state, reason)
        internal = recorded["disposition"]
        assert internal["internalLedgerCanRecordMatch"] is (state == "matched")
        assert internal["carrierEligibilityStatus"] == "not-assessed"
        assert internal["externalSubmissionPerformed"] is False
        assert [rule["evaluation"]["outcome"] for rule in internal["rules"]] == checks + [
            "not-evaluated"
        ] * (5 - len(checks))
        for rule in internal["rules"]:
            assert (rule["authority"], rule["ruleRevision"]) == (
                "sample-facility-policy", inputs["policy"]["revision"]
            )
            captured = rule["evaluation"]["comparison"]["evidence"]["scan"]
            assert captured["capturedAt"] == original["observedAt"]
            assert captured["identifier"] == event
            assert captured["packetSnapshot"]["observedComponentIds"] == original["componentIds"]
    matched = Counter(row["pieceId"] for row in decided if row["state"] == "matched")
    assert matched == Counter({"PIECE-A": 1, "PIECE-B": 1})
    ledger = batch["internalLedger"]
    assert ledger["matchingObservationByPiece"] == {"PIECE-A": "SCAN-005", "PIECE-B": "SCAN-008"}
    assert ledger["unmatchedPieceIds"] == ["PIECE-C"]
    assert case["summary"] == visual["summary"] == {
        "observed": 9, "matched": 2, "held": 7, "remaining": 1, "carrierAccepted": None,
    }
    usps = case["ruleEvidence"]["externalProgramReference"]
    assert usps["sourceAuthority"] == "carrier-program-reference"
    assert usps["eligibilityDetermined"] is False
    assert usps["carrierResponseReceived"] is False
    assert case["scope"]["physicalMailProduced"] is False
    print("Verified nine observations, two unique matches, seven held observations, one unresolved piece; no carrier result.")


if __name__ == "__main__":
    main()
