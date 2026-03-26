#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?usage: ./scripts/release.sh v0.1.0}"

if ! command -v gh &>/dev/null; then
  echo "error: gh cli required" >&2
  exit 1
fi

if ! command -v cargo &>/dev/null; then
  echo "error: cargo required" >&2
  exit 1
fi

if ! docker info &>/dev/null; then
  echo "error: docker required for linux builds" >&2
  exit 1
fi

ARTIFACTS=()
DIST="target/dist"
rm -rf "$DIST"
mkdir -p "$DIST"

echo "build: aarch64-apple-darwin -> cstat-darwin-arm64"
cargo build --release --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/cstat "${DIST}/cstat-darwin-arm64"
ARTIFACTS+=("${DIST}/cstat-darwin-arm64")

echo "build: x86_64-apple-darwin -> cstat-darwin-amd64"
cargo build --release --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/cstat "${DIST}/cstat-darwin-amd64"
ARTIFACTS+=("${DIST}/cstat-darwin-amd64")

echo "build: x86_64-unknown-linux-musl -> cstat-linux-amd64"
docker run --rm --platform linux/amd64 -v "$(pwd)":/src -w /src \
  messense/rust-musl-cross:x86_64-musl \
  cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/cstat "${DIST}/cstat-linux-amd64"
ARTIFACTS+=("${DIST}/cstat-linux-amd64")

echo "build: aarch64-unknown-linux-musl -> cstat-linux-arm64"
docker run --rm --platform linux/arm64 -v "$(pwd)":/src -w /src \
  messense/rust-musl-cross:aarch64-musl \
  cargo build --release --target aarch64-unknown-linux-musl
cp target/aarch64-unknown-linux-musl/release/cstat "${DIST}/cstat-linux-arm64"
ARTIFACTS+=("${DIST}/cstat-linux-arm64")

echo ""
echo "artifacts:"
for a in "${ARTIFACTS[@]}"; do
  echo "  $(basename "$a") ($(du -h "$a" | cut -f1 | xargs))"
done

echo ""
echo "creating release ${VERSION}..."
git tag -a "$VERSION" -m "$VERSION"
git push origin "$VERSION"
gh release create "$VERSION" "${ARTIFACTS[@]}" --generate-notes --title "$VERSION"

echo "done: https://github.com/basuev/cstat/releases/tag/${VERSION}"
