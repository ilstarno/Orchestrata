# MCP service contract

MCP is a separate backend transport over the same typed application services as HTTP. It must not invoke the solver by bypassing validation, authorization, or job lifecycle checks.

## Planned tools

| Tool | Input | Output |
| --- | --- | --- |
| `orchestrata_validate_problem` | Problem definition | Validation report and diagnostics |
| `orchestrata_submit_job` | Problem reference, execution options | Job ID and initial status |
| `orchestrata_get_job` | Job ID | Status, progress, and metadata |
| `orchestrata_get_solution` | Job ID | Verified solution or terminal failure |
| `orchestrata_cancel_job` | Job ID | Cancellation outcome |

## Protocol

Implement MCP with the official JSON-RPC method and schema conventions, using a maintained Rust SDK. Support stdio for local development and authenticated Streamable HTTP for hosted clients. Tool calls that start long-running optimization return a job ID, not a long-held request.

Every operation must enforce tenant scoping, request limits, authorization, and audit logging. Remote MCP cannot expose unrestricted SQL execution or direct connector credentials. External write-back requires a separate authorized action.

## Status

This file defines the intended interface. No MCP server is implemented yet. The synchronous `POST /api/v1/solve` endpoint remains a small reference-solver development endpoint.
