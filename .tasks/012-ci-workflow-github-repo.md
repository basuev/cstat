---
status: done
type: AFK
blocked-by: [011]
---

# local release + GitHub repo

## What to build

Create GitHub repository via `gh repo create`. Local release script for cross-compilation via `scripts/release.sh`.

Targets:
- macOS arm64 (aarch64-apple-darwin)
- macOS x86_64 (x86_64-apple-darwin)
- Linux x86_64 (x86_64-unknown-linux-musl)
- Linux arm64 (aarch64-unknown-linux-musl)

Script builds locally for installed targets, tags, and creates GitHub Release via `gh`.

## Acceptance criteria

- [x] GitHub repo created and code pushed
- [x] `scripts/release.sh` builds for installed targets
- [x] Creates tag and github release with binaries via `gh`
- [x] README with install instructions (cargo install + binary download)
