# Repository guidance

Before changing code, read `brain/index.md` and the relevant subsystem page under `brain/knowledge/`. For architectural changes, read `brain/decisions/`.

Keep the brain aligned with the implementation. Update an existing page when behavior or a boundary changes. Add a decision record for hard-to-reverse choices, including the reason and trade-offs. Do not document proposed functionality as implemented.

Code must be type-safe, readable, and testable. Preserve the separation between the Rust solver, HTTP transport, persistence, and the React frontend. Do not bypass independent solution validation. Never commit credentials or customer data.

Commit messages and pull-request descriptions must describe the technical change without references to assistants, generation tools, or authorship automation.

Before submitting a change, run relevant formatters, linters, tests, and builds. If a check cannot run, state that explicitly.
