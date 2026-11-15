# Atlas schema workflow

Keep each table in its own file under `backend/schema/`. Use Atlas to derive versioned SQL migrations; review generated SQL and commit the generated `atlas.sum` together with the migration files.

From `backend/`, with Atlas and Docker installed:

```sh
atlas migrate diff initial_schema --env local
atlas migrate lint --env local --latest 1
atlas migrate apply --env local
```

Set `DATABASE_URL` to the target PostgreSQL connection string before applying. Do not run migrations against production without review. The current schema is an initial proposal, not a deployed schema. The first migration and `atlas.sum` have not been generated yet.

MariaDB, SQL Server, and Snowflake remain planned external adapters; the PostgreSQL Atlas configuration does not manage those external databases.
