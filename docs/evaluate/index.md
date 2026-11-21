## Choose how to start

To inspect a result without installing tools, open the [worked assignment](/Orchestrata/examples/assignment/). To call a running server, follow the [first HTTP request](/Orchestrata/docs/solve-api/); the client needs no Rust installation.

Rust is the programming language used to build the engine and server. Cargo builds Rust programs and manages their dependencies. Only the source-build path needs these tools. The [server setup guide](/Orchestrata/docs/backend-setup/) explains installation and version checks.

All source commands below run from the Orchestrata checkout root. The shell examples use Linux, macOS or WSL.

Two jobs share slots 0 and 1. Either job can take either slot, but they must use different ones. Job A prefers slot 1.

Run this sample locally. The site does not run the API or accept uploads. For HTTP, use the [local API guide](../api/).

## Run the sample

From the Orchestrata repository root, with a stable Rust toolchain:

```sh
cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-cli -- backend/examples/assignment.json
```

The result contains `values: [1, 0]` with hard and soft scores of zero. Values follow the input order, so job A takes slot 1 and job B takes slot 0. Both rules and preferences are satisfied.

## Change one thing

Copy the `backend/examples/assignment.json` to `backend/examples/preference-change.json`. Change the `value` in job A’s `prefer_value` constraint from `1` to `0`. Run the same command with `backend/examples/preference-change.json`. The expected assignment is now `[0, 1]`; the hard rule does not change.

In a fresh copy, give both jobs the single option `[0]` and leave the different-slots rule in place. In a build with the verified exact-result contract, the CLI prints `null`: there is no valid assignment for this input.

## Decide what belongs in the model

- Make a rule **hard** if violating it makes a plan unusable.
- Make it a **preference** if a plan can still be used when that preference is missed.
- Remove an option entirely if it can never be chosen for a decision.

Start with an example small enough to check by hand. Compare the returned plan with your expected assignments, including a case that cannot be solved. If the search limit is exceeded, no answer was established. That is not the same as checking the model and finding no valid plan.
