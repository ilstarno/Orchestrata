"""Check the recorded outcomes and perturb privacy, scope, and time boundaries."""
from copy import deepcopy
import json
from pathlib import Path

from replay import evaluate


fixture = json.loads(Path(__file__).with_name("operational-case.json").read_text(encoding="utf-8"))
expected = {
    "POL-001": "hold_office_closed",
    "POL-002": "hold_staff_unavailable",
    "POL-003": "propose_to_coordinator",
    "POL-004": "refresh_availability",
    "POL-005": "confirm_office_scope",
    "POL-006": "employer_scope_review",
    "POL-007": "notice_evidence_review",
    "POL-008": "manual_notice_confirmation",
}


def by_id(decisions):
    return {row["recordId"]: row["decision"] for row in decisions}


def refuses(case):
    try:
        evaluate(case)
    except ValueError:
        return
    raise AssertionError("Expected an unsupported scheduling export to be rejected")


assert by_id(evaluate(fixture)) == expected
sources = {source["sourceId"]: source for source in fixture["sourceCatalog"]}
rules = {rule["ruleId"]: rule for rule in fixture["ruleCatalog"]}
for decision in fixture["decisionTrail"]:
    assert decision["reviewAction"]
    for check in decision["effectiveChecks"]:
        rule = rules[check["ruleId"]]
        assert check["version"] == rule["revision"]
        assert check["asOf"] == rule["verifiedAsOf"]
        assert set(check["sourceIds"]) <= sources.keys()
        assert check["evidence"]
assert fixture["decisionTrail"][0]["effectiveChecks"][-1]["authorityType"] == "company_policy_with_federal_date_context"
assert fixture["decisionTrail"][0]["effectiveChecks"][-1]["evidence"]["matchedClosureDates"] == ["2026-11-26"]
assert fixture["decisionTrail"][1]["effectiveChecks"][-1]["evidence"]["matchedWindowIds"] == ["AV-001"]
assert fixture["decisionTrail"][2]["effectiveChecks"][-1]["evidence"]["matchedWindowIds"] == []
assert fixture["decisionTrail"][3]["effectiveChecks"][-1]["evidence"]["snapshotAgeSeconds"] == 601

without_private_adoption = deepcopy(fixture)
without_private_adoption["evidencePacket"]["operatingScope"]["policyBundle"]["companyPolicy"]["recruitingOperations"][
    "schedulingGuard"]["officeCalendar"]["adoptedOfficeClosures"].clear()
assert by_id(evaluate(without_private_adoption))["POL-001"] == "propose_to_coordinator"

ends_at_midnight = deepcopy(fixture)
ends_at_midnight["inputRecords"][0]["appointmentStartUtc"] = "2026-11-26T04:50:00Z"
ends_at_midnight["inputRecords"][0]["appointmentEndUtc"] = "2026-11-26T05:00:00Z"
assert by_id(evaluate(ends_at_midnight))["POL-001"] == "propose_to_coordinator"

notice_only = deepcopy(fixture)
notice_only["inputRecords"][7]["employerScopeState"] = "unconfirmed"
notice_only_results = by_id(evaluate(notice_only))
assert notice_only_results["POL-008"] == "employer_scope_review"
assert list(notice_only_results.items())[:5] == list(expected.items())[:5]

unknown_field = deepcopy(fixture)
unknown_field["inputRecords"][1]["unavailableWindows"][0]["absenceReason"] = "restricted"
refuses(unknown_field)

nonopaque = deepcopy(fixture)
nonopaque["inputRecords"][1]["staffSlotCode"] = "staff name"
refuses(nonopaque)

print("Verified eight dispositions, rule provenance, calendar adoption and midnight bounds, notice isolation, and rejection of unexpected or nonopaque fields.")
