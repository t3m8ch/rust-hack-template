## ADDED Requirements

### Requirement: Application startup SHALL apply pending database migrations before serving requests
The application SHALL connect to the configured Postgres database and apply all pending SQLx migrations from the repository migration set before it starts accepting HTTP traffic.

#### Scenario: Starting the app with pending migrations
- **WHEN** the application starts against a reachable database that does not yet have the latest schema version
- **THEN** it applies the pending SQLx migrations before binding the HTTP listener

#### Scenario: Starting the app with an up-to-date schema
- **WHEN** the application starts against a reachable database where all migrations are already applied
- **THEN** startup completes successfully without requiring any manual schema bootstrap step

### Requirement: Migration failure SHALL prevent the server from starting
If the application cannot successfully apply the configured SQLx migrations, it SHALL fail startup instead of serving requests against a partially prepared or outdated schema.

#### Scenario: Migration execution fails
- **WHEN** the application starts and a migration returns an error after the database connection is established
- **THEN** the startup path returns an error and the server does not begin listening for requests

### Requirement: Local development bootstrap SHALL use SQLx migrations as the only schema source
The repository SHALL support a fresh local development database without `db/CreateTables.sql` or any compose-mounted schema mirror, relying on the application startup migration step to create the schema.

#### Scenario: Bootstrapping a fresh local database
- **WHEN** a developer starts the Postgres service with an empty data volume and then launches the application
- **THEN** the application creates the required schema from `migrations/` and the local setup does not depend on `db/CreateTables.sql`
