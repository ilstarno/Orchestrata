from copy import deepcopy
from unittest import TestCase

from replay import artifacts, plan, read, verify


class PaymentExceptionTests(TestCase):
    def setUp(self):
        self.fixture = read('operational-case.json')
        self.normal, self.disruption = self.fixture['scenarios']

    def test_counterfactual_reconciles_counts_without_reclassifying_cases(self):
        normal, disruption = artifacts(self.fixture)['scenarios']
        self.assertEqual(normal['counts'], {'scheduled': 15, 'held': 8, 'manual_review': 1})
        self.assertEqual(disruption['counts'], {'scheduled': 13, 'held': 10, 'manual_review': 1})
        lost = {a['caseId'] for a, b in zip(normal['decisions'], disruption['decisions'])
                if a['status'] == 'scheduled' and b['status'] == 'held'}
        self.assertEqual(lost, {'PX-007', 'PX-019'})
        for scenario in (normal, disruption):
            decisions = {row['caseId']: row for row in scenario['decisions']}
            for ref in ('PX-013', 'PX-014', 'PX-015'):
                self.assertEqual(decisions[ref]['status'], 'held')
            self.assertEqual(decisions['PX-016']['status'], 'manual_review')
            self.assertEqual(decisions['PX-003']['internalDueLocalMinute'], 615)
            self.assertEqual(decisions['PX-003']['networkTransmissionDeadline'], 630)
            self.assertEqual(decisions['PX-003']['targetNetworkWindowRef'], 'RET-AM')

    def test_independent_verifier_rejects_self_review(self):
        output = deepcopy(plan(self.fixture, self.normal))
        selected = next(row for row in output['decisions'] if len(row['workAssignments']) == 2)
        first, second = selected['workAssignments']
        second['analystRef'] = first['analystRef']
        with self.assertRaises(AssertionError):
            verify(self.fixture, self.normal, output)

    def test_independent_verifier_rejects_assignment_to_unavailable_analyst(self):
        output = plan(self.fixture, self.normal)
        with self.assertRaises(AssertionError):
            verify(self.fixture, self.disruption, output)

    def test_independent_verifier_rejects_past_cutoff(self):
        output = deepcopy(plan(self.fixture, self.normal))
        selected = next(row for row in output['decisions'] if len(row['workAssignments']) == 2)
        interval = selected['workAssignments'][-1]['workIntervalLocalMinutes']
        minutes = interval[1] - interval[0]
        interval[:] = [selected['internalDueLocalMinute'], selected['internalDueLocalMinute'] + minutes]
        with self.assertRaises(AssertionError):
            verify(self.fixture, self.normal, output)
