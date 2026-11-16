"""Offline deterministic exception-work scheduler, not a payment engine."""
from datetime import datetime, timedelta
from pathlib import Path
from zoneinfo import ZoneInfo
import argparse
import itertools
import json

HERE = Path(__file__).resolve().parent
EXPLANATIONS = {
    'PLANNED_INTERNAL_WORK': 'All listed work stages can be planned by the applicable internal due time with qualified staff.',
    'SEPARATE_APPLICABILITY_REVIEW': 'The intake lane is consumer-related or unclear. Route this to a qualified owner without deciding rights, remedies, or deadlines.',
    'EVIDENCE_MISSING_OR_REJECTED': 'The supporting operational record has not been accepted for this work. Do not assign a preparer or reviewer.',
    'EVIDENCE_EXPIRES_BEFORE_MINIMUM_WORK': 'The accepted evidence will expire before even the shortest complete task can end.',
    'DUE_WINDOW_TOO_SHORT': 'Even without competing work, no complete plan fits between arrival and the internal due time.',
    'NO_QUALIFIED_COMPLETE_PLAN': 'There is no complete qualified assignment within the available time. No partial or next-window handoff is scheduled.',
}


def read(filename):
    return json.loads((HERE / filename).read_text(encoding='utf-8'))


def attestation(case):
    return case['operationsRecord']['intakeSnapshot']['workflow']['workItem']['linkedEvidence']['evidenceEnvelope']['manifest']['attestation']


def overlap(a, b):
    return a[0] < b[1] and b[0] < a[1]


