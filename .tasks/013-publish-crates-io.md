---
status: open
type: HITL
blocked-by: [012]
---

# Publish to crates.io

## What to build

Prepare Cargo.toml metadata for crates.io publishing. User performs `cargo login` and `cargo publish`.

## Acceptance criteria

- [ ] Cargo.toml has required metadata: description, license, repository, homepage, keywords, categories
- [ ] `cargo package --list` shows only necessary files (no test fixtures, no .github)
- [ ] User runs `cargo login` with their token
- [ ] `cargo publish` succeeds
- [ ] `cargo install cstat` works from a clean machine
