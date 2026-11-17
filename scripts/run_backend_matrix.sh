#!/usr/bin/env sh
set -eu

echo "=== Backend matrix: backend-memory ==="
cargo test --no-default-features --features backend-memory,schnorr,block-size-2016

echo "=== Backend matrix: backend-rp2040 ==="
cargo test --no-default-features --features backend-rp2040,schnorr,block-size-2016
