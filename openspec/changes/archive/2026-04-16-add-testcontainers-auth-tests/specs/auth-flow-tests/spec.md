## ADDED Requirements

### Requirement: Auth integration tests SHALL validate the cookie-session flow against a real Postgres database
The repository SHALL include integration tests that exercise the Axum router/service directly against a migrated Postgres test database and verify the auth flow outcomes for registration, duplicate registration, login, invalid credentials, authenticated `me`, unauthenticated `me`, logout, and post-logout access.

#### Scenario: Running the auth integration suite
- **WHEN** the integration test suite executes against the router with a real Postgres-backed application state
- **THEN** it verifies successful registration, duplicate-email rejection, successful login, wrong-password rejection, `me` returning unauthorized without a cookie, `me` returning the user with a valid cookie, logout clearing the session, and `me` returning unauthorized after logout

### Requirement: Auth end-to-end tests SHALL verify at least one full happy path over real HTTP
The repository SHALL include an end-to-end auth smoke test that binds the application to `127.0.0.1:0`, uses `reqwest::Client` with cookie persistence, and validates the full register → login → me → logout → me=401 sequence against the real Postgres-backed server.

#### Scenario: Executing the auth happy-path e2e test
- **WHEN** the end-to-end test starts the server on an ephemeral local port and drives requests through a cookie-enabled HTTP client
- **THEN** registration succeeds, login establishes the session cookie, `me` returns the authenticated user, logout succeeds, and a follow-up `me` request returns `401 Unauthorized`