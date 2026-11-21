# Solve API reference

Send a planning problem from your application and receive assignments or an infeasible result. An **API** is the server interface your program calls over HTTP. The request and response use **JSON**, a text format for structured data.

## Connect to the local service

**Calling a running server does not require Rust, Cargo, an SDK or a source checkout.** You need a terminal, [curl](https://curl.se/docs/manpage.html) (a command-line HTTP client), and access to an Orchestrata server.

- **Already have a server?** Ask its operator for the base URL, permitted routes and access method. Use that server's instructions for authentication. The unauthenticated commands below are for the local sample server only.
- **No server yet?** Follow [Run the server from source](../backend-setup/), then return here. The website does not provide a public solve endpoint.
- **Just exploring?** [Read the two-job worked example](/Orchestrata/examples/assignment/) without installing anything. It shows a saved result; it does not start a server.

The commands use a POSIX shell on Linux, macOS or WSL. Open a fresh terminal; no particular working directory is needed. `127.0.0.1` means this computer. Set `API` to the local server's address and check that it responds:

```sh
curl --version
API=http://127.0.0.1:3000
curl -fsS "$API/api/v1/health"
```

Expected response:

```json
{"status":"ok"}
```

Keep this terminal open: later commands use the same `API` variable. Health confirms that the server responds; it does not solve a problem. The local server has no authentication. Keep it on loopback and use sample data.

## Your first solve

Two jobs need different resources. Each may use resource `0` or `1`; job A prefers `1`. Paste this complete request into the same terminal. It includes its input, so no example file or checkout is needed.

```sh
curl -fsS "$API/api/v1/solve" \
  -H 'Content-Type: application/json' \
  --data-binary @- <<'JSON'
{"variables":[{"name":"job_a","domain":[0,1]},{"name":"job_b","domain":[0,1]}],"constraints":[{"kind":"all_different","variables":[0,1],"hard_weight":1},{"kind":"prefer_value","variable":0,"value":1,"soft_weight":3}]}
JSON
```

Expected response:

```json
{"solution":{"values":[1,0],"score":{"hard":0,"soft":0},"evaluated":4}}
```

Read `values` in the input order: job A gets resource `1`, job B gets resource `0`. `hard: 0` means the required rules are satisfied. `soft: 0` means the preference is satisfied. `evaluated: 4` counts the candidate assignments examined. Resource numbers are your own identifiers; they do not imply dates, people or units until your application maps them.

## Check errors before integrating

An infeasible model and a broken request are different outcomes. Give both jobs only resource `[0]` to obtain `{"solution":null}`: no assignment can keep them different. An empty domain is invalid input instead:

```sh
curl -i -sS "$API/api/v1/solve" \
  -H 'Content-Type: application/json' \
  --data-binary '{"variables":[{"name":"empty","domain":[]}],"constraints":[]}'
```

Expect HTTP `422`, `X-Orchestrata-Error-Code: invalid_problem` and the text `Invalid problem`. The `X-Request-ID` value varies. Here `-i` shows headers and omitting `-f` keeps the error body visible.

| Symptom | Next step |
| --- | --- |
| Connection refused | Start the server and check its terminal; confirm the port in `API`. |
| 401 or 403 from an operator-managed server | Follow the operator's access instructions. Do not reuse the unauthenticated sample as an access workaround. |
| 404 | Check the base URL and `/api/v1/openapi.json` on that server. |
| 422 or a solve-limit error | Check the request and the limits below. A rejected request does not prove that the work is impossible. |
| `solution` is null | The exact search found no assignment satisfying all hard rules. Review the allowed resources and required rules. |

## Routes and contract

```sh
curl -fsS "$API/api/v1/openapi.json"
```

**OpenAPI** is the machine-readable description of the running server's endpoints and request fields. Use it to confirm capabilities before integrating. `GET` reads data; `POST` submits a request. The `/api/v1` prefix identifies this API version.

| Method and path | Purpose |
| --- | --- |
| `GET /api/v1/health` | Check reachability. |
| `GET /api/v1/openapi.json` | Inspect the running contract. |
| `POST /api/v1/solve` | Submit the JSON problem and read its `solution`. |
| `GET /api/v1/error-codes` | Look up error codes and their meanings. |

The reference below describes the local sample server. An operator's build or proxy may impose different limits. Recipe catalog operations have a [separate guide](../recipes-api/).

## Problem fields

All fields below are required, with no implicit defaults. An empty array is different from an omitted field. There are no request fields for a search budget, timeout, random seed, scheduling dates, or units. A plain integer such as `3` has only the meaning your model gives it.

| Field | JSON type | Allowed values and meaning |
| --- | --- | --- |
| `variables` | Array of objects | One object per decision, in the order you want the returned values. A constraint uses the variable’s array position. An empty array is permitted when the constraints do not refer to variables; it represents one empty assignment. |
| `variables[].name` | String | Required and nonblank. Names must be unique after trimming surrounding whitespace. Comparison is case-sensitive. Use names to label decisions in your client. The response uses input order. |
| `variables[].domain` | Array of signed 64-bit JSON integers | At least one value and no duplicates. The solver chooses exactly one value for each variable. Keep the intended order: for equally scored candidates, the first one encountered in declared domain order wins. |
| `constraints` | Array of objects | Use `[]` for no rules or preferences. Each object requires a `kind` tag and the fields for that kind. |

Indices start at zero and must refer to an existing variable. Use ordinary nonnegative JSON integers for indices, never numeric strings, decimals, or variable names. Avoid unspecified properties: the starter does not guarantee that unknown keys are rejected, so a typo in an extra key may not be noticed.

## Constraint fields

Each penalty is dimensionless. A higher positive weight produces a larger deduction in that score component. Hard score is compared before soft score; increasing a soft weight does not override a hard rule. Higher scores are better.

| `kind` | Additional required fields | Effect |
| --- | --- | --- |
| `all_different` | `variables`: nonempty array of distinct zero-based indices. `hard_weight`: positive signed 64-bit integer. | Deduct `hard_weight` for each pair of referenced variables with equal selected values. A single referenced variable has no pair and incurs no deduction. |
| `not_equal_offset` | `left`, `right`: zero-based indices. `offset`: signed 64-bit integer. `hard_weight`: positive signed 64-bit integer. | Deduct `hard_weight` when the selected value on the left minus the selected value on the right equals `offset`. Direction matters. Use `offset: 0` to forbid equality. |
| `prefer_value` | `variable`: zero-based index. `value`: signed 64-bit integer. `soft_weight`: positive signed 64-bit integer. | Deduct `soft_weight` if the variable’s selected value is not `value`. Check that the preferred value is in the domain if you want it to be attainable; a preferred value does not add an option. |

### Integer bounds

Domain members, offsets, preferred values and scores use the signed 64-bit range, from `-9223372036854775808` to `9223372036854775807`. Weights must be between `1` and `9223372036854775807` inclusive. The offset comparison compares the mathematical difference even when subtraction would not fit in a signed 64-bit integer. A sum outside that signed range fails; it never wraps or saturates. The minimum score itself is valid. On the starter build, overflow while checking any candidate can fail the entire request, even if another candidate could have been usable.

JavaScript’s ordinary `Number` cannot represent every 64-bit integer exactly. Stick to `-9007199254740991` through `9007199254740991` for values and scores if you use ordinary `JSON.parse` and number-based serialization, and keep combined penalties in that range, or use a lossless JSON tool on both sides. Quoting a number changes its JSON type and is not a substitute for lossless encoding.

## Worked request

Two jobs, two possible resources. They must use different resources. Among assignments that satisfy the rule, prefer giving the first job resource `1`.

```sh
curl -sS -i http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "variables": [
      {"name": "job_a", "domain": [0, 1]},
      {"name": "job_b", "domain": [0, 1]}
    ],
    "constraints": [
      {"kind": "all_different", "variables": [0, 1], "hard_weight": 1},
      {"kind": "prefer_value", "variable": 0, "value": 1, "soft_weight": 3}
    ]
  }'
```

Expected status: **200 OK**. Response header: `content-type: application/json`. Exact body:

```json
{"solution":{"values":[1,0],"score":{"hard":0,"soft":0},"evaluated":4}}
```

| Response field | Type | Interpretation |
| --- | --- | --- |
| `solution` | Object or null in the schema | An object contains the optimum found within the supplied rules. Null means the completed exact search proved there was no hard-feasible assignment. |
| `solution.values` | Array of signed 64-bit integers | One entry per input variable in the same order. Job A takes resource 1 and Job B takes resource 0. |
| `solution.score.hard` | Signed 64-bit integer | `0` means the assignment satisfies every required rule supplied. Under this exact-result contract, the solver only returns hard-feasible assignments. |
| `solution.score.soft` | Signed 64-bit integer | `0` means all supplied preferences are met. A negative value reports the combined deductions for unmet preferences. |
| `solution.evaluated` | Nonnegative JSON integer (the OpenAPI schema marks it as `int64` with minimum 0) | The number of assignments examined. The local API caps the number of combinations. |

**An input error or search limit is not proof of infeasibility.** To check the null response, give both jobs the sole domain `[0]` in the worked request. Their all-different rule makes the model infeasible. A build with this exact-result contract returns HTTP 200 and `{"solution":null}`. Reserve negative-hard candidates from older builds for diagnosis; do not apply them as plans.

## Status and response headers

| Status | `X-Orchestrata-Error-Code` | Client action |
| --- | --- | --- |
| `200 OK` | None | Read `solution` and distinguish an assignment from proved infeasibility. |
| `400`, `413`, `415`, or `422` | `invalid_request_body` | Check syntax, body size, content type, or JSON field types as appropriate. |
| `422 Unprocessable Entity` | `invalid_problem` | Correct the model. A variable with an empty domain is one example. |
| `422 Unprocessable Entity` | `search_limit_exceeded` | Reduce the number of combinations. No conclusion about feasibility was reached. |
| `500 Internal Server Error` | `solve_execution_failed` | Keep the request ID and the reproducible sample. An overflow in scoring can trigger this response. |
| `503 Service Unavailable` | `solver_busy` | Solve slots are occupied. Retry later with backoff. |

Read `GET /api/v1/error-codes` for the running build’s code catalog and keep `X-Request-ID` for correlation. Error bodies can be plaintext. This solve contract has no per-request timeout field or header.

## Size limits and troubleshooting

- The exact-search limit is **100,000 combinations**. Calculate the total by multiplying all domain lengths. Five variables with ten options each are within the cap; six such variables exceed it. The limit is checked before search. A model rejected on this basis produces no planning result.
- The tested starter build accepts a JSON body up to **2,097,152 bytes (2 MiB)**. This is independent of the combinations cap. A compact JSON body can encode a huge search; a model with few choices can still have a body too large to read. Your running server or a proxy may impose a different body limit.
- If `curl` cannot connect, start the API on the same machine and confirm that port 3000 is available. The project site URL is not the API base URL. If health works but a solve fails, confirm that the request body contains the file contents.
- If a result looks surprising, confirm zero-based indices and the order in `variables`, check for a preferred value outside the corresponding domain, and review hard and soft scores separately. If you intended an option to be forbidden, remove it from the domain. A preference only changes the score.
- If you exceed the limit, reduce the case to a smaller representative model or remove options that genuinely cannot be chosen. Do not drop hard rules to get an answer. No retry can turn an unchanged over-budget model into a proof.

## Scope of this version

The starter solves finite integer-domain assignments with the three constraint kinds above. It has no native model for time windows, live routing, cumulative vehicle capacity, calendars, deployment-specific sign-in, or persistent job submission. It does not forecast demand, dispatch people, update external systems, or return partial progress. All quantities and rules you rely on must be present in the input. A model can be valid without representing every rule that applies to the real work.
