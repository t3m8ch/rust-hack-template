## Why

The project still relies on `db/CreateTables.sql` as a separate bootstrap path for local Postgres startup, which duplicates the authoritative SQLx migrations and creates extra setup friction for anyone starting the project. We should make the application self-bootstrap its schema from `migrations/` so local development uses one source of truth and does not require manual schema preparation.

## What Changes

- Remove the local Docker init dependency on `db/CreateTables.sql`.
- Apply SQLx migrations automatically during application startup before serving requests.
- Fail startup with a clear error if the database is reachable but migrations cannot be applied.
- Update local developer setup documentation to describe the simpler workflow: start Postgres, then run the app.
- Keep test and local flows aligned around `migrations/` as the only schema source of truth.

## Capabilities

### New Capabilities
- `startup-database-migrations`: The application automatically applies pending SQLx migrations on startup before binding the HTTP listener.

### Modified Capabilities
- None.

## Impact

- Affected code: `src/main.rs`, reusable database bootstrap code in `src/lib.rs` or a nearby module, local startup docs, and Docker compose configuration.
- Removed artifact: `db/CreateTables.sql`.
- Runtime behavior: app startup now depends on successful migration execution against the configured Postgres database.
- Developer workflow: no separate schema bootstrap file to maintain for local setup.
