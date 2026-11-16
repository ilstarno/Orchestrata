"""Write the public worked-example input tables, preserving anonymous IDs."""
from datetime import date, timedelta
import json
from pathlib import Path

WEEK = date(2026, 11, 22)
DAYS = 7
HOLIDAY_DAY = 4
HERE = Path(__file__).resolve().parent


def clock(hours, minutes=0):
    return 60*hours+minutes


def ts(day, minutes=0):
    hour, minute = divmod(minutes, 60)
    return f"{WEEK+timedelta(days=day)}T{hour:02d}:{minute:02d}:00-05:00"


def window(day, first, last):
    return {"startLocal": ts(day, first), "endLocal": ts(day, last)}


def rule(identifier, category, description, scope, limitation, source=None):
    return {"ruleId": identifier, "category": category, "description": description,
            "applicability": scope, "limits": limitation,
            **({"source": source, "accessedOn": "2026-11-16"} if source else {"policyVersion": "ALB-RET-3"})}


RULES = [
    rule("US-FLSA-40", "federal", "Overtime for covered, nonexempt staff starts after 40 worked hours in the employer's fixed workweek.", "Fixture assumes FLSA coverage and nonexempt status for everyone.", "This calculates overtime minutes, not wages. Overtime is not a legal maximum on hours.", "https://www.dol.gov/agencies/whd/fact-sheets/23-flsa-overtime-pay"),
    rule("US-HOURS-WORKED", "federal", "Time not worked on paid leave does not increase hours worked for federal overtime. Only duty-free bona fide meals are subtracted.", "Paid absence and fully relieved meal intervals in this fixture.", "A missed meal or work during a break changes actual payable hours and requires payroll review.", "https://www.dol.gov/agencies/whd/fact-sheets/22-flsa-hours-worked"),
    rule("US-PAID-ABSENCE", "federal", "Paid holidays, sick leave and vacation do not count as hours worked for federal overtime.", "Preapproved paid absence recorded separately from worked time.", "Other applicable benefit obligations not inferred from general federal guidance.", "https://webapps.dol.gov/elaws/whd/flsa/hoursworked/screenEr14.asp"),
    rule("NY-161-REST", "state", "Preserve the designated 24-hour rest window for each scheduled mercantile worker.", "Assumed covered staff where establishment operates Sunday; designated windows already recorded.", "No exception, variance or new worker notice modeled. Do not treat this as weekly legal certification.", "https://www.nysenate.gov/legislation/laws/LAB/161"),
    rule("NY-162-NOON", "state", "For ordinary nonfactory shifts longer than six hours covering the noon window, require 30 uninterrupted minutes during 11:00–14:00.", "Only clearly covered daytime shifts in the modeled work patterns.", "No short-meal permit, waiver, or borderline shift interpretation.", "https://www.nysenate.gov/legislation/laws/LAB/162"),
    rule("NY-162-EVENING", "state", "For shifts starting before 11:00 and ending after 19:00, require an additional 20 minutes during 17:00–19:00.", "Ordinary mercantile shifts spanning both periods; fixture treats the dinner relief as paid.", "No short-meal permit or waiver. Paid treatment in this case is explicit company accounting policy.", "https://www.nysenate.gov/legislation/laws/LAB/162"),
    rule("NY-162-LATE", "state", "Require 45 uninterrupted minutes around the midpoint for modeled nonfactory shifts longer than six hours starting after 13:00.", "Sample late shifts start at 14:00 and end the same day.", "The statute also addresses qualifying shifts starting through 06:00. No overnight or night-shift cases are modeled.", "https://www.nysenate.gov/legislation/laws/LAB/162"),
    rule("NY-196B-APPROVED", "state", "Honor the already-approved protected paid leave block.", "The example assumes a private employer with 128 employees and uses approved blocks only; state rules for employers with 100 or more include up to 56 paid hours per calendar year.", "May not establish entitlement, require diagnosis, compute balance, or decide leave eligibility; those decisions stay outside the plan.", "https://www.nysenate.gov/legislation/laws/LAB/196-B"),
    rule("NY-142-SCOPE", "state-reference", "Miscellaneous Industries Wage Order is a payroll implementation reference.", "Input scope assumes applicable Albany-area retail work under the miscellaneous order.", "The solver does not calculate wage rates, premium interactions, spread-of-hours, call-in pay or payroll. Company fixture allows one shift/day and no schedule over 10 hours/day.", "https://dol.ny.gov/minimum-wage-order-miscellaneous-industries-and-occupations-cr142"),
    rule("US-NY-HOLIDAY", "calendar-reference", "Thanksgiving Day is November 26 in 2026.", "U.S. federal public holiday calendar and separate NY public holiday statute.", "The calendar does not establish private closure or holiday premiums. SITE-A closure and SITE-B limited hours are company inputs.", "https://www.opm.gov/policy-data-oversight/pay-leave/federal-holidays/"),
    rule("CO-HOLIDAY-HOURS", "company", "SITE-A closes on Thanksgiving. SITE-B opens 09:00–17:00 for selected short inventory and pickup operations.", "Fixture company retail operating-hours policy only.", "This is not a government closure or holiday-pay mandate."),
    rule("CO-APPROVED-PTO", "company", "Accepted vacation blocks are unavailable for scheduling.", "Approved company PTO windows; limited data disclosed.", "No statutory vacation accrual or carryover rules modeled."),
    rule("CO-QUALIFICATION", "company", "Only staff with the requested training code hold that role.", "Keyholders, checkout, pickup, floor, and stock training tags.", "Training is a known roster fact, not a worker assessment."),
    rule("CO-SITE", "company", "Staff must be on the pre-cleared site roster.", "Cross-store workers have an explicitly allowed site; otherwise flagged.", "Does not assess legal work authorization or eligibility."),
    rule("CO-AVAILABILITY", "company", "Work must fit reported time availability. Operational unavailability is a hard block.", "Horizon and disruption input only; no leave reason is inferred.", "No prediction of future availability; no schedule writes."),
    rule("CO-ONE-SHIFT", "company", "Only one shift per day per worker in this case.", "Avoids split shifts in the bounded fixture.", "Not asserted as state or federal law."),
    rule("CO-MEAL-RELIEF", "company", "A planned break needs a pre-cleared relief resource reference so it can be duty-free.", "Relief references represent operations-confirmed spare capacity, outside this scheduler's resource pool.", "Does not optimize or validate the relief resource roster; supervisors confirm actual release."),
    rule("CO-LOCKED", "company", "Keep confirmed appointments and flag any newly invalid locked input for a supervisor.", "Imported immutable staffing commitments.", "A closure or unavailability can require a refreshed roster; do not silently change the lock."),
    rule("CO-BASELINE-FIRST-FIT", "company", "Baseline selects the first permitted named worker without comparing overtime.", "Comparison baseline only.", "This policy does not describe a real employer or claim a measured improvement."),
    rule("CO-PLAN-OBJECTIVE", "company", "Cover as many shifts as possible, then minimize new overtime, then keep prior assignments where possible.", "Uses the scheduling priorities stated in the example.", "Overtime remains permissible and counted; no salary, cost, candidate quality, or hiring decisions."),
]


