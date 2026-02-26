#!/usr/bin/env sh
set -eu

cargo check --no-default-features --features backend-memory >/dev/null
cargo check --no-default-features --features backend-rp2040 >/dev/null

if cargo check --no-default-features >/dev/null 2>&1; then
  echo "expected failure for no backend feature, but command succeeded" >&2
  exit 1
fi

if cargo check --no-default-features --features "backend-memory backend-rp2040" >/dev/null 2>&1; then
  echo "expected failure for multiple backend features, but command succeeded" >&2
  exit 1
fi

echo "backend feature exclusivity checks passed"
