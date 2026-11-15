# Model Context Protocol (MCP)

MCP is a planned integration interface for external clients and agents to use Orchestrata's optimization services. It is not the solver, the constraint compiler, or the durable job system.

## Proposed capabilities

- **Tools:** validate a problem definition; submit a solve job; inspect job status; cancel a job; retrieve a solution and validation report.
- **Resources:** read-only solver capabilities, constraint schema documentation, and permitted job artifacts.
- **Prompts:** optional examples for describing optimization problems; prompts cannot bypass validation.

## Design boundaries

Implement MCP as an adapter over the same typed application services used by the HTTP API. Avoid duplicating business logic or inventing MCP-only constraint semantics. Long-running solves must return a job identifier instead of holding a protocol request open indefinitely.

Authentication, tenant-scoped authorization, request quotas, input-size limits, and audit logging are mandatory before exposing remote MCP endpoints. External write-back requires separate explicit authorization. Do not expose credentials or unrestricted database query tools.

## Current status

No MCP server or tools are implemented. Define and test the application service boundary before selecting a Rust MCP SDK and transport. Track protocol compatibility against the official MCP specification.
