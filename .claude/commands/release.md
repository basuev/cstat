release cstat to crates.io and github.

## steps

1. read current version from `Cargo.toml`
2. find the latest git tag: `git describe --tags --abbrev=0`
3. list all commits since that tag: `git log <tag>..HEAD --oneline`
4. decide the next version based on changes:
   - **patch** (0.1.0 -> 0.1.1): bug fixes, docs, chores, refactors
   - **minor** (0.1.0 -> 0.2.0): new features, significant behavior changes
   - **major** (0.1.0 -> 1.0.0): breaking changes, api removals
   - if no commits since last tag, abort with a message
5. show the user: current version, proposed version, and the commit list. ask for confirmation before proceeding
6. update `version = "..."` in `Cargo.toml` to the new version
7. run `cargo check` to update Cargo.lock
8. commit: `release: v<VERSION>`
9. create annotated git tag: `git tag -a v<VERSION> -m v<VERSION>`
10. run `cargo publish`
11. push commit and tag: `git push origin main && git push origin v<VERSION>`
12. build binaries for available targets:
    - check which targets are installed with `rustup target list --installed`
    - build for each available target from: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-musl, aarch64-unknown-linux-musl
    - for linux-musl targets, use `cross build --release --target <target>` if `cross` is available, otherwise `cargo build --release --target <target>`
    - copy binaries to `target/dist/` with names: cstat-darwin-arm64, cstat-darwin-amd64, cstat-linux-amd64, cstat-linux-arm64
13. create github release: `gh release create v<VERSION> target/dist/* --generate-notes --title v<VERSION>`

## important

- abort if working tree is dirty (uncommitted changes)
- abort if not on `main` branch
- cargo publish must succeed before pushing to github - if it fails, the version commit is local and can be fixed
- all output text must be lowercase (per project convention)
