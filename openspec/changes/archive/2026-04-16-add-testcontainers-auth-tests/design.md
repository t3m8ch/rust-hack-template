## Context

The current repository is a small Axum + SQLx + Postgres backend template with auth already implemented. The runtime entrypoint in `src/main.rs` currently performs all startup responsibilities in one place: loading config, building the SQLx pool, constructing `AppState`, assembling the router, wiring Socket.IO, binding the listener, and serving HTTP. That shape is fine for a binary, but it blocks straightforward reuse from tests because integration tests cannot construct the application without either duplicating startup logic or driving the binary from the outside.

Database schema setup also currently lives in `db/CreateTables.sql`, which is mounted into the development Postgres container through `compose.yml`. The repo already uses SQLx checked macros and `.sqlx/` metadata, but it does not yet have a `migrations/` directory that tests can apply automatically. As a result, there is no single schema source that a containerized test harness can reuse.

There are also no integration or e2e tests in the repository today. For auth, that means the most failure-prone flow in the template—register, login, cookie session lookup, logout, and post-logout access—is not continuously validated against a real database.

## Goals / Non-Goals

**Goals:**
- Make application assembly reusable from tests without changing the HTTP contract.
- Establish SQLx migrations as the schema source used by containerized tests.
- Add a reusable async Postgres test harness based on `testcontainers-modules::postgres`.
- Ensure tests can run through `cargo test` without manual `docker compose up`.
- Isolate parallel tests by creating a unique database per test while reusing one Postgres container per test process.
- Cover the existing auth flow with both router-level integration tests and at least one real-HTTP e2e test.
- Keep the change small and aligned with the current project layout.

**Non-Goals:**
- Redesign the auth API, cookie format, or session persistence model.
- Introduce mocks or replace real Postgres with an in-memory substitute.
- Add a broad service layer or major architectural rewrite beyond startup extraction.
- Add multi-container orchestration or Docker Compose-based test execution for the single-Postgres case.
- Duplicate every integration scenario in the e2e suite.

## Decisions

### 1. Expose reusable application assembly from library code
Create `src/lib.rs` (or an equivalent library surface) that owns module declarations and exports three reusable functions:
- `build_state(config, pgpool) -> AppState` for constructing the shared state object,
- `build_router(state) -> Router<AppState>` for assembling routes, middleware, and websocket wiring,
- `run(listener, app) -> impl Future<Output = anyhow::Result<()>>` for serving the application on a supplied listener.

`src/main.rs` will become a thin entrypoint that initializes tracing/env, loads config, connects the database pool, calls the library helpers, binds the configured address, and awaits `run(...)`.

**Rationale:**
- Router-level integration tests can build the app directly and use `tower::ServiceExt::oneshot` without a real TCP server.
- E2E tests can bind a `TcpListener` to `127.0.0.1:0`, call the same `run(...)`, and exercise the real HTTP stack with `reqwest`.
- Startup logic stays single-sourced instead of being copied into tests.

**Alternatives considered:**
- Keep everything in `main.rs` and spawn the binary in tests. Rejected because it is slower, more brittle, and makes router-level tests needlessly hard.
- Move only router construction out of `main.rs`. Rejected because tests also need a supported way to build state and serve the app on an ephemeral listener.

### 2. Make SQLx migrations the authoritative schema source
Add a `migrations/` directory with an initial migration that reproduces the current auth/session schema from `db/CreateTables.sql`. Tests will always create a fresh database and run `sqlx::migrate!()` before building the app. The development bootstrap SQL may remain as a convenience for `compose.yml`, but it must be kept aligned with the migration and treated as a derivative local-dev helper rather than the source of truth.

This change also requires enabling the SQLx migration feature in `Cargo.toml` so both runtime code (if needed) and tests can compile the migration macro.

**Rationale:**
- Testcontainers needs a schema bootstrap path that is independent of a manually prepared Docker volume.
- SQLx migrations are the standard reusable mechanism for schema setup in integration/e2e tests.
- Centralizing schema evolution in migrations reduces drift between CI, tests, and local development.

**Alternatives considered:**
- Continue using only `db/CreateTables.sql`. Rejected because tests would have to emulate Docker init script behavior and would not have an application-owned migration path.
- Remove the compose init SQL entirely. Rejected for now to avoid unnecessary local-dev churn; the repo can keep its existing dev convenience while shifting authority to migrations.

