# Story 1.4: Enforce backend feature exclusivity and backend module isolation

Status: done

## Story

As a MoonBlokz storage maintainer,
I want compile-time enforcement for exactly one backend feature,
so that builds are deterministic and backend behavior stays isolated.

## Acceptance Criteria

1. Given backend features are declared in `Cargo.toml`, when compile-time guards are added, then build fails if zero or multiple backend features are enabled.
2. Backend modules (`backend_rp2040`, `backend_memory`) are isolated and do not share backend implementation logic.
3. Public `StorageTrait` surface remains backend-agnostic and unchanged by backend-feature wiring.
4. Simplicity-first rules are preserved: minimal gating logic, no unnecessary abstraction layers.

## Tasks / Subtasks

- [x] Add backend feature declarations in `moonblokz-storage/Cargo.toml`
  - [x] Add `backend-memory` and `backend-rp2040` features
  - [x] Keep feature names canonical and explicit
- [x] Add compile-time exclusivity guards in `moonblokz-storage/src/lib.rs`
  - [x] Fail build when no backend feature is enabled
  - [x] Fail build when more than one backend feature is enabled
  - [x] Keep guards simple and deterministic
- [x] Add backend module stubs with isolation boundaries
  - [x] Add `src/backend_memory.rs`
  - [x] Add `src/backend_rp2040.rs`
  - [x] Avoid shared backend implementation modules
- [x] Wire feature-gated module exports in `lib.rs`
  - [x] Expose backend structs only under matching feature
  - [x] Keep core trait/error/types independent from backend modules
- [x] Add validation tests/checks
  - [x] Add compile-fail or build-check workflow for invalid feature combinations
  - [x] Keep tests deterministic and lightweight
- [x] Add mandatory docs
  - [x] Module docs and API docs for new backend stubs
  - [x] Update README for backend feature selection usage

## Developer Context

### Technical Requirements

- Continue implementation in `moonblokz-storage`.
- Preserve existing Story 1.3 API surface (`init`, `save_block`, `read_block`).
- Keep backend behavior isolated in per-backend modules.
- Keep implementation simple and embedded-focused.

### Architecture Compliance

- Follow moonblokz-crypto-lib style for feature-based backend selection.
- Exactly one backend feature must be active at compile-time.
- No shared backend implementation logic between backend modules.
- `moonblokz-chain-types` remains the canonical source for block and hash contracts.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/Cargo.toml`
- `moonblokz-storage/src/lib.rs`
- `moonblokz-storage/src/backend_memory.rs`
- `moonblokz-storage/src/backend_rp2040.rs`
- `moonblokz-storage/README.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 1, Story 1.4)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`
- `/_bmad-output/implementation-artifacts/1-3-define-no-std-synchronous-storage-trait-and-core-error-model.md`

## Dev Agent Record

### Completion Notes

- Added backend feature flags to `moonblokz-storage/Cargo.toml`:
  - `backend-memory`
  - `backend-rp2040`
  - default feature set to `backend-memory`
- Added compile-time exclusivity guards in `moonblokz-storage/src/lib.rs`:
  - build fails when no backend feature is enabled
  - build fails when multiple backend features are enabled
- Added isolated backend modules:
  - `moonblokz-storage/src/backend_memory.rs`
  - `moonblokz-storage/src/backend_rp2040.rs`
- Added feature-gated exports for backend structs in `lib.rs`.
- Added deterministic backend feature combination check script:
  - `moonblokz-storage/scripts/check_backend_features.sh`
- Added `moonblokz-storage/README.md` with backend feature selection guidance.

### Validation

- Executed `cargo fmt && cargo test` in `moonblokz-storage`.
- Executed `./scripts/check_backend_features.sh`:
  - memory-only build: pass
  - rp2040-only build: pass
  - no-feature build: fail (expected)
  - multi-feature build: fail (expected)

### File List

- `moonblokz-storage/Cargo.toml`
- `moonblokz-storage/src/lib.rs`
- `moonblokz-storage/src/backend_memory.rs`
- `moonblokz-storage/src/backend_rp2040.rs`
- `moonblokz-storage/scripts/check_backend_features.sh`
- `moonblokz-storage/README.md`
- `/_bmad-output/implementation-artifacts/1-4-enforce-backend-feature-exclusivity-and-backend-module-isolation.md`
