# containerized-postgres-tests Specification

## Purpose
TBD - created by archiving change add-testcontainers-auth-tests. Update Purpose after archive.
## Requirements
### Requirement: Application assembly SHALL be reusable from library code
The repository SHALL expose supported library-level application assembly helpers so tests can construct runtime-equivalent state, router, and server execution without invoking the binary entrypoint.

#### Scenario: Building the app for tests
- **WHEN** integration or e2e tests need to run the existing Axum application
- **THEN** they can call shared library helpers to build `AppState`, assemble the `Router`, and run the server on a supplied listener while `src/main.rs` remains a thin entrypoint

### Requirement: Test infrastructure SHALL provision isolated Postgres databases through async testcontainers
The repository SHALL provide reusable test helpers that start Postgres through `testcontainers-modules::postgres` async APIs, discover the dynamically assigned host and port, create a unique database per test, and return a usable `DATABASE_URL` for that isolated database.

#### Scenario: Parallel tests request databases
- **WHEN** multiple tests run in parallel in the same test process
- **THEN** the helpers reuse one shared Postgres container but create a different uniquely named database for each test instead of sharing a single database instance

### Requirement: Test databases SHALL be migrated before use without manual compose startup
The repository SHALL apply SQLx migrations to every freshly created test database before the application uses it, and running the test suite SHALL NOT require a manual `docker compose up`, hardcoded `localhost:5432`, or ad-hoc sleep-based readiness waits.

#### Scenario: Running containerized tests from a clean environment
- **WHEN** a developer or CI job runs `cargo test` with Docker available but without a pre-started Postgres service
- **THEN** the test harness starts Postgres automatically, waits through container readiness APIs, migrates the new test database, and allows the tests to execute against the real schema

