## Context

This template already has clear top-level layer directories for HTTP handlers (`src/rest`), DTOs (`src/dto`), DB helpers (`src/db`), and reusable extractors (`src/extractors`). The auth feature currently breaks that pattern by keeping `CurrentUser` inside `src/auth/extractor.rs`, even though handlers consume it the same way they consume `ValidatedJson` from the shared extractor layer. The result is an organization rule that is easy to violate because similar concerns are split between layer-first and feature-first placement.

The requested refactor should improve predictability without changing runtime behavior, endpoint contracts, session handling, or database access semantics.

## Goals / Non-Goals

**Goals:**
- Make code placement predictable for auth-related extractors, DTOs, and DB helpers.
- Adopt one rule for transport and persistence surface code: place feature-specific implementations in the corresponding top-level layer module under a feature-named file.
- Keep `src/auth/` dedicated to auth domain services and helper logic.
- Preserve existing HTTP behavior and keep the refactor low-risk.

**Non-Goals:**
- Rework the authentication flow, validation logic, or cookie/session behavior.
- Introduce a full feature-first repository layout.
- Rename endpoints, DTO fields, database tables, or SQL queries unless required by module moves.
- Refactor unrelated modules outside the organization cleanup.

## Decisions

### 1. Standardize on layer-first placement for feature-facing adapter code
Feature-specific DTOs, SQL helpers, and request extractors will live in their layer directories using the feature name as the file name, such as `src/dto/auth.rs`, `src/db/auth.rs`, and `src/extractors/auth.rs`.

**Rationale:**
- This matches the majority of the current repository layout.
- It keeps all handler-facing adapter code in predictable places.
- It minimizes churn compared with moving DTOs and DB helpers into feature subtrees.

**Alternative considered:** move all auth-related code under `src/auth/` with nested `dto`, `db`, and `extractor` modules. This was rejected because it would be a broader architectural shift, would conflict with the existing project layout guidance, and would require more future migration work for other features.

### 2. Keep `src/auth/` for auth domain services and workflows
The `src/auth/` module will retain password hashing, session token management, and other auth-specific business helpers, but it will no longer own HTTP extractor definitions.

**Rationale:**
- It separates transport adapters from domain logic.
- It makes `src/auth/` easier to scan as the feature service layer.
- It clarifies that extractors belong to the extractor layer, even when they are feature-specific.

**Alternative considered:** continue re-exporting the extractor from `src/auth/` after moving its file. This was rejected because it preserves the same ambiguity the change is trying to remove.

### 3. Preserve ergonomic imports through `src/extractors/mod.rs`
`src/extractors/mod.rs` should re-export both shared and feature-specific extractor types used by handlers, such as `ValidatedJson` and `CurrentUser`.

**Rationale:**
- Handler imports stay simple and consistent.
- The public API reflects the repository rule: handlers pull extractors from `crate::extractors`.
- File moves remain internal implementation details.

## Risks / Trade-offs

- **Cross-module dependency remains**: `src/extractors/auth.rs` will still depend on auth session lookup helpers. → Mitigation: keep the dependency narrow and one-directional (`extractors` depends on auth service functions, not the reverse).
- **Small refactor churn**: moving files and imports can temporarily break compilation. → Mitigation: perform the move with matching re-exports and immediately run `cargo fmt`, `cargo check`, and `cargo clippy`.
- **Convention drift later**: future contributors may reintroduce mixed placement if the rule is undocumented. → Mitigation: update repository guidance in `AGENTS.md` and keep module names aligned across layers.

## Migration Plan

1. Create `src/extractors/auth.rs` and move the `CurrentUser` extractor there.
2. Update `src/extractors/mod.rs` to expose both `ValidatedJson` and `CurrentUser`.
3. Remove the old `src/auth/extractor.rs` module and its re-export from `src/auth/mod.rs`.
4. Update imports in handlers and other call sites.
5. Document the placement convention in `AGENTS.md`.
6. Run formatting and compile/lint verification.

Rollback is straightforward: restore `src/auth/extractor.rs`, revert the updated exports/imports, and remove the new extractor-layer file.

## Open Questions

- For future features, should layer-specific folders like `src/dto/` and `src/db/` remain required even when a feature is very small? The default in this change is yes, to preserve consistency, but small one-file exceptions may still be worth discussing separately.