def plan(fixture, scenario):
    policy = fixture['companyPolicy']
    step = policy['timeStepMinutes']
    analysts = fixture['staffing']['analysts']
    windows = {window['windowId']: window for window in fixture['returnWindows']}
    buffers = {ref: window['networkTransmissionDeadline'] - policy['transmissionBufferMinutes'] for ref, window in windows.items()}
    busy = {a['analystRef']: list(a['unavailableLocalMinutes']) for a in analysts}
    for outage in scenario['outages']:
        busy[outage['analystRef']].append(outage['windowLocalMinutes'])

    def deadline(case):
        ref = case['targetNetworkWindowRef']
        return min(case['serviceDueLocalMinute'], buffers[ref]) if ref else case['serviceDueLocalMinute']

    def possibilities(stage, earliest, latest, forbidden):
        duration = stage['minutes']
        for analyst in analysts:
            ref = analyst['analystRef']
            if ref == forbidden or stage['requiredQualification'] not in analyst['qualifications']:
                continue
            shift_start, shift_end = analyst['shiftLocalMinutes']
            start = max(earliest, shift_start)
            start = ((start + step - 1) // step) * step
            for time in range(start, min(latest, shift_end) - duration + 1, step):
                interval = [time, time + duration]
                if not any(overlap(interval, other) for other in busy[ref]):
                    yield {'stage': stage['stage'], 'requiredQualification': stage['requiredQualification'],
                           'analystRef': ref, 'workIntervalLocalMinutes': interval}

    def candidates(stages, earliest, latest, previous_ref=None):
        if not stages:
            yield []
        else:
            for first in possibilities(stages[0], earliest, latest, previous_ref):
                next_start = first['workIntervalLocalMinutes'][1]
                for rest in candidates(stages[1:], next_start, latest, first['analystRef']):
                    yield [first, *rest]

    decisions = []
    for case in sorted(fixture['cases'], key=lambda case: (deadline(case), case['priorityRank'], case['caseId'])):
        due = deadline(case)
        evidence = attestation(case)
        from_minute = case['availableAtLocalMinute']
        until = evidence['validity']['validThroughLocalMinute']
        duration = sum(stage['minutes'] for stage in case['workSteps'])
        steps = []
        if case['applicability']['intakeLane'] != 'nonconsumer_operations':
            reason = 'SEPARATE_APPLICABILITY_REVIEW'
        elif evidence['status'] != 'verified' or evidence['validity']['observedAtLocalMinute'] > from_minute:
            reason = 'EVIDENCE_MISSING_OR_REJECTED'
        elif from_minute + duration > until:
            reason = 'EVIDENCE_EXPIRES_BEFORE_MINIMUM_WORK'
        elif from_minute + duration > due:
            reason = 'DUE_WINDOW_TOO_SHORT'
        else:
            combinations = candidates(case['workSteps'], from_minute, min(due, until))
            steps = min(combinations, key=lambda assigned: (
                assigned[-1]['workIntervalLocalMinutes'][1],
                tuple((item['workIntervalLocalMinutes'][0], item['analystRef']) for item in assigned),
            ), default=[])
            reason = 'PLANNED_INTERNAL_WORK' if steps else 'NO_QUALIFIED_COMPLETE_PLAN'
        for item in steps:
            busy[item['analystRef']].append(item['workIntervalLocalMinutes'])
        status = 'scheduled' if steps else 'manual_review' if reason == 'SEPARATE_APPLICABILITY_REVIEW' else 'held'
        decisions.append({
            'caseId': case['caseId'], 'label': case['label'], 'status': status, 'reasonCode': reason, 'explanation': EXPLANATIONS[reason],
            'workType': case['workType'], 'availableAtLocalMinute': from_minute,
            'serviceDueLocalMinute': case['serviceDueLocalMinute'], 'internalDueLocalMinute': due,
            'targetNetworkWindowRef': case['targetNetworkWindowRef'],
            'networkTransmissionDeadline': windows.get(case['targetNetworkWindowRef'], {}).get('networkTransmissionDeadline'),
            'workAssignments': steps, 'paymentAuthorized': False, 'consumerRightsDetermined': False,
            'audit': {'control': {'caseTrace': {'intake': {'workEvidence': {'envelope': {'validation': {'attestation': evidence}}}},
                                  'companyPolicyRef': policy['rulesetId'],
                                  'intakeLane': case['applicability']['intakeLane']}}}
        })
    return {
        'scenarioId': scenario['scenarioId'], 'counterfactual': scenario['counterfactual'], 'outages': scenario['outages'],
        'counts': {status: sum(result['status'] == status for result in decisions) for status in ('scheduled','held','manual_review')},
        'analystUtilization': [
            {'analystRef': a['analystRef'], 'plannedIntervalCount': sum(item['analystRef'] == a['analystRef'] for row in decisions for item in row['workAssignments']),
             'plannedWorkMinutes': sum(item['workIntervalLocalMinutes'][1] - item['workIntervalLocalMinutes'][0]
                                       for row in decisions for item in row['workAssignments'] if item['analystRef'] == a['analystRef'])}
            for a in analysts
        ], 'decisions': sorted(decisions, key=lambda row: row['caseId']),
    }


def verify(fixture, scenario, output):
    """Proof checks operate on final assignments without entering the scheduling loop."""
    inputs = {row['caseId']: row for row in fixture['cases']}
    refs = {a['analystRef']: a for a in fixture['staffing']['analysts']}
    windows = {window['windowId']: window['networkTransmissionDeadline'] for window in fixture['returnWindows']}
    zones = {a: [] for a in refs}
    assert sorted(inputs) == [row['caseId'] for row in output['decisions']]
    assert sum(output['counts'].values()) == len(inputs)
    for row in output['decisions']:
        assert not row['paymentAuthorized'] and not row['consumerRightsDetermined']
        original = inputs[row['caseId']]
        status, assigned = row['status'], row['workAssignments']
        assert bool(assigned) == (status == 'scheduled')
        if status == 'manual_review':
            assert original['applicability']['intakeLane'] != 'nonconsumer_operations'
        if not assigned:
            continue
        assert original['applicability']['intakeLane'] == 'nonconsumer_operations'
        original_evidence = attestation(original)
        assert original_evidence['status'] == 'verified'
        assert original_evidence['validity']['observedAtLocalMinute'] <= original['availableAtLocalMinute']
        assert len(assigned) == len(original['workSteps'])
        assert len({stage['analystRef'] for stage in assigned}) == len(assigned)
        lower_bound = original['availableAtLocalMinute']
        hard_limit = min(original['serviceDueLocalMinute'], original_evidence['validity']['validThroughLocalMinute'])
        if original['targetNetworkWindowRef']:
            hard_limit = min(hard_limit, windows[original['targetNetworkWindowRef']] - fixture['companyPolicy']['transmissionBufferMinutes'])
        for step, operational in zip(assigned, original['workSteps']):
            assert step['stage'] == operational['stage']
            resource = refs[step['analystRef']]
            assert operational['requiredQualification'] in resource['qualifications']
            interval = step['workIntervalLocalMinutes']
            begin, end = interval
            assert begin % fixture['companyPolicy']['timeStepMinutes'] == 0
            assert end - begin == operational['minutes']
            assert resource['shiftLocalMinutes'][0] <= begin < end <= resource['shiftLocalMinutes'][1]
            assert lower_bound <= begin < end <= hard_limit
            lower_bound = end
            excluded = resource['unavailableLocalMinutes'] + [outage['windowLocalMinutes'] for outage in scenario['outages'] if outage['analystRef'] == step['analystRef']]
            assert not any(overlap(interval, other) for other in excluded)
            zones[step['analystRef']].append(interval)
    for intervals in zones.values():
        for first, second in itertools.combinations(intervals, 2):
            assert not overlap(first, second)


def artifacts(fixture):
    timestamp = datetime.fromisoformat(fixture['scope']['replayDate']).replace(tzinfo=ZoneInfo(fixture['scope']['timezone']))
    outputs = [plan(fixture, scenario) for scenario in fixture['scenarios']]
    for scenario, output in zip(fixture['scenarios'], outputs):
        verify(fixture, scenario, output)
    return {'caseId': fixture['id'], 'schemaVersion': 1, 'scope': fixture['scope'],
            'replayTimeReference': timestamp.isoformat(), 'replayTimeEnds': (timestamp + timedelta(days=1)).isoformat(),
            'primarySources': fixture['primarySources'], 'returnWindows': fixture['returnWindows'],
            'companyPolicy': fixture['companyPolicy'], 'scenarios': outputs}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write-traces', action='store_true')
    args = parser.parse_args()
    output = artifacts(read('operational-case.json'))
    line = json.dumps(output, indent=2) + '\n'
    path = HERE / 'decision-traces.json'
    if args.write_traces:
        path.write_text(line, encoding='utf-8')
    elif line != path.read_text(encoding='utf-8'):
        raise SystemExit('Checked-in traces differ from replay')
    counts = {row['scenarioId']: row['counts'] for row in output['scenarios']}
    print(json.dumps(counts, sort_keys=True))


if __name__ == '__main__':
    main()
