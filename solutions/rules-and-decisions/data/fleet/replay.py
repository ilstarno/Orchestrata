"""Replay bounded candidate-delivery gates; no address, routing or external traffic feed."""
import json
import sys
from datetime import date, datetime, timedelta
from pathlib import Path
from zoneinfo import ZoneInfo

HERE = Path(__file__).resolve().parent
EXPLANATIONS = {
    'STREET_PROOF_STALE_OR_UNKNOWN': 'The route evidence is unconfirmed or does not remain valid for the full planned visit.',
    'TRUCK_CLASSIFICATION_UNKNOWN': 'Truck network applicability cannot be verified from axle and tire records.',
    'UNSUPPORTED_TRUCK_DEVIATION': 'The candidate does not use the confirmed nearest truck-route intersection for an off-network destination.',
    'TRUCK_DEVIATION_PROOF_UNKNOWN': 'The off-network destination or nearest connection has not been verified.',
    'TRUCK_NETWORK_STATUS_UNKNOWN': 'The selected street is not confirmed as part of the modeled local truck network.',
    'DELIVERY_WORK_CONTEXT_UNVERIFIED': 'Applicability of the adult employee delivery bicycle requirements is not verified.',
    'BIKE_REQUIREMENT_UNVERIFIED': 'The company cannot confirm the listed delivery bicycle readiness check.',
    'BIKE_REQUIREMENT_FAILED': 'The listed delivery bicycle readiness requirement is not met.',
    'OXFORD_CYCLE_ENTRY_PROHIBITED': 'This cycle candidate enters the traffic-free Oxford Street section, where cycles are not allowed.',
    'OXFORD_ACCESS_WINDOW_MISSED': 'The vehicle cannot complete both entry and exit within the Oxford Street access window.',
    'OXFORD_ACCESS_ROUTE_NOT_APPROVED': 'The approach is known not to be one of the approved access approaches.',
    'OXFORD_ACCESS_ROUTE_UNVERIFIED': 'A daytime restriction does not lapse by implication: approval of this approach has not been verified.',
    'OXFORD_STOP_NOT_FOR_LOADING': 'The planned stop is for waiting rather than loading or unloading.',
    'LOADING_DOES_NOT_FIT_ACCESS_STAY': 'The service window cannot fit between the proposed entry and exit.',
    'CYCLE_SEGMENT_CLASS_UNKNOWN': 'The street or track classification is unknown and cannot be relied on for cycling access.',
    'ORDINARY_PAVEMENT_CYCLING_PROHIBITED': 'The segment is designated for pedestrians, not for cycling or shared use.',
    'RESOURCE_CAPACITY_EXCEEDED': 'The order is heavier or larger than this delivery resource is allowed to carry.',
    'RESOURCE_CAPACITY_UNKNOWN': 'The cargo or resource capacity is not confirmed.',
    'COMPANY_EXPERIENCE_MINIMUM_NOT_MET': 'The operator does not meet the company minimum for this mode.',
    'COMPANY_EXPERIENCE_UNKNOWN': 'The operator experience record is not confirmed.',
    'COMPANY_SHIFT_BUFFER_MISSED': 'This candidate leaves too little time for depot return or falls outside the company shift envelope.',
    'HUB_HANDOFF_NOT_SIGNED': 'The company has not signed the hub handoff.',
    'HUB_HANDOFF_UNKNOWN': 'The hub references or handoff record are not confirmed.',
    'RECEIVING_SLOT_NOT_BOOKED': 'The receiving site says this booking is not confirmed.',
    'RECEIVING_SLOT_UNVERIFIED': 'There is no verified receiving slot for the proposed visit.',
    'RECEIVING_WINDOW_MISSED': 'The service interval does not fit the receiving slot.',
    'COMPANY_COLD_CHAIN_SENSOR_FAILED': 'The required chilled-cargo sensor check did not pass.',
    'COMPANY_COLD_CHAIN_SENSOR_UNKNOWN': 'The required chilled-cargo sensor check has not been verified.',
}


def read(name):
    return json.loads((HERE / name).read_text())


