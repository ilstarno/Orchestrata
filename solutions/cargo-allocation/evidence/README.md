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

Use Python 3.10+ and a Rust toolchain. From this bundle, read the engine revision the native check used:

```sh
ENGINE_REVISION=$(python3 -c 'import json; from pathlib import Path; print(json.loads(Path("allocation-report.json").read_text())["proof"]["native"]["engine_revision"])')
```

Use a standalone engine checkout at that revision. Set its local path below, then build its native C ABI. The shared-library path is for Linux; use the corresponding `.dylib` or `.dll` on macOS or Windows.

```sh
ENGINE_DIR=/path/to/orchestrata-engine
test "$(git -C "$ENGINE_DIR" rev-parse HEAD)" = "$ENGINE_REVISION" &&
  cargo build --manifest-path "$ENGINE_DIR/backend/Cargo.toml" --locked -p orchestrata-ffi
export ORCHESTRATA_CARGO_LIBRARY="$ENGINE_DIR/backend/target/debug/liborchestrata_ffi.so"
python3 reproduce.py --library "$ORCHESTRATA_CARGO_LIBRARY" \
  --engine-revision "$ENGINE_REVISION"
python3 -m unittest -v test_reproduce.py
```

The comparison includes an actual native call at cap 64, an actual native search-limit response at 63, and a separately implemented enumeration. The latter finds three hard-feasible assignments and exactly one optimum with zero missed-preference points. All 364 metric tonnes are assigned within the three modeled booking allowances. None of this verifies physical loading, cargo securing, hazardous-material segregation, reefer slot or temperature compliance, or any real voyage.
