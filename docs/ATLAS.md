# Database migrations

Atlas uses SQL schema files and versioned SQL migrations. No HCL configuration is required.

Each table has one source file in `backend/schema/`. Define table-specific constraints and indexes with that table. Generate migrations into `backend/migrations/` and review the SQL before applying.

From `backend/`:

```sh
atlas migrate diff initial_schema \
  --dir "file://migrations" \
  --to "file://schema" \
  --dev-url "docker://postgres/17/dev"
```

For a configured PostgreSQL database:

```sh
atlas migrate apply --dir "file://migrations" --url "$DATABASE_URL"
```

Commit the generated migration files and `atlas.sum` together. Never edit an applied migration. PostgreSQL is the intended operational database; external database adapters are managed separately.

IDs are generated as UUIDv7 by the application. Database schema uses the PostgreSQL `UUID` type. Do not generate random v4 IDs.

The initial migration has not yet been generated or applied.
