# Frontend

The frontend is a separately buildable React and TypeScript application under `frontend/`, using Vite and TanStack Router and Query. Other TanStack packages may be added where they solve concrete UI requirements.

Use current stable React and compatible stable TanStack versions. Pin resolved dependencies through a committed lockfile; update intentionally and verify builds rather than using floating runtime versions.

The current interface edits a JSON problem and displays the solver response. The Vite proxy forwards `/api` to `127.0.0.1:3000`.

## Planned

A typed problem editor, constraint diagnostics, job history, and result visualization. The frontend is not an MCP server; MCP belongs at the backend protocol boundary.

## Gotchas

- Generate frontend API types from an OpenAPI contract before expanding endpoints.
- Browser validation does not replace backend validation.
