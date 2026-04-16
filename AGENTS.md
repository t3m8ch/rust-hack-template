# AGENTS.md

This file is for coding agents working in this repository.

## Project Summary

- Stack: Rust, Axum, Postgres, SQLx, Socketioxide.
- App entry point: `src/main.rs`.
- Reusable app assembly lives in `src/lib.rs` via `build_state(...)`, `build_router(...)`, and `run(...)`.
- Runtime config comes from environment variables via `envy` and `.env`.
- SQL queries use SQLx compile-time checked macros plus checked metadata in `.sqlx/`.
- SQL schema source of truth lives in `migrations/`.

## Repository Layout

- `src/lib.rs`: reusable app bootstrap, module declarations, top-level router assembly, and server runner.
- `src/main.rs`: thin app entrypoint that loads config, connects Postgres, applies migrations, binds the listener, and delegates to `src/lib.rs`.
- `src/rest/`: HTTP routers and handlers only.
- `src/dto/`: request/response DTOs and transport-facing mapping.
- `src/db/`: DB row structs and SQL helpers.
- `src/auth/`: auth-specific domain helpers such as password hashing and session workflows.
- `src/extractors/`: reusable Axum extractors such as `ValidatedJson<T>`.
- `src/error.rs`: unified API error type and HTTP error serialization.
- `migrations/`: authoritative SQLx schema migrations used by tests, local startup, and schema evolution.
- `compose.yml`: local Postgres service.
- `justfile`: common dev helpers.

## Editor-Specific Rules

- No `.cursor/rules/`, `.cursorrules`, or `.github/copilot-instructions.md` are present.
- There are currently no repository-local Cursor or Copilot rules beyond this file.

## Environment Setup

- Start Postgres: `just dev`
- Default DB URL from `.env`: `postgres://postgres:postgres@localhost:1311/postgres`
- The app loads `.env` automatically via `dotenvy`.
- The app applies pending SQLx migrations automatically on startup before serving requests.

## Build Commands

- Build debug binary: `cargo build`
- Build release binary: `cargo build --release`
- Check compilation only: `cargo check`
- Run lints: `cargo clippy --all-targets --all-features -- -D warnings`
- Run the app locally: `cargo run`
- Format the codebase: `cargo fmt`
- Check formatting without changing files: `cargo fmt -- --check`

## Test Commands

- Run all tests: `cargo test`
- Run a specific test by name: `cargo test <test_name>`
- Run a specific integration/unit test and show stdout: `cargo test <test_name> -- --nocapture`
- Run tests in one module/file by substring match: `cargo test auth`
- Run one exact-named test only: `cargo test <test_name> -- --exact --nocapture`
- Run the containerized auth integration suite: `cargo test --test auth_integration -- --nocapture`
- Run the containerized auth e2e smoke test: `cargo test --test auth_e2e -- --nocapture`
- Optional faster local/CI runner: `cargo nextest run`

## Linting And Verification

- Preferred minimal verification after code changes: `cargo fmt && cargo check && cargo clippy --all-targets --all-features -- -D warnings`
- If tests exist for touched code, run targeted tests before broad test suites.
- If SQL query text or selected columns change, refresh SQLx metadata.
- Refresh SQLx metadata: `just sqlx-prepare`

## SQLx Workflow

- This repo uses `sqlx::query!` and `sqlx::query_as!`.
- Those macros depend on either a live `DATABASE_URL` or `.sqlx` metadata.
- Schema changes must be made in `migrations/` first.
- When changing SQL queries, selected columns, or schema, update `.sqlx/`.
- Preferred flow: ensure Postgres is running, ensure live schema matches the current migrations, run `cargo check`, run `just sqlx-prepare`, then re-run `cargo check`.

## Database Notes

- `migrations/` is the authoritative schema source.
- Local app startup applies pending SQLx migrations automatically against the configured database.
- For schema changes, update the migration and apply the change to the running local DB or recreate the volume if you need a fresh local environment.
- Containerized tests create a fresh database per test and run SQLx migrations automatically; they do not require manual `docker compose up`.

## HTTP/API Conventions

- JSON handlers should return `AppResult<T>` when errors are possible.
- Use the unified `AppError` in `src/error.rs` for API failures.
- Error responses must stay in the shape `{ "error": { "code": ..., "message": ..., "fields": ...? } }`.
- Validation errors should map into `AppError::Validation`.
- Auth/JSON validation should go through reusable extractors such as `ValidatedJson<T>`.
- For browser session auth, prefer cookie-based flows over JWT unless requirements change.

