# Cargo allocation: evidence bundle

This worked example uses sample data and demonstrates allocation against an explicit booking allowance, not vessel load approval or safe stowage. The six indivisible consignments each have two eligible vessels, so the native exact search evaluates 64 candidate assignments. The input's voyage window, reefer profiles, lot references and rationale are review context. Only explicit eligibility, mass, booking allowances and preferences are normalized into the solver problem.

| File | Purpose |
| --- | --- |
| `input.json` | Operational-style source. Explicit allow/reject decisions for every consignment–vessel pair, with supporting metadata kept separate. |
| `problem.json` | Exact normalized constraint model passed to the engine. |
| `expected.json` | Exact expected native response fixture. |
| `allocation-report.json` | Checked assignments, all eligibility decisions, vessel residuals, preference penalties, source SHA-256 digests and proof scope. |
| `reproduce.py` | Standard-library normalization, independent exhaustive oracle, native ABI call and report comparison. |
| `test_reproduce.py` | Source/model validation, report checks and an optional live native test. |

## Reproduce

With Python 3.10+ and the Rust toolchain installed, check out the standalone engine revision recorded under `proof.native.engine_revision`. From the engine checkout, build its native C ABI:

```sh
cargo build --manifest-path backend/Cargo.toml --locked -p orchestrata-ffi
```

From this bundle, substitute the path to that compiled shared library (Linux shown; use the corresponding `.dylib` or `.dll` on macOS or Windows) and the exact revision you built:

```sh
export ORCHESTRATA_CARGO_LIBRARY=/path/to/engine/backend/target/debug/liborchestrata_ffi.so
python3 reproduce.py --library "$ORCHESTRATA_CARGO_LIBRARY" \
  --engine-revision 0281da6328b8a02b52e35fafee7348e144b22b48
python3 -m unittest -v test_reproduce.py
```

The comparison includes an actual native call at cap 64, an actual native search-limit response at 63, and a separately implemented enumeration. The latter finds three hard-feasible assignments and exactly one optimum with zero missed-preference points. All 364 metric tonnes are assigned within the three modeled booking allowances. None of this verifies physical loading, cargo securing, hazardous-material segregation, reefer slot or temperature compliance, or any real voyage.
