# Orchestrata

**Planning and optimization for problems with competing constraints.**

Orchestrata is an optimization engine under development. It models decisions as variables, evaluates hard and soft constraints, and searches for better assignments. The engine is independent of the web application and is intended to be reusable from other programs through a native interface.

Typical planning problems include workforce allocation, timetabling, field service, routing, and resource scheduling. These are development targets, not claims of current support.

## Current capabilities

The reference engine supports finite integer domains and three constraint types:

- All-different assignments
- Unequal values with an offset
- Preferred values with a weighted penalty

It provides exhaustive reference search, bounded local search, and a deterministic tabu-search implementation. Scores are lexicographic: hard penalties take precedence over soft penalties. The engine is still an early implementation and has not been benchmarked against established optimization solvers.

The native interface currently exposes a smaller subset of the engine's constraints. The Go SDK wraps that interface; it is not yet a complete representation of the engine.

## Repository

| Directory | Purpose |
| --- | --- |
| `backend/crates/solver/` | Standalone optimization engine |
| `backend/crates/ffi/` | C-compatible native library and public header |
| `backend/crates/core/` | Shared typed primitives |
| `backend/crates/hygiene/` | Input validation foundation |
| `backend/crates/application/` | Application-facing orchestration |
| `backend/crates/api/` | HTTP adapter |
| `backend/crates/cli/` | Command-line adapter |
| `sdk/go/` | Go native SDK |
| `frontend/` | Development interface |
| `docs/` | Architecture and project documentation |

The dependency direction is from adapters to the engine. The engine does not depend on the HTTP service, UI, or Go SDK.

## Run locally

Requires a Rust toolchain with support for the 2024 edition.

```sh
cd backend
cargo test --workspace --all-targets
cargo run -p orchestrata-cli -- --help
```

To build the native library:

```sh
cd backend
cargo build -p orchestrata-ffi --release
```

The public C header is at `backend/crates/ffi/include/orchestrata.h`. The Go SDK build instructions are in [sdk/go/README.md](sdk/go/README.md).

To run the development interface, use Node.js 22:

```sh
cd frontend
npm install
npm run dev
```

## Engineering expectations

- Typed boundaries and explicit ownership
- Independent validation of solver results
- Reproducible tests and measurable benchmarks
- No hidden score overflow or silent constraint relaxation
- Tests kept separate from production source
- Each enum in its own file; module entry points contain no business logic
- No claims of optimality without proof

The current search algorithms are baselines. Incremental scoring, compiled constraints, richer planning variables, and broader problem-class coverage remain in development.

## Contributing

Please open an issue describing the problem and expected behavior before substantial changes. Keep changes focused and include tests. Code ownership is defined in [CODEOWNERS](.github/CODEOWNERS).

## License

No open-source license has been published for this repository yet. Until a license is added, the repository's contents are not licensed for redistribution or modification by others.
