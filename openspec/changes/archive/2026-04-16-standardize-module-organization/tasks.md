## 1. Reorganize auth-related extractor code

- [x] 1.1 Create `src/extractors/auth.rs` and move the `CurrentUser` extractor implementation into the extractor layer.
- [x] 1.2 Update `src/extractors/mod.rs` to declare the new auth extractor module and re-export `CurrentUser` alongside `ValidatedJson`.
- [x] 1.3 Remove `src/auth/extractor.rs` usage and keep `src/auth/mod.rs` focused on password/session domain helpers.

## 2. Align imports and public module surfaces

- [x] 2.1 Update handler and crate imports so auth handlers consume `CurrentUser` and `ValidatedJson` from `crate::extractors`.
- [x] 2.2 Adjust any module declarations or re-exports in `src/main.rs` and related files so the new layout compiles cleanly.

## 3. Document and verify the convention

- [x] 3.1 Update `AGENTS.md` to document the layer-first placement rule for feature-specific extractors, DTOs, and DB helpers.
- [x] 3.2 Run `cargo fmt`, `cargo check`, and `cargo clippy --all-targets --all-features -- -D warnings` to verify the refactor.
