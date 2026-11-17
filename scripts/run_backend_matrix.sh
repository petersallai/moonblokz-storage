#!/usr/bin/env sh
set -eu

# Every storage backend under every signature scheme (DEC-255).
for scheme in schnorr bls-min-sig bls-min-pk; do
  echo "=== Backend matrix: backend-memory, $scheme ==="
  cargo test --no-default-features --features "backend-memory,$scheme,block-size-2016"

  echo "=== Backend matrix: backend-rp2040, $scheme ==="
  cargo test --no-default-features --features "backend-rp2040,$scheme,block-size-2016"
done
