# Run a sample lead-to-contact decision

The local API evaluates 60 built-in examples. The examples use generated identities and non-deliverable sample addresses. It does not accept a real lead, connect to a CRM, create contacts, or write to a database.

From the repository root, start the local API:

```sh
(cd backend && cargo run --locked -p orchestrata-api)
```

Send a POST with no body. Use a built-in ID from `L-001` through `L-060`:

```sh
curl -fsS -X POST http://127.0.0.1:3000/api/v1/examples/lead-to-contact/L-001/preview
```

```json
{"lead_id":"L-001","outcome":"created","reason":"new","assigned_representative_id":"REP-08","contact_id":"CT-001"}
```

The contact ID is part of the simulated outcome, not a receipt. For `held` and `review`, the contact ID is null. When a representative cannot be selected, its ID is null. Running the same example twice does not create or change records.

See `/api/v1/openapi.json` for the response schema and `/api/v1/error-codes` for the error index. An unknown sample ID returns `404` with code `unknown_lead_example`. A nonempty request body is rejected with code `invalid_lead_preview_request`.
