# Add New Device Support

This guide describes how to add a new storage backend in `moonblokz-storage`
without changing chain-level semantics.

## Scope and Contract

- Backends implement `StorageTrait` only.
- Backends do not implement chain policy (no prune/reclaim/fork decisions).
- API is synchronous by design.
- Error categories must remain:
  - `InvalidIndex`
  - `BlockAbsent`
  - `IntegrityFailure`
  - `BackendIo { code }`

## Step 1: Add Backend Feature Flag

Edit `Cargo.toml`:

```toml
[features]
default = ["backend-memory"]
backend-memory = []
backend-rp2040 = []
backend-yourdevice = []
```

If target-specific dependencies are needed, add them under a target table or
behind feature-gated optional dependencies.

## Step 2: Add Backend Module

Create `src/backend_yourdevice.rs` with:

- module-level block comment at file start
- backend struct
- constructor(s)
- `StorageTrait` implementation

Use fixed-size slot model with deterministic index-to-address mapping. Keep
RAM usage bounded and avoid dynamic allocations in hot paths.

## Step 3: Wire Module in `lib.rs`

Add feature-gated module export:

```rust
#[cfg(feature = "backend-yourdevice")]
pub mod backend_yourdevice;
```

Add public type export and alias mapping:

```rust
#[cfg(feature = "backend-yourdevice")]
pub use backend_yourdevice::YourDeviceBackend;

#[cfg(feature = "backend-yourdevice")]
pub type MoonblokzStorage<const STORAGE_SIZE: usize> = YourDeviceBackend<STORAGE_SIZE>;
```

Update feature-exclusivity compile guards in `src/lib.rs` so exactly one
backend feature must be enabled.

## Step 4: Implement Required Semantics

Required behavior:

- `save_block`:
  - validate index bounds
  - persist fixed-size block bytes
  - persist hash metadata for retrieve-time integrity verification
- `read_block`:
  - validate index bounds
  - return `BlockAbsent` for empty slot
  - recompute and compare hash; return `IntegrityFailure` on mismatch
  - return parsed block on success
- `init`:
  - backend-local initialization only
  - no chain-level reconstruction logic inside backend

## Step 5: BackendIo Error Code Map

Add backend-specific `BackendIo { code }` documentation:

- update `src/error.rs` code mapping comments
- update `README.md` code mapping section

Codes must be deterministic and stable for integration troubleshooting.

## Step 6: Add Backend Tests

In backend module tests, include at minimum:

- deterministic mapping tests
- valid save/read round-trip
- invalid index handling
- empty slot handling
- integrity mismatch handling
- startup-style mixed-slot scan outcomes

Add integration-style ingest/startup/query flow tests for representative valid
and corrupted datasets.

## Step 7: Add Conformance Support

Update `src/conformance.rs`:

- add feature-gated `new_backend()` constructor path for your backend
- define deterministic `TEST_STORAGE_SIZE`
- keep shared conformance scenarios backend-agnostic

Conformance tests must pass for your backend with feature-isolated test runs.

## Step 8: Update Matrix Runner and CI

Update:

- `run_tests.sh`
- `.github/workflows/rust.yml`

Add your backend feature to matrix entries so CI runs:

- `cargo test --no-default-features --features backend-yourdevice`
- feature exclusivity checks

## Step 9: Documentation Checklist

Required documentation before review:

- every source file has a module-level block comment
- every public function documents:
  - `Parameters`
  - at least one `Example`
- README includes integration notes if backend has special target/setup needs

## Step 10: Final Validation Checklist

Run:

```sh
cargo test --no-default-features --features backend-memory
cargo test --no-default-features --features backend-rp2040
cargo test --no-default-features --features backend-yourdevice
./scripts/check_backend_features.sh
./run_tests.sh
```

If all checks pass and docs are complete, backend is ready for code review.
