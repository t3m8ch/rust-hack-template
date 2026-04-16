## Why

The repository currently has no reusable test harness for running integration or end-to-end tests against a real Postgres instance, and the application assembly is embedded in `src/main.rs`, which makes router-level and server-level testing awkward. Auth is already implemented, but the critical register/login/session flow is not protected by containerized tests that can run through plain `cargo test` without manual `docker compose up`.

## What Changes

- Add a reusable containerized Postgres test harness based on `testcontainers-modules::postgres` with one shared container per test process and a unique migrated database per test.
- Refactor application startup so app state creation, router construction, and server execution are available from library code via `build_state(...)`, `build_router(...)`, and `run(...)`, while `src/main.rs` remains a thin entrypoint.
- Move database schema ownership into SQLx migrations so tests and runtime initialization use the same source of truth instead of relying only on Docker init SQL.
- Add Postgres-backed integration tests for the auth HTTP flow using the Axum router/service directly where TCP is unnecessary.
- Add at least one real HTTP end-to-end auth happy-path test using `reqwest::Client` with cookie persistence and a server bound to `127.0.0.1:0`.
- Update dependencies and test organization so the suite is CI-friendly and runs via ordinary `cargo test` without manual environment preparation beyond Docker availability.

## Capabilities

### New Capabilities
- `containerized-postgres-tests`: Provide reusable real-Postgres test infrastructure with async testcontainers startup, SQLx migrations, and isolated databases per test.
- `auth-flow-tests`: Define integration and end-to-end auth-flow coverage that validates registration, login, session cookies, logout, and unauthorized access paths against the real database.

### Modified Capabilities
- None.

## Impact

- Affected code: `src/main.rs`, new library bootstrap surface (`src/lib.rs` or equivalent), `src/state.rs`, auth routes/helpers, new `tests/common/` helpers, integration/e2e test files, and SQLx metadata/migrations.
- Affected data setup: introduce `migrations/` as the authoritative schema source and keep Docker bootstrap SQL aligned or derived from it.
- Dependencies: add only the necessary dev/runtime features for SQLx migrations, `testcontainers-modules`, and test HTTP execution.
- Runtime/API impact: no intended HTTP contract change; auth endpoints should preserve existing request/response and cookie behavior while becoming easier to test.