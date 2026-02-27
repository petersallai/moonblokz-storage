#!/usr/bin/env sh
set -eu

cargo test --no-default-features --features backend-memory
cargo test --no-default-features --features backend-rp2040
./scripts/check_backend_features.sh