def staff(identifier, roles, sites, rest_day, leave=(), unavailable=()):
    return {"workerId": identifier, "roleCodes": roles, "approvedSiteIds": sites,
            "availabilityByDate": [window(day, clock(7), clock(22)) for day in range(7)],
            "protectedRest": {"window": {"startLocal": ts(rest_day), "endLocal": ts(rest_day+1)},
                              "recordRef": f"REST-{identifier}", "designationRecorded": True},
            "approvedBlocks": list(leave), "otherUnavailability": list(unavailable),
            "classification": "synthetic-covered-nonexempt-adult", "assumedHoursOutsideFixture": 0}


def leave(identifier, day, start, stop, kind):
    return {"blockId": identifier, **window(day, start, stop), "category": kind,
            "paidMinutes": stop-start, "approvalRecorded": True}


ROSTER = [
    staff("W01", ["stock", "keyholder"], ["SITE-B"], 6),
    staff("W02", ["checkout"], ["SITE-A"], 4),
    staff("W03", ["checkout", "pickup"], ["SITE-A", "SITE-B"], 0, [
        leave("LEV-03-TH", 4, clock(11), clock(16), "company-vacation"),
        leave("LEV-03-FR", 5, clock(8), clock(16), "protected-paid")]),
    staff("W04", ["stock", "keyholder"], ["SITE-B"], 2, [
        leave("LEV-04-SA", 6, clock(9), clock(17), "company-vacation")]),
    staff("W05", ["keyholder", "checkout"], ["SITE-A", "SITE-B"], 4),
    staff("W06", ["checkout", "floor"], ["SITE-A", "SITE-B"], 4, [
        leave("LEV-06-SA", 6, clock(8), clock(16), "company-vacation")]),
    staff("W07", ["stock"], ["SITE-A", "SITE-B"], 0),
    staff("W08", ["floor", "pickup"], ["SITE-A", "SITE-B"], 1),
    staff("W09", ["pickup"], ["SITE-B"], 0),
    staff("W10", ["keyholder"], ["SITE-B"], 0),
]


