## 1. Startup migration bootstrap

- [x] 1.1 Add a reusable database bootstrap helper that connects to Postgres and applies SQLx migrations from `./migrations`
- [x] 1.2 Update the application startup path to run the migration helper before building app state and binding the HTTP listener
- [x] 1.3 Ensure migration failures abort startup cleanly instead of allowing the server to listen

## 2. Remove duplicate local schema bootstrap

- [x] 2.1 Remove the `db/CreateTables.sql` artifact from the repository
- [x] 2.2 Update `compose.yml` to stop mounting any init SQL file for schema creation

## 3. Align docs and verification

- [x] 3.1 Update README and any setup guidance to describe the new local workflow and note that the app auto-applies migrations on startup
- [x] 3.2 Run formatting and compile checks, then perform a fresh local database startup smoke test to confirm the schema is created without `db/CreateTables.sql`
