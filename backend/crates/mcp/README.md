# Local example tools

From the repository's `backend` directory, run:

```sh
cargo run --quiet -p orchestrata-mcp
```

Configure a local MCP client to launch that command over standard input and output. The process does not listen on a network port.

- `list_recipes` lists the built-in synthetic examples.
- `describe_recipe` takes an `id` from the list and describes that recipe's inputs and outputs.
- `get_recipe_example` takes an `id` and returns the corresponding synthetic solver input.
- `list_lead_examples` lists identifiers for the built-in lead preview.
- `preview_lead_example` takes a listed `lead_id` and returns its synthetic decision, the same as the HTTP preview at `POST /api/v1/examples/lead-to-contact/{lead_id}/preview`.

These tools do not run a solver, accept real records or uploads, access accounts, or modify data.
