"""Regression tests for the auditable synthetic cargo fixture."""

from copy import deepcopy
import os
from pathlib import Path
import unittest

from reproduce import exhaustive_oracle, generate_report, native_solve, normalize, read_json


class CargoFixture(unittest.TestCase):
    """Check the data transformation, independent oracle, and optional live ABI."""

    def setUp(self):
        self.raw = read_json("input.json")
        self.problem = read_json("problem.json")
        self.expected = read_json("expected.json")
        self.report = read_json("allocation-report.json")

    def test_fixture_normalizes_to_checked_problem(self):
        self.assertEqual(normalize(self.raw), self.problem)

    def test_supporting_metadata_does_not_enter_model(self):
        raw = deepcopy(self.raw)
        raw["supporting_context"]["lane"] = "different review-only description"
        raw["vessels"][0]["supporting"]["reefer_profiles"] = ["frozen"]
        raw["consignments"][0]["supporting"]["storage_profile"] = "changed metadata"
        raw["consignments"][0]["eligibility"][0]["reason"] = "reworded explanation"
        self.assertEqual(normalize(raw), self.problem)

    def test_invalid_eligibility_is_rejected(self):
        raw = deepcopy(self.raw)
        raw["consignments"][0]["eligibility"].append(raw["consignments"][0]["eligibility"][0])
        with self.assertRaisesRegex(ValueError, "one eligibility decision required per vessel"):
            normalize(raw)

    def test_preferred_vessel_must_be_eligible(self):
        raw = deepcopy(self.raw)
        raw["consignments"][0]["preference"]["vessel_id"] = "boreal"
        with self.assertRaisesRegex(ValueError, "preferred vessel is not eligible"):
            normalize(raw)

    def test_independent_oracle_proves_unique_winner(self):
        self.assertEqual(
            exhaustive_oracle(self.problem),
            (64, 3, 0, [tuple(self.expected["solution"]["values"])]),
        )

    def test_report_explains_all_shipments_and_reconciles_totals(self):
        self.assertEqual([v["residual_t"] for v in self.report["vessels"]], [3, 10, 23])
        self.assertEqual(self.report["totals"]["assigned_mass_t"], 364)
        self.assertEqual(sum(shipment["assigned_mass_t"] for shipment in self.report["shipments"]), 364)
        self.assertEqual(len(self.report["shipments"]), 6)
        for shipment in self.report["shipments"]:
            self.assertEqual(len(shipment["eligibility_detail"]), 3)
            rejected = [decision for decision in shipment["eligibility_detail"] if not decision["allowed"]]
            self.assertEqual(len(rejected), 1)
            self.assertTrue(rejected[0]["reason"])
            self.assertIn(shipment["assigned_vessel_id"], shipment["eligible_vessel_ids"])

    @unittest.skipUnless(os.getenv("ORCHESTRATA_CARGO_LIBRARY"), "set ORCHESTRATA_CARGO_LIBRARY for actual native ABI comparison")
    def test_native_result_and_limit_match_report(self):
        library = Path(os.environ["ORCHESTRATA_CARGO_LIBRARY"])
        problem_bytes = (Path(__file__).resolve().parent / "problem.json").read_bytes()
        native = native_solve(library, problem_bytes, 64)
        revision = self.report["proof"]["native"]["engine_revision"]
        self.assertEqual(generate_report(self.raw, self.problem, self.expected, revision, native), self.report)


if __name__ == "__main__":
    unittest.main()