def timestamp(value, timezone):
    raw = datetime.fromisoformat(value)
    if raw.tzinfo is None or raw.utcoffset() != raw.astimezone(timezone).utcoffset():
        raise ValueError('Timestamp must use the correct local offset')
    return raw.astimezone(timezone)


def minute(stamp):
    return stamp.hour * 60 + stamp.minute


def evaluate(row, ruleset):
    city = row['city']
    dispatch = row['dispatch']
    order, operator, resource = (dispatch[key] for key in ('order', 'operator', 'resource'))
    shift, site, route = (dispatch[key] for key in ('shift', 'site', 'route'))
    mode = resource['mode']
    segment = route['validation']['segment']
    authority = ruleset['authority']['jurisdictions'][city]
    legal = {rule['id']: rule for rule in authority['rules']}
    timezone = ZoneInfo(authority['timezone'])
    company = ruleset['company']
    planned = date.fromisoformat(route['replayDate'])
    entered, exited = (timestamp(segment[key], timezone) for key in ('enteredAt', 'exitedAt'))
    if entered > exited or entered.date() != planned or exited.date() != planned:
        raise ValueError('Example expects same-day segment entry and exit in order')
    if mode not in company['minimumRouteExperienceMonths']:
        raise ValueError('Unknown delivery mode')
    reasons = []
    evaluated = []

    def note(rule, status, code):
        scope = 'authority' if rule in legal else 'company'
        reason = {'ruleId': rule, 'layer': scope, 'status': status, 'code': code, 'detail': EXPLANATIONS[code]}
        if scope == 'authority':
            reason['sourceId'] = legal[rule]['sourceId']
        reasons.append(reason)

    def check(rule, value, code, unknown, *, fail_code=None):
        evaluated.append(rule)
        if value is not True:
            note(rule, 'review' if value is None else 'blocked', unknown if value is None else fail_code or code)

    proof = segment['evidence']['capture']['verification']
    validity = proof['validityWindow']
    observed, verified, expires = (timestamp(validity[key], timezone) for key in ('recordedAt', 'verifiedAt', 'validUntil'))
    max_age = timedelta(hours=company['proofWindowHours'])
    in_force = date.fromisoformat(company['effectiveFrom']) <= planned <= date.fromisoformat(company['effectiveThrough'])
    proof_fresh = (observed <= verified <= entered <= exited <= expires and exited - observed <= max_age)
    check('EVIDENCE_VALID_AT_DELIVERY', True if in_force and route['jurisdictionBoundaryVerified'] is True and
          proof['status'] == 'confirmed' and proof_fresh else None, '', 'STREET_PROOF_STALE_OR_UNKNOWN')

    if city == 'NYC' and mode == 'vehicle':
        rule = 'NYC_TRUCK_NETWORK'
        axles, tires = resource['axles'], resource['tires']
        if not all(type(v) is int and v > 0 for v in (axles, tires)):
            check(rule, None, '', 'TRUCK_CLASSIFICATION_UNKNOWN')
        elif axles >= 3 or (axles == 2 and tires == 6):
            network = segment['networkMembership']
            if network == 'local_truck_route':
                check(rule, True, '', '')
            elif network == 'off_network':
                proofs = (segment['offNetworkDestinationVerified'], segment['closestIntersectionProof'])
                verdict = False if False in proofs else True if all(x is True for x in proofs) else None
                check(rule, verdict, 'UNSUPPORTED_TRUCK_DEVIATION', 'TRUCK_DEVIATION_PROOF_UNKNOWN')
            else:
                check(rule, None, '', 'TRUCK_NETWORK_STATUS_UNKNOWN')
        elif (axles, tires) != (2, 4):
            check(rule, None, '', 'TRUCK_CLASSIFICATION_UNKNOWN')
    if city == 'NYC' and mode == 'bicycle':
        rule = 'NYC_BIKE_READINESS'
        if operator['commercialEmployee'] is not True:
            check(rule, None, '', 'DELIVERY_WORK_CONTEXT_UNVERIFIED')
        else:
            readiness = operator['bicycleReadiness']
            evaluated.append(rule)
            for item in legal[rule]['requiredChecks']:
                value = readiness.get(item)
                if value is not True:
                    note(rule, 'review' if value is None else 'blocked',
                         'BIKE_REQUIREMENT_UNVERIFIED' if value is None else 'BIKE_REQUIREMENT_FAILED')
                    reasons[-1]['check'] = item
    if city == 'LON' and segment['kind'] == 'oxford_traffic_free' and mode in ('vehicle', 'bicycle'):
        rule = 'LON_OXFORD_ACCESS'
        limits = legal[rule]
        if mode == 'bicycle':
            check(rule, False, 'OXFORD_CYCLE_ENTRY_PROHIBITED', '')
        else:
            check(rule, limits['accessStartMinute'] <= minute(entered) <= minute(exited) < limits['accessEndMinuteExclusive'],
                  'OXFORD_ACCESS_WINDOW_MISSED', '')
            check(rule, segment['tflApprovedAccess'], 'OXFORD_ACCESS_ROUTE_NOT_APPROVED', 'OXFORD_ACCESS_ROUTE_UNVERIFIED')
            check(rule, order['purpose'] == limits['requiredVehiclePurpose'], 'OXFORD_STOP_NOT_FOR_LOADING', '')
            check(rule, minute(entered) <= site['serviceLocalStartMinute'] <= site['serviceLocalEndMinute'] <= minute(exited),
                  'LOADING_DOES_NOT_FIT_ACCESS_STAY', '')
    if city == 'LON' and mode == 'bicycle' and segment['kind'] != 'oxford_traffic_free':
        rule = 'LON_PAVEMENT'
        if segment['kind'] not in ('pedestrian_pavement', 'shared_use_track', 'public_road'):
            check(rule, None, '', 'CYCLE_SEGMENT_CLASS_UNKNOWN')
        else:
            check(rule, segment['kind'] != legal[rule]['prohibitedSegment'], 'ORDINARY_PAVEMENT_CYCLING_PROHIBITED', '')

    check('LOAD_LIMIT', order['massKg'] <= resource['maxPayloadKg'] and order['volumeL'] <= resource['maxVolumeL'],
          'RESOURCE_CAPACITY_EXCEEDED', 'RESOURCE_CAPACITY_UNKNOWN')
    experience = operator['experienceMonths']
    check('LOCAL_ORIENTATION', None if experience is None else experience >= company['minimumRouteExperienceMonths'][mode],
          'COMPANY_EXPERIENCE_MINIMUM_NOT_MET', 'COMPANY_EXPERIENCE_UNKNOWN')
    check('SHIFT_ENVELOPE', 0 <= shift['startMinute'] <= minute(entered) <= minute(exited) <= shift['routeReturnMinute']
          <= shift['endMinute'] - company['returnBufferMinutes'] and
          shift['endMinute'] - shift['startMinute'] <= company['maximumShiftMinutes'],
          'COMPANY_SHIFT_BUFFER_MISSED', '')
    check('HUB_HANDOFF', shift['signedHandoff'] if shift['hubStartRef'] and shift['hubEndRef'] else None,
          'HUB_HANDOFF_NOT_SIGNED', 'HUB_HANDOFF_UNKNOWN')
    if company['bookingRequired']:
        check('BOOKING_PROOF', site['booked'] if site['bookingRef'] else None, 'RECEIVING_SLOT_NOT_BOOKED', 'RECEIVING_SLOT_UNVERIFIED')
        check('BOOKING_WINDOW', site['windowStartMinute'] <= site['serviceLocalStartMinute'] <=
              site['serviceLocalEndMinute'] < site['windowEndMinuteExclusive'], 'RECEIVING_WINDOW_MISSED', '')
    if company['coldChainSensorRequired'] and order['requiresChilled']:
        check('COLD_CHAIN_SENSOR', resource['chilledSensorVerified'],
              'COMPANY_COLD_CHAIN_SENSOR_FAILED', 'COMPANY_COLD_CHAIN_SENSOR_UNKNOWN')
    status = 'hold' if any(item['status'] == 'blocked' for item in reasons) else 'review' if reasons else 'passes_modeled_checks'
    return {'id': row['id'], 'city': city, 'mode': mode, 'status': status,
            'dispatchAuthorized': False, 'appliedRuleIds': list(dict.fromkeys(evaluated)), 'reasons': reasons}


