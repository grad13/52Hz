# tests

Tests for the 52Hz app. Integration tests live in `integration/`
(`app_lifecycle.rs`, the canonical source). Rust unit tests are inline
`#[cfg(test)]` under `code/app/tauri/src/`.

## Naming

Source-mirrored names (mirror the module); no date prefix.

## Frontmatter

Managed by `normalize-frontmatter` (run it; do not hand-edit).
