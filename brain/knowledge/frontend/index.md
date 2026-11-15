# Frontend

The frontend is a separate React and TypeScript application under `frontend/`, built with Vite.

TanStack Router handles routing and TanStack Query handles the solve request. The current page edits a JSON problem and displays the server response.

The Vite development proxy sends `/api` requests to `127.0.0.1:3000`.

## Planned UI

A structured problem editor, constraint diagnostics, run history, and result visualization are future work. The current JSON editor is a development tool, not a finished planning interface.

## Gotchas

- Frontend types are not yet generated from OpenAPI. Keep API changes coordinated until code generation is established.
- Do not treat client-side validation as authoritative.
