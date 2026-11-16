Orchestrata helps you test a plan against rules and preferences. This example has two jobs and two slots. The slots are numbered 0 and 1. Both jobs can use either slot, but they cannot use the same one. Job A prefers slot 1.

The example uses sample data and runs locally. This website does not operate the API or accept uploads. To run it over HTTP instead, use the [local API guide](../api/).

## Run the sample

From the Orchestrata repository root, with a stable Rust toolchain:

```sh
(cd backend && cargo run -p orchestrata-cli -- examples/assignment.json)
```

The result contains `values: [1, 0]` with hard and soft scores of zero. Values follow the input order, so job A takes slot 1 and job B takes slot 0. Both rules and preferences are satisfied.

## Change one thing

Copy the [assignment fixture](https://github.com/ilstarno/Orchestrata/blob/9dab0545beb8cb9ded93cc632e41f0b6bc718e77/backend/examples/assignment.json) to `backend/examples/preference-change.json`. Change the `value` in job A’s `prefer_value` constraint from `1` to `0`. Run the same command with `examples/preference-change.json`. The expected assignment is now `[0, 1]`; the hard rule does not change.

In a fresh copy, give both jobs the single option `[0]` and leave the different-slots rule in place. In a build with the verified exact-result contract, the CLI prints `null`: there is no valid assignment for this input.

## Decide what belongs in the model

- Make a rule **hard** if violating it makes a plan unusable.
- Make it a **preference** if a plan can still be used when that preference is missed.
- Remove an option entirely if it can never be chosen for a decision.

Start with an example small enough to check by hand. Compare the returned plan with your expected assignments, including a case that cannot be solved. If the search limit is exceeded, no answer was established. That is not the same as checking the model and finding no valid plan.