### 3. Use one async Postgres container per test process and one database per test
Add shared test helpers under `tests/common/` that lazily start a single `testcontainers-modules::postgres::Postgres` container through the async API and keep it alive for the lifetime of the test process. Each test will request a new unique database name, connect via an admin URL built from the container's dynamically assigned host/port, create that database, run migrations, and then return a test-specific `DATABASE_URL` (and optionally a ready pool/app helper).

The helper must not use `sleep` or hardcoded `localhost:5432`; it must rely on the container runtime APIs for readiness and dynamic port lookup.

**Rationale:**
- Reusing one container keeps the suite reasonably fast.
- A unique database per test avoids cross-test contamination and supports parallel execution.
- The helper API can be reused by both integration and e2e tests without duplicated setup logic.

**Alternatives considered:**
- New container per test. Rejected as unnecessarily slow for this template.
- Shared database with table cleanup between tests. Rejected because it is fragile under parallel execution and increases coupling between tests.
- Docker Compose-based test stack. Rejected because the current need is a single Postgres dependency.

### 4. Split test coverage into router-level integration tests and narrow e2e smoke coverage
Add integration tests that exercise the Axum `Router` directly with request objects and a real Postgres-backed state. These tests will cover the auth scenarios individually: successful registration, duplicate registration conflict, successful login, invalid password, unauthorized `me` access without a cookie, authorized `me` access after login, logout, and unauthorized `me` after logout.

Add a separate e2e test that starts a real server on `127.0.0.1:0` and uses `reqwest::Client` with cookie storage enabled to validate one full happy path: register → login → me → logout → me returns 401.

**Rationale:**
- Integration tests give high scenario coverage quickly without paying TCP/server startup cost for every case.
- A small e2e layer confirms that the actual listener/server/request stack behaves correctly, including cookies.
- This keeps the suite fast while still validating the most important path end-to-end.

**Alternatives considered:**
- Only e2e tests. Rejected because they are slower and would push too much scenario coverage through the network boundary.
- Only router-level tests. Rejected because one real server smoke test is valuable for catching listener/cookie/client integration issues.

### 5. Keep dependency additions minimal and CI-oriented
Update `Cargo.toml` with only the features needed for this plan: SQLx migration support, async testcontainers Postgres module, and any missing test-side utilities such as `tower` service helpers or `reqwest` cookie-store support if not already available. The resulting suite must run with plain `cargo test`; documentation may additionally recommend `cargo nextest run` for faster CI execution.

**Rationale:**
- The repo is a small template and should stay lightweight.
- CI usability is a first-class requirement for this change.

**Alternatives considered:**
- Introduce broader test frameworks or mocking libraries. Rejected because they are not needed for the real-Postgres goal.

## Risks / Trade-offs

- **Schema duplication between `migrations/` and `db/CreateTables.sql`** → Mitigation: treat the migration as authoritative, keep the init SQL aligned in the same change, and document the relationship in repository guidance.
- **Async global container lifecycle can be tricky** → Mitigation: centralize container startup in one helper using a single lazy initialization path and leak/hold the container handle for the whole process so it is never dropped mid-test.
- **Parallel test database creation may leave many temporary DBs** → Mitigation: use unique names and rely on container teardown at process end; optional drop-on-teardown can be added later if needed.
- **Startup extraction could accidentally change runtime behavior** → Mitigation: keep the refactor mechanical, preserve existing route composition and middleware ordering, and use the new tests to verify behavior.
- **Docker availability still matters in CI/local runs** → Mitigation: tests should self-provision everything else and fail only when Docker itself is unavailable, which is expected for testcontainers-based suites.

## Migration Plan

1. Introduce the library bootstrap surface and make `main.rs` delegate to it without changing endpoint behavior.
2. Add the initial SQLx migration that matches the current auth/session schema and update local bootstrap SQL to stay in sync.
3. Add shared test helpers for container startup, per-test database creation, migration execution, router construction, and optional HTTP server spawning.
4. Add router-level auth integration tests.
5. Add the real-HTTP auth e2e smoke test.
6. Run formatting, compilation, targeted tests, full `cargo test`, and SQLx preparation checks as appropriate.

Rollback is straightforward: revert the library extraction, remove the migrations and tests, and restore the previous startup path. No production data migration is introduced beyond formalizing the existing schema in SQLx migration files.

## Open Questions

- Should the repository also add a CI workflow update in the same implementation change, or keep this change limited to code/tests plus a documented `cargo nextest run` recommendation?
- If future features add more tables, should `compose.yml` continue to bootstrap from copied SQL or should local development move fully to running migrations at startup or via a dedicated command?