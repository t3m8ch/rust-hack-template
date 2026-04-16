## Context

The repository already treats `migrations/` as the authoritative database schema, but local development still has a second bootstrap mechanism through `db/CreateTables.sql` mounted into the Postgres container. That duplication creates drift risk and makes the local startup story more complicated than the test story, where fresh databases are already migrated directly with SQLx.

This change is cross-cutting because it touches runtime startup behavior, local Docker bootstrap, and developer documentation. The project already includes the `sqlx` `migrate` feature, so the missing piece is wiring migration execution into normal application startup.

## Goals / Non-Goals

**Goals:**
- Make `migrations/` the only schema source for local runtime startup.
- Apply pending SQLx migrations automatically before the app starts serving HTTP traffic.
- Remove the need to maintain `db/CreateTables.sql` and the compose bind mount that depends on it.
- Keep startup failure explicit if the database is reachable but the migration set cannot be applied.
- Reuse the migration behavior in a small, clear bootstrap path that matches the repository’s layer-first structure.

**Non-Goals:**
- Provisioning or creating the Postgres server/container automatically.
- Changing the SQL schema itself.
- Replacing the existing test helper migration flow unless a small shared helper naturally improves reuse.
- Adding rollback automation for failed migrations beyond SQLx’s existing migration semantics.

## Decisions

### Run migrations after connecting and before binding the listener
The application will connect to Postgres first, then run `sqlx::migrate!("./migrations")`, and only after that continue with state construction and HTTP listener startup.

**Rationale:** this guarantees no requests are accepted against an outdated schema and keeps failure behavior simple: startup aborts early.

**Alternatives considered:**
- Run migrations after the server starts: rejected because requests could race with schema updates.
- Keep Docker init SQL for local dev and only migrate in some environments: rejected because it preserves duplicate schema sources.

### Expose migration bootstrap through reusable app code
The runtime migration step should live in reusable application/bootstrap code instead of being hardcoded only inside `src/main.rs`, for example as a small helper alongside `connect_pgpool` in `src/lib.rs` or a nearby bootstrap module.

**Rationale:** tests and future binaries can reuse one supported way to connect and prepare the database without duplicating migration logic.

**Alternatives considered:**
- Put all migration logic directly in `main`: simpler initially, but less reusable and easier to diverge from tests or tooling.
- Hide migration execution inside every caller: rejected because migration policy should be centralized.

### Remove the compose-time schema mirror entirely
`compose.yml` will stop mounting `db/CreateTables.sql`, and the tracked file will be deleted.

**Rationale:** the migration directory already provides the schema, and keeping the mirror invites drift.

**Alternatives considered:**
- Keep the file as optional reference documentation: rejected because the repository already has actual migrations.
- Generate the file from migrations: rejected because the team goal is to eliminate the extra artifact, not automate its upkeep.

### Keep startup errors surfacing as application startup failure
If migration execution fails, startup returns an error and the server does not begin listening.

**Rationale:** partial startup would hide schema problems and produce harder-to-debug runtime errors.

**Alternatives considered:**
- Log and continue: rejected because handlers may then run against an invalid schema.
- Retry indefinitely: rejected because this is operational policy rather than application bootstrap behavior.

## Risks / Trade-offs

- **Migration duration increases startup time** → Acceptable for this project; local migrations are small and the consistency benefit outweighs a short startup delay.
- **Existing local volumes may expose migration issues that the init SQL previously masked** → This is desirable visibility; docs should state that the app now applies the same SQLx migrations used elsewhere.
- **A shared migration helper can blur responsibility if overdesigned** → Keep the abstraction minimal: connect, migrate, return pool.
- **Automatic startup migrations are unsuitable for some production workflows** → The change only defines the repository default; deployment-specific wrappers can still choose when to invoke the binary.

## Migration Plan

1. Add a reusable bootstrap helper that connects to Postgres and applies SQLx migrations.
2. Update the binary entrypoint to use that helper before building app state and binding the listener.
3. Remove the `db/CreateTables.sql` file and delete the corresponding bind mount from `compose.yml`.
4. Update README and any setup notes to describe the new workflow: start Postgres, run the app, let startup migrations prepare the schema.
5. Verify with formatting, compilation, and a fresh local database startup smoke test.

Rollback strategy: restore the previous compose bind mount and `db/CreateTables.sql`, then revert the startup migration call if the new bootstrap flow causes blocking operational issues.

## Open Questions

- None at proposal time.