## Code Organization Rules

- Keep `src/rest/*.rs` focused on routing and HTTP flow.
- Put request and response DTOs in `src/dto/`, including feature-specific files such as `src/dto/auth.rs`.
- Put SQL row structs and query helpers in `src/db/`, including feature-specific files such as `src/db/auth.rs`.
- Put reusable domain helpers in feature modules such as `src/auth/`.
- Put Axum extractors in `src/extractors/`, including feature-specific files such as `src/extractors/auth.rs`.
- Prefer a layer-first layout for transport and persistence adapter code; do not place HTTP extractor definitions inside feature service modules by default.
- Avoid mixing DTOs, DB structs, SQL, and handlers in one file unless the code is truly tiny.

## Formatting And Imports

- Always run `cargo fmt` after edits.
- Follow rustfmt output instead of hand-formatting preferences.
- Group imports in the current repo style: std first, external crates second, `crate::...` last.
- Prefer nested imports when they improve readability, for example `auth::{CurrentUser, session::...}`.
- Remove unused imports promptly.

## Naming Conventions

- Types and traits: `PascalCase`.
- Functions and modules: `snake_case`.
- Constants: `SCREAMING_SNAKE_CASE`.
- Error variants: concise semantic names such as `InvalidCredentials`, not transport strings.
- DTO names should be explicit: `LoginRequest`, `UserResponse`.
- DB row structs should communicate their DB role: `UserRow`, `UserAuthRow`.

## Types And Data Modeling

- Prefer concrete types over unnecessary generics in app code.
- Use `Uuid` for primary IDs where already established.
- Use `chrono::DateTime<Utc>` for persisted timestamps in models and DTOs.
- Use `Vec<u8>` for `BYTEA` values in SQLx-facing code.
- Keep transport types and DB types separate when they serve different roles.

## Error Handling

- Prefer `AppError` for API-facing code paths.
- Prefer `anyhow::Result` only at app boundaries such as `main()` or tooling-like code.
- Do not leak raw database or framework errors to clients.
- Map known DB conflicts into semantic `AppError` variants.
- Let unexpected infrastructure failures become `AppError::Internal`.
- Log internal errors through the centralized error response path.

## Validation

- Use `validator` derives on DTOs.
- Use `axum-valid` through `ValidatedJson<T>` rather than manual field validation in handlers.
- Normalize user input separately from validation when business logic requires it, for example lowercasing emails.

## Auth And Security Notes

- Passwords must be hashed with `argon2`.
- Session cookies should remain `HttpOnly` and `SameSite=Lax` unless requirements change.
- Session tokens are opaque secrets; store only a SHA-256 hash in the database.
- In Postgres, `sessions.token_hash` is `BYTEA`, not text.
- Keep invalid login responses generic to avoid account enumeration.

## Handler Style

- Keep handlers short and linear.
- Parse/validate input first.
- Normalize input second.
- Call DB/domain helpers next.
- Convert to response DTO at the end.
- Prefer early returns for error cases.

## When Changing Schema Or Auth Logic

- Update `migrations/` first.
- Update any manual local DB schema needed for SQLx macro checks.
- Update SQL queries and row structs.
- Run `cargo fmt && cargo check`.
- Run `just sqlx-prepare`.
- Re-test the relevant endpoints with `curl`, `cargo test --test auth_integration`, or targeted tests.

## Manual API Smoke Test Commands

- Register example: `curl -i -X POST http://127.0.0.1:8050/auth/register -H "Content-Type: application/json" -d '{"email":"user@example.com","password":"password123"}'`
- Login example: `curl -i -c cookies.txt -X POST http://127.0.0.1:8050/auth/login -H "Content-Type: application/json" -d '{"email":"user@example.com","password":"password123"}'`
- Me example: `curl -i -b cookies.txt http://127.0.0.1:8050/auth/me`
- Logout example: `curl -i -b cookies.txt -c cookies.txt -X POST http://127.0.0.1:8050/auth/logout`

## Practical Guidance For Agents

- Make the smallest correct change.
- Preserve the current layered structure unless there is a clear reason to refactor it.
- Prefer extending existing modules over introducing many tiny abstractions.
- Keep compile-time SQL checks working.
- If you touch SQL, assume migrations plus `sqlx-prepare` are part of the change.
- Prefer reusing `tests/common/` for any new real-Postgres integration/e2e coverage.
- If you touch handlers, preserve the unified API error format.
