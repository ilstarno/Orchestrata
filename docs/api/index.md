Send sample planning JSON to the starter API on your workstation. This static site provides no solver, sign-in, or request proxy.

## Start and call the service

Run this from the Orchestrata repository root:

```sh
(cd backend && cargo run -p orchestrata-api)
```

In another terminal, also from the repository root:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

| Method and path | Purpose |
| --- | --- |
| `GET /api/v1/health` | Check that the local HTTP process responds. |
| `POST /api/v1/solve` | Submit an integer-domain problem for a bounded exact solve. |
| `GET /api/v1/openapi.json` | Read the schema emitted by the running build. |

Every variable has a name and an array of allowed integers. Constraints reference positions in that array of variables, starting at zero. See the running schema for fields and tags. Keep integers lossless if submitting from a client that uses floating-point numbers by default.

## Read results and errors

The example returns a JSON object with a `solution` field. The returned values follow your variable order. Hard and soft scores are separate. The local starter caps exact search at 100,000 combinations. A request error, an exhausted limit, and a server failure do not prove the model infeasible. A build with the verified exact-result contract returns `{"solution":null}` only when exact search proves no plan satisfies the hard rules.

## Recipe and error catalog review work

The following paths are implemented in review or local integration work. See the [HTTP review #22](https://github.com/ilstarno/Orchestrata/pull/22) for the changes it contains; the merged set of local review paths may differ. Check the revision you run; do not assume they exist on a default checkout or on a hosted service.

| Method and path | Purpose |
| --- | --- |
| `GET /api/v1/error-codes` | Read stable error identifiers and their meanings. |
| `GET /api/v1/recipes` | List the bundled examples. |
| `GET /api/v1/recipes/{recipe_id}` | Read a recipe’s description and declared fields. |
| `GET /api/v1/recipes/{recipe_id}/example` | Return the sample problem, without running it. |
| `POST /api/v1/recipes/{recipe_id}/run-example` | Solve a bundled example with no request body. This path is local review work, not a released interface. |

A separate local MCP review build exposes `list_recipes`, `describe_recipe`, and `get_recipe_example` through standard input and output. It lists and reads sample cases only. It neither listens for network connections nor runs a solve. Do not expect hosted MCP, remote credentials, or sign-in from the local evaluation build.
