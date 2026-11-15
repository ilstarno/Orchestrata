# Solver milestone 1

This implementation is a generic finite-domain reference solver. It accepts variables and runtime-defined constraints, evaluates hard/soft scores, and enumerates assignments up to a fixed limit. It is deliberately not described as a production optimizer.

## Run

```sh
cd backend
cargo test --workspace
cargo run -p orchestrata-api
```

In another terminal:

```sh
curl -sS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

When running from `backend/`, use `@examples/assignment.json` instead.

The expected assignment is `[1,0]` with hard and soft scores of zero. N-Queens and assignment problems run through the same evaluator.

## Limits

Search is exhaustive and capped at 100,000 combinations over HTTP. Constraint evaluation is full recomputation. A negative hard score means infeasible; the API does not claim otherwise. There are no incremental score updates, move operators, shadow variables, persistence, connectors, authentication, or production resource isolation yet.

Next: separate compilation from evaluation, add typed constraint IR, construction heuristics and local search, then differential-test incremental scoring against this reference implementation.
