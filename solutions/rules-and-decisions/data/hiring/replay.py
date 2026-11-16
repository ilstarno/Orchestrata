"""Replay scheduling availability gates and employer-level notice review, with no I/O except fixture files."""
from datetime import datetime, timedelta, timezone
import json
import re
from pathlib import Path
import sys
from zoneinfo import ZoneInfo

RECORD_FIELDS = {
    "interview_window": {"recordId", "recordKind", "officeCode", "appointmentStartUtc", "appointmentEndUtc",
                         "staffSlotCode", "snapshotAtUtc", "unavailableWindows"},
    "employer_notice_review": {"recordId", "recordKind", "officeCode", "employerScopeState", "noticeEvidenceState"},
}
WINDOW_FIELDS = {"windowId", "availabilityCode", "blockedFromUtc", "blockedUntilUtc"}


def instant(value):
    parsed = datetime.fromisoformat(value[:-1] + "+00:00" if value.endswith("Z") else value)
    if parsed.utcoffset() != timezone.utc.utcoffset(None):
        raise ValueError("Timestamps must be supplied in UTC")
    return parsed


def calendar_interval(from_utc, until_utc):
    start, end = instant(from_utc), instant(until_utc)
    if start >= end:
        raise ValueError("Calendar intervals must be positive")
    return start, end


def overlaps(left, right):
    return left[0] < right[1] and right[0] < left[1]


def check(rulebook, rule_id, status, evidence):
    rule = rulebook[rule_id]
    return {
        "ruleId": rule_id, "version": rule["revision"], "asOf": rule["verifiedAsOf"], "status": status,
        "authorityType": rule["authorityType"], "sourceIds": rule["sourceIds"], "evidence": evidence,
    }


def interview_window(record, policy, bundle, clock, rules):
    guard = policy["recruitingOperations"]["schedulingGuard"]
    office = guard["officeCalendar"]
    normal = guard["calendarFeed"]["normalization"]
    allowed_codes = normal["exportBoundary"]["exposureControls"]["reasonRedaction"]["allowedAvailabilityCodes"]
    if not re.fullmatch(r"SLOT-[0-9]{2}", record["staffSlotCode"]):
        raise ValueError("Staff slots must use opaque identifiers")
    for window in record["unavailableWindows"]:
        if set(window) != WINDOW_FIELDS or window["availabilityCode"] not in allowed_codes:
            raise ValueError("Only opaque scheduling windows are accepted")
        if not re.fullmatch(r"AV-[0-9]{3}", window["windowId"]):
            raise ValueError("Availability windows must use opaque identifiers")
    office_check = record["officeCode"] == office["scheduledOffice"]
    results = [check(rules, "HR-OPS-001", "pass" if office_check else "review",
                     {"submittedOffice": record["officeCode"], "approvedOffice": office["scheduledOffice"]})]
    if not office_check:
        return "confirm_office_scope", "Confirm the applicable office schedule before reusing this policy.", results

    start, end = calendar_interval(record["appointmentStartUtc"], record["appointmentEndUtc"])
    observed = instant(record["snapshotAtUtc"])
    age = int((clock - observed).total_seconds())
    minimum, maximum = normal["freshness"]["minimumAgeSeconds"], normal["freshness"]["maximumAgeSeconds"]
    fresh = minimum <= age <= maximum
    zone = ZoneInfo(office["localTimeZone"])
    local_date = start.astimezone(zone).date().isoformat()
    windows = []
    for window in record["unavailableWindows"]:
        window_range = calendar_interval(window["blockedFromUtc"], window["blockedUntilUtc"])
        if overlaps((start, end), window_range):
            windows.append(window["windowId"])

    closed_dates = [closure["observedLocalDate"] for closure in office["adoptedOfficeClosures"]]
    last_date = (end - timedelta(microseconds=1)).astimezone(zone).date().isoformat()
    matched_closure = sorted(day for day in closed_dates if local_date <= day <= last_date)
    results.append(check(rules, "HR-OPS-002", "block" if matched_closure else "pass",
                         {"officeLocalDate": local_date, "matchedClosureDates": matched_closure}))
    if matched_closure:
        return "hold_office_closed", "Find another date on the approved office calendar.", results

    results.append(check(rules, "HR-OPS-003", "pass" if fresh else "review",
                         {"snapshotAgeSeconds": age, "acceptableAgeSeconds": maximum, "actualCheckAtUtc": record["snapshotAtUtc"]}))
    if not fresh:
        return "refresh_availability", "Refresh the availability snapshot before proposing a panel.", results

    results.append(check(rules, "HR-OPS-004", "block" if windows else "pass",
                         {"opaqueStaffSlot": record["staffSlotCode"], "matchedWindowIds": sorted(windows),
                          "intervalSemantics": bundle["timeRules"]["occupancyIntervals"]}))
    if windows:
        return "hold_staff_unavailable", "Try another approved time or panel member without exposing why the slot is unavailable.", results
    return "propose_to_coordinator", "Coordinator reviews the slot and panel before sending an invitation.", results