def meal(start, stop, code, relief="REL-STAFF-DUTY-FREE", paid=False):
    return {"startMinuteLocal": start, "endMinuteLocal": stop, "mealCode": code,
            "paid": paid, "reliefRef": relief}


DAY_MEAL = [meal(clock(12), clock(12,30), "noon")]
LATE_MEAL = [meal(clock(17,30), clock(18,15), "midpoint")]
LONG_MEALS = [*DAY_MEAL, meal(clock(17,30), clock(17,50), "additional-evening", paid=True)]


def shift(ident, site, day, role, first, last, *, assigned=None, pool=(), meals=(), queue=None):
    return {"shiftId": ident, "siteId": f"SITE-{site}", "serviceDate": str(WEEK+timedelta(days=day)),
            "requestedRole": role, **window(day, first, last), "meals": list(meals),
            "fixedWorkerId": assigned, "namedWorkerPool": list(pool), "demandQueue": queue or "store-coverage",
            "countUnit": "one_staff_slot", "expectedTaskBatch": role+"-duty"}


FIXED = [
    shift(f"F-B-STOCK-{day+1:02}", "B", day, "stock", clock(9), clock(17) if day!=HOLIDAY_DAY else clock(14),
          assigned="W01", meals=[] if day==HOLIDAY_DAY else DAY_MEAL) for day in (0,1,2,3,4,5)
] + [
    shift(f"F-A-CHECK-{day+1:02}", "A", day, "checkout", clock(8), clock(16),
          assigned="W02", meals=DAY_MEAL) for day in (0,1,2,3,5)
] + [
    shift("F-A-KEY-01", "A", 0, "keyholder", clock(9), clock(17), assigned="W05", meals=DAY_MEAL),
    shift("F-A-FLOOR-01", "A", 0, "floor", clock(10), clock(16), assigned="W08"),
    shift("F-B-KEY-02", "B", 1, "keyholder", clock(9), clock(17), assigned="W04", meals=DAY_MEAL),
    shift("F-B-CHECK-02", "B", 1, "checkout", clock(8), clock(16), assigned="W06", meals=DAY_MEAL),
    shift("F-A-KEY-03", "A", 2, "keyholder", clock(14), clock(21,45), assigned="W05", meals=LATE_MEAL),
    shift("F-A-STOCK-03", "A", 2, "stock", clock(10), clock(16), assigned="W07"),
    shift("F-A-PICK-03", "A", 2, "pickup", clock(10), clock(16), assigned="W08"),
    shift("F-A-KEY-04", "A", 3, "keyholder", clock(9), clock(17), assigned="W05", meals=DAY_MEAL),
    shift("F-A-FLOOR-04", "A", 3, "floor", clock(9), clock(15), assigned="W06"),
    shift("F-B-PICK-04", "B", 3, "pickup", clock(10), clock(16), assigned="W09"),
    shift("F-A-FLOOR-06", "A", 5, "floor", clock(15), clock(20), assigned="W08"),
    shift("F-A-KEY-07", "A", 6, "keyholder", clock(9), clock(17), assigned="W05", meals=DAY_MEAL),
    shift("F-B-PICK-07", "B", 6, "pickup", clock(15), clock(20), assigned="W08"),
    shift("F-B-KEY-01", "B", 0, "keyholder", clock(9), clock(17), assigned="W04", meals=DAY_MEAL),
    shift("F-B-PICK-02", "B", 1, "pickup", clock(11), clock(16), assigned="W09"),
    shift("F-A-STOCK-07", "A", 6, "stock", clock(10), clock(16), assigned="W07"),
]
OPEN = [
    shift("O-01", "B", 4, "pickup", clock(11), clock(16), pool=["W03","W09"], queue="holiday-pickup"),
    shift("O-02", "A", 4, "checkout", clock(12), clock(16), pool=["W02","W03"], queue="unreconciled-holiday-request"),
    shift("O-03", "A", 5, "keyholder", clock(10,30), clock(19,30), pool=["W05","W10"], meals=LONG_MEALS, queue="promotion-keys"),
    shift("O-04", "A", 5, "checkout", clock(8), clock(16), pool=["W02","W03","W06"], meals=DAY_MEAL, queue="promotion-checkout"),
    shift("O-05", "B", 5, "stock", clock(14), clock(21,45), pool=["W01","W04","W07"], meals=LATE_MEAL, queue="promotion-restock"),
    shift("O-06", "B", 5, "pickup", clock(15), clock(20), pool=["W08","W09"], queue="online-pickup"),
    shift("O-07", "A", 6, "checkout", clock(8), clock(16), pool=["W02","W03","W06"], meals=DAY_MEAL, queue="weekend-checkout"),
    shift("O-08", "B", 6, "keyholder", clock(9), clock(17), pool=["W01","W04","W05","W10"], meals=DAY_MEAL, queue="weekend-keys"),
    shift("O-09", "A", 6, "floor", clock(9), clock(17), pool=["W06","W07","W08"], meals=DAY_MEAL, queue="weekend-floor"),
    shift("O-10", "A", 6, "checkout", clock(9), clock(16), pool=["W02","W03"], meals=[meal(clock(15),clock(15,30),"noon")], queue="unreconciled-break-request"),
]


