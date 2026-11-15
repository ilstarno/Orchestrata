# 0004 — MCP as an application protocol adapter

Status: accepted

## Decision

Support Model Context Protocol (MCP) through a backend adapter sharing typed services with the HTTP API. Keep protocol-specific types outside the solver core.

## Why

MCP clients need to discover and invoke optimization capabilities without duplicating the solver or binding it to a single client ecosystem.

## Consequences

MCP tools must enforce the same authorization, quotas, validation, job lifecycle, and audit semantics as other transports. No MCP implementation exists yet; SDK and transport selection are deferred until the service boundary is established.
