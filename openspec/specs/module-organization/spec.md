# module-organization Specification

## Purpose
TBD - created by archiving change standardize-module-organization. Update Purpose after archive.
## Requirements
### Requirement: Feature-facing adapter code SHALL follow a layer-first module layout
The repository SHALL place feature-specific request extractors, DTOs, and database helpers inside the corresponding top-level layer module using the feature name as the file grouping.

#### Scenario: Adding auth-specific transport and persistence types
- **WHEN** the project defines auth-specific request extractors, DTOs, or database helpers
- **THEN** those modules are organized under the layer directories as `src/extractors/auth.rs`, `src/dto/auth.rs`, and `src/db/auth.rs` instead of mixing them into unrelated feature service modules

### Requirement: Feature service modules SHALL contain domain helpers, not HTTP extractor definitions
Feature modules such as `src/auth/` SHALL be used for domain services, workflows, and business helpers, and SHALL NOT be the default location for HTTP extractor types.

#### Scenario: Organizing the auth module
- **WHEN** the project contains password hashing, session management, and current-user lookup workflows for authentication
- **THEN** those helpers remain in `src/auth/` while the `CurrentUser` request extractor is exposed from the extractor layer

### Requirement: Handler imports SHALL remain consistent through extractor re-exports
The extractor layer SHALL provide a stable public interface for handlers by re-exporting both shared and feature-specific extractor types from `src/extractors/mod.rs`.

#### Scenario: Consuming extractors in a handler
- **WHEN** an HTTP handler needs `ValidatedJson` and `CurrentUser`
- **THEN** the handler imports both types from `crate::extractors` instead of mixing imports across unrelated modules

