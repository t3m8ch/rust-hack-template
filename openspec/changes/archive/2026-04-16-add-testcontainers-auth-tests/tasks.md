## 1. Extract reusable app bootstrap and schema setup

- [x] 1.1 Move crate module declarations and reusable startup logic out of `src/main.rs` into `src/lib.rs` (or equivalent) and expose `build_state(...)`, `build_router(...)`, and `run(...)`.
- [x] 1.2 Keep `src/main.rs` as a thin entrypoint that initializes tracing/config, opens the Postgres pool, binds the configured address, and delegates to the shared library helpers.
- [x] 1.3 Add SQLx migrations for the existing auth/session schema and enable the required SQLx migration feature(s) in `Cargo.toml`.
- [x] 1.4 Align `db/CreateTables.sql` and any developer guidance with the new migration-owned schema source of truth.

## 2. Add reusable containerized Postgres test infrastructure

- [x] 2.1 Add the minimal dev-dependencies/features needed for async Postgres testcontainers, router/service testing, and cookie-aware HTTP clients.
- [x] 2.2 Create `tests/common/` helpers that lazily start one `testcontainers-modules::postgres` container per test process using the async API and dynamic host/port discovery.
- [x] 2.3 Implement per-test database creation with unique names, run SQLx migrations for each database, and expose convenient helpers for test `DATABASE_URL`, pool creation, and app/router construction.
- [x] 2.4 Add an HTTP test helper that binds the app to `127.0.0.1:0` and returns the spawned server handle/base URL for e2e tests.

## 3. Cover the auth flow with integration and e2e tests

- [x] 3.1 Add router-level integration tests for auth register success and duplicate registration conflict against a real Postgres database.
- [x] 3.2 Add router-level integration tests for login success, login with the wrong password, and `me` without a cookie.
- [x] 3.3 Add router-level integration tests for `me` with a valid cookie, logout, and `me` returning unauthorized after logout.
- [x] 3.4 Add an end-to-end auth happy-path test using a real HTTP server plus `reqwest::Client` cookie storage to verify register → login → me → logout → me=401.

## 4. Verify and document the result

- [x] 4.1 Run `cargo fmt`, `cargo check`, and targeted auth tests, then confirm the full suite runs with plain `cargo test` without manual `docker compose up`.
- [x] 4.2 Refresh SQLx metadata if query/schema changes require it and update repository notes to mention the new test workflow plus optional `cargo nextest run` usage.