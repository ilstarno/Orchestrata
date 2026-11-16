import unittest

from make_sample import make_sample, period
from normalize import normalize
from reproduce import demand_report, oracle


class NormalizationTests(unittest.TestCase):
    def test_prunes_by_full_shift_break_language_skill_and_channel(self):
        raw = make_sample()
        problem, checks, seconds, combinations = normalize(raw)
        self.assertEqual((seconds, combinations), (1800, 486))
        self.assertEqual([len(v["domain"]) for v in problem["variables"]], [3, 2, 3, 3, 3, 3])
        self.assertIn("shift_does_not_cover_window", checks[0]["excluded"]["OP-016"])
        self.assertIn("break_overlaps_window", checks[2]["excluded"]["OP-017"])
        self.assertIn("no_voice_capacity", checks[0]["excluded"]["OP-018"])
        self.assertIn("required_language_below_minimum", checks[1]["excluded"]["OP-003"])
        self.assertEqual(oracle(problem), (((0, 1, 3, 6, 9, 12), -7), 324))

    def test_timing_boundaries_are_half_open(self):
        raw = make_sample()
        raw["operators"][0]["shift"] = period(9, 30, 10, 30)
        raw["operators"][0]["breaks"] = [period(9, 45, 10, 0)]
        problem, _, _, _ = normalize(raw)
        self.assertIn(0, problem["variables"][0]["domain"])

    def test_rejects_duplicate_ids_invalid_rating_and_timezone_offset(self):
        for change in (
            lambda x: x["operators"][1].update(id=x["operators"][0]["id"]),
            lambda x: x["operators"][0]["skills"].update(billing=True),
            lambda x: x["coverage_window"].update(start="2026-11-16T10:00-04:00"),
            lambda x: x["policy"]["demand_scenarios_basis_points"].update(surge=2999),
            lambda x: x["policy"].update(evaluation_budget=485),
        ):
            raw = make_sample()
            change(raw)
            with self.assertRaises(ValueError):
                normalize(raw)

    def test_workload_is_calculated_but_not_part_of_assignment_request(self):
        raw = make_sample()
        before = normalize(raw)[0]
        report = demand_report(raw["queues"][0], raw["policy"], 1800)
        self.assertEqual(report["expected_arrivals"], 3.6)
        self.assertEqual(report["expected_handling_seconds"], 1296)
        self.assertEqual(report["expected_occupancy_basis_points_rounded"], 7200)
        self.assertEqual(report["probability_above_planned_capacity_basis_points"], 3000)
        raw["queues"][0]["arrival_counts"]["surge"] = 6
        self.assertEqual(normalize(raw)[0], before)
        self.assertEqual(demand_report(raw["queues"][0], raw["policy"], 1800)["expected_handling_seconds"], 1404)


if __name__ == "__main__":
    unittest.main()
