#!/usr/bin/env sh
set -eu

cargo check --no-default-features --features backend-memory >/dev/null

RUST_HOST_ARCH="$(rustc -vV | awk '/host:/ {print $2}' | cut -d- -f1)"
if [ "$RUST_HOST_ARCH" = "arm" ]; then
  cargo check --no-default-features --features backend-rp2040 >/dev/null
else
  echo "skipping backend-rp2040 compile check on non-rust-arm host: $RUST_HOST_ARCH"
fi

if cargo check --no-default-features >/dev/null 2>&1; then
  echo "expected failure for no backend feature, but command succeeded" >&2
  exit 1
fi

if cargo check --no-default-features --features "backend-memory backend-rp2040" >/dev/null 2>&1; then
  echo "expected failure for multiple backend features, but command succeeded" >&2
  exit 1
fi

echo "backend feature exclusivity checks passed"
