## Why

The current template mixes two organization styles: most transport and persistence code is grouped by layer (`src/dto/auth.rs`, `src/db/auth.rs`, `src/extractors/validated_json.rs`), while `CurrentUser` lives inside `src/auth/extractor.rs`. That makes it harder to predict where new code should go and increases friction when extending the template during a hackathon.

## What Changes

- Standardize on a layer-first module layout for feature-specific transport and persistence concerns.
- Move the auth-specific request extractor into `src/extractors/auth.rs` so extractors follow the same placement pattern as DTOs and DB helpers.
- Keep `src/auth/` focused on auth domain services and utilities such as password hashing, session management, and other auth-specific business helpers.
- Update public re-exports, imports, and repository guidance so the placement rule is explicit for future changes.

## Capabilities

### New Capabilities
- `module-organization`: Define a single repository convention for where feature-specific extractors, DTOs, DB helpers, and domain services belong.

### Modified Capabilities
- None.

## Impact

- Affected code: `src/auth/`, `src/extractors/`, `src/rest/auth.rs`, `src/main.rs`, and any module re-exports tied to `CurrentUser`.
- APIs: No HTTP or WebSocket contract changes.
- Dependencies: No new external dependencies.
- Tooling and docs: Repository guidance should reflect the new organization rule so future additions follow the same style.
