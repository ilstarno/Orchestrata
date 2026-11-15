# Engineering conventions

Read the relevant `brain/` knowledge before changing a subsystem. Keep implementation details accurate and separate from future plans.

Use the latest stable Rust toolchain and compatible stable React dependencies. Format and lint code before review. Keep source files under 500 lines; split files by responsibility, not arbitrary size. Keep each type and enum in its owning file; place shared constants in dedicated modules and local constants at the top of their files. Use early returns and small functions instead of deep nesting. Keep comments purposeful.

Rust: typed boundaries, explicit errors, separate solver, application, persistence, and transport layers. Use utoipa for OpenAPI. Keep MCP tools behind the same application services as HTTP.

Database: use Atlas with SQL schema files and SQL migrations, without HCL configuration. Keep one table definition per file under `backend/schema/`. Use UUIDv7 for every application-generated identifier. Generate and review migrations under `backend/migrations/`; never modify applied migrations.

React: use React Aria Components, TanStack libraries, and an established icon package. Provide keyboard-accessible dark and light themes and responsive layouts. Consult UI/UX Pro Max guidance for interface design.