def trace_output(fixture, checks):
    ruleset = fixture['ruleset']
    by_id = {check['id']: check for check in checks}
    results = []
    for row in fixture['cases']:
        check = by_id[row['id']]
        context = row['dispatch']
        segment = context['route']['validation']['segment']
        legal = ruleset['authority']['jurisdictions'][row['city']]
        findings = check['reasons']
        if findings:
            why = ' '.join(item['detail'] for item in findings)
        else:
            why = 'The modeled rule checks passed with current evidence. This is not permission to dispatch.'
        results.append({
            **check, 'title': row['title'], 'explanation': why,
            'plannedSegmentEntry': segment['enteredAt'], 'plannedSegmentExit': segment['exitedAt'],
            'authorityReferences': [
                {'ruleId': rule['id'], 'sourceId': rule['sourceId']} for rule in legal['rules']
                if rule['id'] in check['appliedRuleIds']
            ],
            'companyPolicyRef': ruleset['company']['id'],
            'auditPack': {'planning': {'dispatchContext': {
                'plannedTimezone': legal['timezone'], 'caseReplayedForDate': context['route']['replayDate'],
                'operatorRef': context['operator']['operatorRef'], 'dispatchOrderRef': context['order']['orderRef'],
                'assetRef': context['resource']['assetRef'], 'siteRef': context['site']['siteRef'],
                'cargoReview': {key: context['order'][key] for key in ('cargoCategory', 'massKg', 'volumeL', 'requiresChilled')},
                'truckOrCarryCapacity': {key: context['resource'][key] for key in ('maxPayloadKg', 'maxVolumeL')},
                'receivingProof': {key: context['site'][key] for key in ('bookingRef', 'booked', 'windowStartMinute', 'windowEndMinuteExclusive')},
                'shiftWindow': {key: context['shift'][key] for key in ('startMinute','endMinute','routeReturnMinute','hubStartRef','hubEndRef','signedHandoff')},
                'validatedCandidate': {'routeGate': {'routeRef': context['route']['routeRef'], 'boundaryVerified': context['route']['jurisdictionBoundaryVerified'],
                    'segmentScope': {'roadSegmentId': segment['roadSegmentId'], 'segmentKind': segment['kind'], 'direction': segment['direction'],
                        'permissionSnapshot': {'attestation': segment['evidence']['capture']['verification']}}}}
            }}}
        })
    return {'caseId': fixture['id'], 'schemaVersion': 1, 'scope': fixture['scope'],
            'sources': ruleset['authority']['sources'], 'countsByStatus': {
                result: sum(check['status'] == result for check in checks)
                for result in ('passes_modeled_checks', 'hold', 'review')
            }, 'decisions': results}


