# Solve API reference

Use this endpoint when you have a small set of decisions, a finite list of allowable values for each decision, and rules to compare assignments. This reference describes the local starter API on [Orchestrata’s default branch](https://github.com/ilstarno/Orchestrata). It separates that contract from changes still under review.

## Connect to the local service

| Setting | Value for the starter API |
| --- | --- |
| Prerequisites | An Orchestrata source checkout and a current stable Rust toolchain. The examples also use a shell and `curl`. |
| Base URL | `http://127.0.0.1:3000`. This is loopback on the machine running the service. |
| Request body | UTF-8 JSON. Send `Content-Type: application/json` when posting. |
| Authentication | None in this local starter. Do not expose it to the internet or submit customer or employee data during evaluation. |
| Transport | Local HTTP. The project website is static and does not run or proxy the service. |
| Version | Beginning of each path: `/api/v1`. Read the OpenAPI document from the running build to confirm its contract. |

Start the service at the checkout root:

```sh
(cd backend && cargo run -p orchestrata-api)
```

| Method and path | Request | Successful response |
| --- | --- | --- |
| `GET /api/v1/health` | No body | `{"status":"ok"}`. Confirms the process responds, not that a plan can be found. |
| `GET /api/v1/openapi.json` | No body | JSON schema for the health and solve operations. The schema document does not list its own URL as an operation. |
| `POST /api/v1/solve` | Problem JSON, described below | A JSON object containing `solution`. Check its hard score before using the assignment. |

If you use the CLI from the same checkout, the [assignment example](https://github.com/ilstarno/Orchestrata/blob/9dab0545beb8cb9ded93cc632e41f0b6bc718e77/backend/examples/assignment.json) is also available as a file: `(cd backend && cargo run -p orchestrata-cli -- examples/assignment.json)`. The CLI prints the solution directly rather than wrapping it in an HTTP response.

## Problem fields

All fields below are required, with no implicit defaults. An empty array is different from an omitted field. There are no request fields for a search budget, timeout, random seed, scheduling dates, or units. A plain integer such as `3` has only the meaning your model gives it.

| Field | JSON type | Allowed values and meaning |
| --- | --- | --- |
| `variables` | Array of objects | One object per decision, in the order you want the returned values. A constraint refers to a position in this array, not to a variable name. An empty array is permitted when the constraints do not refer to variables; it represents one empty assignment. |
| `variables[].name` | String | Required and nonblank. Names must be unique after trimming surrounding whitespace. Comparison is case-sensitive. Use these names to label decisions in your own client; the response returns positions rather than names. |
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

Domain members, offsets, preferred values and scores use the signed 64-bit range, from `-9223372036854775808` to `9223372036854775807`. Weights must be between `1` and `9223372036854775807` inclusive. The offset comparison compares the mathematical difference even when subtraction would not fit in a signed 64-bit integer. A sum of deductions outside that signed range fails instead of wrapping or saturating. The minimum score itself is valid. On the starter build, overflow while checking any candidate can fail the entire request, even if another candidate could have been usable.

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
| `solution` | Object or null in the schema | Inspect the returned object and its score. Do not infer proof semantics from the nullable schema alone. |
| `solution.values` | Array of signed 64-bit integers | One entry per input variable in the same order. Job A takes resource 1 and Job B takes resource 0. |
| `solution.score.hard` | Signed 64-bit integer | `0` means the assignment satisfies the required rules you supplied. A negative value means one or more were broken. |
| `solution.score.soft` | Signed 64-bit integer | `0` means all supplied preferences are met. A negative value reports the combined deductions for unmet preferences. |
| `solution.evaluated` | Nonnegative JSON integer (the OpenAPI schema marks it as `int64` with minimum 0) | The number of assignments examined. It is capped by the local API budget, not a duration. |

**Do not use a negative-hard result as a valid plan.** On the starter API, the solver enumerates the allowed combinations and can return HTTP 200 with the highest-scoring assignment even when every candidate violates a hard rule. For example, giving both jobs the sole domain `[0]` returns a best candidate with a negative hard score; for the same request above with both domains changed to `[0]`, the body is `{"solution":{"values":[0,0],"score":{"hard":-1,"soft":-3},"evaluated":1}}`. A result-handling change under review returns `solution: null` when the completed search proves infeasibility. Do not write clients that assume that review behavior is already in the starter.

## Status and response headers

| Status | Starter behavior | Client action |
| --- | --- | --- |
| `200 OK` | JSON result; not necessarily a valid plan. | Parse the body and check `score.hard`. |
| `400 Bad Request` | JSON syntax is malformed. | Correct the JSON before retrying. |
| `413 Payload Too Large` | The JSON body exceeds the accepted request size. | Send a smaller valid body; this is not the combinations limit. |
| `415 Unsupported Media Type` | No supported JSON request content type. | Set `Content-Type: application/json`. |
| `422 Unprocessable Entity` | JSON has the wrong shape or types, or the application rejects the problem. The application's plaintext body is `Invalid problem or search space too large` for invalid models, excess combinations and solver score overflow. | Check the input first, then calculate its combinations and inspect weights. Do not classify an HTTP 422 as proof of infeasibility. |
| `500 Internal Server Error` | The server could not complete execution. The application's plaintext body is `Solver execution failed`. | Preserve the status and the reproducible synthetic input. Do not treat it as a planning outcome. |

Framework and application errors are generally plaintext rather than solution JSON; check content type before parsing. On the starter API there is no promised application error-code header, request-ID header or per-request timeout header. A [public HTTP change under review](https://github.com/ilstarno/Orchestrata/pull/22) is not a guarantee that your running build has the same status details or optional headers. Check its exact revision and its live OpenAPI document before integrating against an expanded contract.

## Size limits and troubleshooting

- The exact-search limit is **100,000 combinations**. Multiply the length of every variable’s domain, rather than adding lengths or counting rules. Five variables with ten options each are within the cap; six such variables exceed it. The limit is checked before search. A model rejected on this basis produces no planning result.
- The tested starter build accepts a JSON body up to **2,097,152 bytes (2 MiB)**. This is independent of the combinations cap. A compact JSON body can encode a huge search; a model with few choices can still have a body too large to read. Other revisions or a proxy may impose a different body limit.
- If `curl` cannot connect, start the API on the same machine and confirm that port 3000 is available. The project site URL is not the API base URL. If health works but a solve fails, confirm that you are sending the file contents, not a shell path as plain text.
- If a result looks surprising, confirm zero-based indices and the order in `variables`, check for a preferred value outside the corresponding domain, and review hard and soft scores separately. If you intended an option to be forbidden, remove it from the domain rather than simply preferring another value.
- If you exceed the limit, reduce the case to a smaller representative model or remove options that genuinely cannot be chosen. Do not drop hard rules to get an answer. No retry can turn an unchanged over-budget model into a proof.

## Scope of this version

The starter solves finite integer-domain assignments with the three constraint kinds above. It has no native model for time windows, live routing, cumulative vehicle capacity, calendars, deployment-specific sign-in, or persistent job submission. It does not forecast demand, dispatch people, update external systems, or return partial progress. All quantities and rules you rely on must be present in the input. A model can be valid without representing every rule that applies to the real work.
