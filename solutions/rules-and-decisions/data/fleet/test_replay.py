from copy import deepcopy
from unittest import TestCase

from replay import evaluate, read


class FleetGateTests(TestCase):
    def setUp(self):
        fixture = read('operational-case.json')
        self.rules = fixture['ruleset']
        self.cases = {row['id']: row for row in fixture['cases']}

    def test_unknown_truck_deviation_cannot_be_read_as_false_or_permission(self):
        row = deepcopy(self.cases['NYC-02'])
        segment = row['dispatch']['route']['validation']['segment']
        segment['closestIntersectionProof'] = None
        result = evaluate(row, self.rules)
        self.assertEqual(result['status'], 'review')
        self.assertEqual(result['reasons'][0]['sourceId'], 'nycTruck')
        self.assertFalse(result['dispatchAuthorized'])

    def test_loading_that_outlasts_the_vehicle_stay_cannot_pass(self):
        row = deepcopy(self.cases['LON-01'])
        site = row['dispatch']['site']
        site['serviceLocalEndMinute'] = 347  # vehicle exited at 05:45 local
        result = evaluate(row, self.rules)
        self.assertEqual(result['status'], 'hold')
        self.assertEqual(result['reasons'][0]['code'], 'LOADING_DOES_NOT_FIT_ACCESS_STAY')

    def test_wrong_zone_offset_refused_not_converted_silently(self):
        row = deepcopy(self.cases['NYC-01'])
        segment = row['dispatch']['route']['validation']['segment']
        segment['enteredAt'] = '2026-11-16T05:30:00+00:00'
        with self.assertRaisesRegex(ValueError, 'correct local offset'):
            evaluate(row, self.rules)

    def test_precedence_unknown_access_and_known_company_limit(self):
        row = deepcopy(self.cases['LON-03'])
        row['dispatch']['shift']['routeReturnMinute'] = 530
        result = evaluate(row, self.rules)
        self.assertEqual(result['status'], 'hold')
        self.assertEqual([(reason['layer'], reason['status']) for reason in result['reasons']],
                         [('authority', 'review'), ('company', 'blocked')])
        self.assertFalse(result['dispatchAuthorized'])