def weekly_hours(site):
    return [{"serviceDate": str(WEEK+timedelta(days=day)),
             "calendarTag": "thanksgiving" if day==HOLIDAY_DAY else ("promotion" if day==5 else "regular"),
             "openWindows": [] if day==HOLIDAY_DAY and site=="A" else
                 [window(day, clock(9), clock(17)) if day==HOLIDAY_DAY else window(day,clock(8),clock(22))]}
            for day in range(7)]


if __name__ == "__main__":
    data = {
        "asOfLocal": ts(0, clock(7)), "analysisDate": "2026-11-16", "timeZone": "America/New_York",
        "calendarWeekStartLocal": ts(0), "calendarWeekEndLocal": ts(7),
        "fixedEmployerWorkweekStartLocal": ts(0), "fixedEmployerWorkweekEndLocal": ts(7),
        "scope": {"countryCode": "US", "stateCode": "NY", "county": "Albany",
                  "businessType": "private non-factory retail", "corporateHeadcountAssumed": 128,
                  "workersCoveredAssumed": True, "coveredByFLSAAssumed": True, "operatesSundayAssumed": True,
                  "unionRulesIncluded": False, "approvedVarianceIncluded": False, "realEmployeeRecords": False,
                  "timeUnits": "integer minutes; intervals half-open; date inputs carry UTC offset"},
        "companyCalendar": {"weeklySiteHours": [{"siteId":"SITE-"+site,"hours":weekly_hours(site)} for site in ("A","B")],
                            "holiday": {"name": "Thanksgiving Day", "date": str(WEEK+timedelta(days=HOLIDAY_DAY)),
                                        "calendarRule": "US-NY-HOLIDAY", "operatingRule": "CO-HOLIDAY-HOURS",
                                        "stateCalendarSource": "https://www.nysenate.gov/legislation/laws/GCN/24",
                                        "premiumPayModeled": False}},
        "ruleRegister": RULES, "workerRoster": ROSTER, "fixedShiftDemand": FIXED, "unresolvedShiftDemand": OPEN,
        "disruptionSnapshot": [
            {"workerId":"W06",**window(5,clock(7),clock(22)),"signalId":"UNAV-FRI-01"},
            {"workerId":"W10",**window(6,clock(7),clock(22)),"signalId":"UNAV-SAT-01"},
        ],
    }
    (HERE/"inputs.json").write_text(json.dumps(data, indent=2)+"\n",encoding="utf-8")
