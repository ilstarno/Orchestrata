# Orchestrata quickstart

See the [web quickstart](site/index.html) for a worked planning example. The developer API binds locally by default. It is not intended for unauthenticated public access.

Start from the repository root with a stable Rust toolchain:

```sh
(cd backend && cargo run -p orchestrata-api)
```

From a second terminal:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

The bundled example returns `[1, 0]`. A successful response with `"solution": null` means no assignment meets every hard rule. A request or search-limit error does not prove infeasibility. Inspect `/api/v1/openapi.json` on the same server for request and result schemas.

For local CLI usage:

```sh
(cd backend && cargo run -p orchestrata-cli -- examples/assignment.json)
```
