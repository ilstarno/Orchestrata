# Local recipe tools

From the repository's `backend` directory, run:

```sh
cargo run --quiet -p orchestrata-mcp
```

Configure a local MCP client to launch that command over standard input and output. The process does not listen on a network port.

- `list_recipes` lists the built-in synthetic examples.
- `describe_recipe` takes an `id` from the list and describes that recipe's inputs and outputs.
- `get_recipe_example` takes an `id` and returns the corresponding synthetic solver input.

The tools do not run a solve, accept file uploads, access accounts, or modify data.