def employer_notice_review(record, policy, rules):
    workflow = policy["recruitingOperations"]["employerNoticeWorkflow"]
    scope = record["employerScopeState"]
    if scope not in workflow["acceptedScopeStates"] or record["noticeEvidenceState"] not in workflow["acceptedEvidenceStates"]:
        raise ValueError("Unsupported employer-level review evidence")
    confirmed = scope == "coverage_confirmed_by_owner"
    supplied = record["noticeEvidenceState"] == "owner_reports_on_file"
    results = [
        check(rules, "HR-NOTICE-001", "scope_supplied" if confirmed else "review",
              {"employerScopeState": scope, "scopeOwner": workflow["ownerRole"]})
    ]
    if not confirmed:
        return "employer_scope_review", "Have the designated HR owner verify coverage, applicable notices, and policy before relying on this checklist.", results
    results.append(check(rules, "HR-NOTICE-002", "record_supplied" if supplied else "review",
                         {"evidenceState": record["noticeEvidenceState"], "automaticPosting": workflow["canPublishNotice"]}))
    if not supplied:
        return "notice_evidence_review", "HR checks the general posting and, where applicable, employee distribution; nothing is published by this replay.", results
    return "manual_notice_confirmation", "HR verifies posting visibility and any applicable distribution requirements. An evidence record is not a compliance conclusion.", results


def evaluate(case):
    packet = case["evidencePacket"]
    bundle = packet["operatingScope"]["policyBundle"]
    policy = bundle["companyPolicy"]
    clock = instant(packet["snapshotEvaluatedAtUtc"])
    rules = {rule["ruleId"]: rule for rule in case["ruleCatalog"]}
    decisions = []
    for record in case["inputRecords"]:
        kind = record["recordKind"]
        if kind not in RECORD_FIELDS or set(record) != RECORD_FIELDS[kind]:
            raise ValueError("Only explicitly allowed scheduling or employer-level fields are accepted")
        if not re.fullmatch(r"POL-[0-9]{3}", record["recordId"]) or not re.fullmatch(r"[A-Z]{2,8}-[A-Z0-9]{2,8}", record["officeCode"]):
            raise ValueError("Records and offices must use opaque identifiers")
        if kind == "interview_window":
            result, next_step, trail = interview_window(record, policy, bundle, clock, rules)
        else:
            result, next_step, trail = employer_notice_review(record, policy, rules)
        decisions.append({
            "recordId": record["recordId"], "workflowLane": kind, "decision": result,
            "reviewAction": next_step, "evaluationTimeUtc": packet["snapshotEvaluatedAtUtc"], "effectiveChecks": trail,
        })
    return decisions


def main():
    path = Path(__file__).with_name("operational-case.json")
    fixture = json.loads(path.read_text(encoding="utf-8"))
    actual = evaluate(fixture)
    if sys.argv[1:] == ["--write"]:
        fixture["decisionTrail"] = actual
        path.write_text(json.dumps(fixture, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    elif sys.argv[1:]:
        raise SystemExit("Use replay.py or replay.py --write")
    else:
        if actual != fixture["decisionTrail"]:
            raise SystemExit("Policy replay differs from the recorded decision trail")
        print(f"Verified {len(actual)} offline policy records.")


if __name__ == "__main__":
    main()
