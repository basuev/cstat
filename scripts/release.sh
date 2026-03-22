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

TARGETS=(
  aarch64-apple-darwin:cstat-darwin-arm64
  x86_64-apple-darwin:cstat-darwin-amd64
  x86_64-unknown-linux-musl:cstat-linux-amd64
  aarch64-unknown-linux-musl:cstat-linux-arm64
)

ARTIFACTS=()
DIST="target/dist"
rm -rf "$DIST"
mkdir -p "$DIST"

for entry in "${TARGETS[@]}"; do
  target="${entry%%:*}"
  name="${entry##*:}"

  if ! rustup target list --installed | grep -q "^${target}$"; then
    echo "skip: ${target} (not installed, run: rustup target add ${target})" >&2
    continue
  fi

  USE_CROSS=false
  if [[ "$target" == *linux-musl* ]] && command -v cross &>/dev/null; then
    USE_CROSS=true
  fi

  echo "build: ${target} -> ${name}"
  if $USE_CROSS; then
    cross build --release --target "$target"
  else
    cargo build --release --target "$target"
  fi

  cp "target/${target}/release/cstat" "${DIST}/${name}"
  ARTIFACTS+=("${DIST}/${name}")
done

if [ ${#ARTIFACTS[@]} -eq 0 ]; then
  echo "error: no artifacts built" >&2
  exit 1
fi

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
