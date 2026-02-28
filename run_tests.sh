#!/usr/bin/env sh
set -eu

./scripts/run_backend_matrix.sh
./scripts/check_backend_features.sh
