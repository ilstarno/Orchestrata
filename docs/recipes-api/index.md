# Recipe API reference

Use the built-in catalog on [public revision `b8d1271e3faddfffaf220698e381197bcae1752f`](https://github.com/ilstarno/Orchestrata/tree/b8d1271e3faddfffaf220698e381197bcae1752f). The catalog holds worked examples and runs on your own machine. It does not accept customer uploads. The project website does not host these endpoints.

## Start with the catalog

From the repository root at the pinned revision, run `(cd backend && cargo run -p orchestrata-api)`. Run the following in a second terminal:

```sh
API=http://127.0.0.1:3000
curl -fsS "$API/api/v1/health"
curl -fsS "$API/api/v1/openapi.json"
curl -fsS "$API/api/v1/recipes"
curl -fsS "$API/api/v1/recipes/field-service"
curl -fsS "$API/api/v1/recipes/field-service/example"
curl -fsS -X POST "$API/api/v1/recipes/field-service/run-example"
```

| Method and path | Success response |
| --- | --- |
| `GET /api/v1/recipes` | A JSON array of metadata objects. |
| `GET /api/v1/recipes/{recipe_id}` | One metadata object. Use the exact `id` returned by the catalog. |
| `GET /api/v1/recipes/{recipe_id}/example` | The bundled planning problem as JSON. It does not solve it. |
| `POST /api/v1/recipes/{recipe_id}/run-example` | Solves the bundled example and returns a JSON result. Send no request body. Where variants exist, this route uses the default example. |
| `GET /api/v1/recipes/grid` | One filtered, sorted page of catalog metadata. |
| `GET /api/v1/error-codes` | An array of error codes, descriptions, and associated status lists (keys: `code`, `description`, `status_codes`). |
| `GET /api/v1/openapi.json` | The OpenAPI document emitted by the running build. Check it before using a route. |

Catalog metadata has `id` (the path identifier), `title`, `scenario` (descriptive text), and the `inputs` and `outputs` arrays. Each field contains `name`, `kind`, `required` (boolean), and `description`. Supported field kinds are `string`, `integer`, `boolean`, `string-list`, and `integer-list`. Treat these fields as documentation. `run-example` uses the built-in input.

The example response contains `variables` and `constraints`. A variable has a `name` and a `domain` of allowed signed 64-bit integers. Constraints are tagged by `kind` and use zero-based indices into `variables`. Consult the running OpenAPI schema for specific constraint fields. When reading JSON in a client with floating-point numbers by default, choose a way to preserve 64-bit integers exactly if values can exceed the safe integer range.

A successful run wraps the result in `solution`. A non-null result contains `values` in input variable order, `score` with separate `hard` and `soft` fields, and `evaluated`, the number of candidates evaluated. `{"solution":null}` means the exact search proved that no assignment satisfied all hard rules. Request errors, exhausted limits, and solver failures do not imply infeasibility. The bundled runner is not a custom-data upload, does not create a job, and does not call external systems.

## Browse the grid

Send query parameters as URL-encoded values. The endpoint filters metadata before sorting and paging; it does not solve plans. Parameters are optional.

| Parameter | Default | Accepted values |
| --- | --- | --- |
| `page` | `1` | Integer from `1` to `100000`, inclusive. Pages are one-based. |
| `page_size` | `25` | Integer from `1` to `100`, inclusive. It is the maximum rows returned. |
| `sort` | `id` | `id` or `title`. Sorting compares stored strings directly, independent of locale. |
| `direction` | `asc` | `asc` or `desc`. Rows with equal titles always tie-break on ascending `id`. |
| `q` | Empty string | A literal, case-insensitive substring of `id`, `title`, or `scenario`. Surrounding whitespace is trimmed for matching. The decoded input must be at most 100 UTF-8 bytes before trimming and must not contain control characters. Empty or whitespace-only input matches every row. Wildcards and regular expressions are not supported. |

```sh
curl -fsS --get "$API/api/v1/recipes/grid" \
  --data-urlencode 'q= FIELD-SERVICE ' \
  --data-urlencode 'sort=title' \
  --data-urlencode 'direction=desc' \
  --data-urlencode 'page_size=1' \
  --data-urlencode 'page=1'
```

A grid response has five properties:

- `columns`: descriptors with `id`, `label`, and `sortable`. In order: `id` / ID / true, `title` / Title / true, `scenario` / Scenario / false.
- `items`: rows for this page, each with only `id`, `title`, and `scenario`. Fetch the detail route for input and output fields.
- `page` and `page_size`: the effective page requested and maximum rows for that page.
- `total_items`: number of matches across all pages.

An in-range page past the available rows returns `200`, the requested page, and an empty `items` array; it does not erase `total_items`. Unknown, duplicate, malformed, and out-of-range parameters return `400`. For instance, `?page=0`, `?sort=scenario`, and `?page=1&page=2` are invalid.

## Diagnose a response

Use `curl -i` to inspect status and headers on a failure. Omit `curl -f` to keep the error body. This revision attaches `X-Request-ID` for correlation and use `X-Orchestrata-Error-Code` for machine-readable classification on errors. Header names are case-insensitive. Error text can be plain text; do not assume a JSON error envelope or parse the prose as a code.

| Response | Meaning and next step |
| --- | --- |
| `400 request_rejected` from the grid | Correct the query. The body is `Invalid recipe grid query`; the server does not echo the submitted query. |
| `404 not_found` | Check the exact recipe ID and running revision. An unknown recipe and an unavailable route can both return this code. |
| `405 method_not_allowed` | Check the verb; running an example uses `POST`, while fetching it uses `GET`. |
| `500 internal_error` from catalog, detail, or grid | The build could not load bundled metadata. Keep the request ID and inspect the local API output. |
| `500 solve_execution_failed` from the bundled runner | The bundled solve could not finish. Capture the request ID and revision. This is not a planning outcome. |
| `503 solver_busy` from the bundled runner | All solve slots were occupied. Wait and retry with backoff. |

```sh
curl -si "$API/api/v1/recipes/grid?page=0"
curl -si "$API/api/v1/recipes/no-such-recipe"
curl -fsS "$API/api/v1/error-codes"
```

If the grid returns catalog metadata, a 404, or HTML, check the pinned revision and confirm that the request targets the loopback API port.
