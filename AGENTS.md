# Engineering conventions

Read the relevant `brain/` knowledge before changing a subsystem. Keep implementation details accurate and separate from future plans.

Use the latest stable Rust toolchain and compatible stable React dependencies. Format and lint code before review. Keep source files under 500 lines; split files by responsibility, not arbitrary size. Place substantial types, enums, and constants in dedicated modules. Use early returns and small functions instead of deep nesting. Avoid repetitive comments and avoid publishing personal background or private experience.

Rust: typed boundaries, explicit errors, separate solver, application, persistence, and transport layers. Use utoipa for OpenAPI. Keep MCP tools behind the same application services as HTTP.

Database: use Atlas versioned migrations. Maintain one SQL schema file per table under `backend/schema/`; generate and review migration files under `backend/migrations/`. Do not modify applied migrations.

React: use React Aria Components, TanStack libraries, and an established icon package. Provide keyboard-accessible dark and light themes and responsive layouts. Consult UI/UX Pro Max guidance for interface design.

Do not mention assistants or code-generation tools in commits or pull requests.