def main():
    fixture = read('operational-case.json')
    checks = [evaluate(row, fixture['ruleset']) for row in fixture['cases']]
    reference = read('expected-decisions.json')
    assert len({check['id'] for check in checks}) == len(checks)
    observed = {row['id']: [row['status'], [reason['code'] for reason in row['reasons']]] for row in checks}
    assert observed == reference['decisionsById'], observed
    assert all(not row['dispatchAuthorized'] for row in checks)
    assert all(reasons['sourceId'] in fixture['ruleset']['authority']['sources'] for row in checks for reasons in row['reasons']
               if reasons['layer'] == 'authority')
    serialized = json.dumps(trace_output(fixture, checks), ensure_ascii=False, indent=2) + '\n'
    trace_path = HERE / 'decision-traces.json'
    if '--write-traces' in sys.argv:
        trace_path.write_text(serialized, encoding='utf-8')
    elif trace_path.read_text(encoding='utf-8') != serialized:
        raise SystemExit('Checked-in decision traces differ from this replay')
    if '--json' in sys.argv:
        print(serialized, end='')
    else:
        print('Verified', len(checks), 'candidate gates:',
              ', '.join(f'{sum(row["status"] == result for row in checks)} {result}' for result in ('passes_modeled_checks', 'hold', 'review')))


if __name__ == '__main__':
    main()
