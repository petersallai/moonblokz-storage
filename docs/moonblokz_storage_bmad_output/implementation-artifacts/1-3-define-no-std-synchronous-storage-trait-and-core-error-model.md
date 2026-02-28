# Story 1.3: Define no_std synchronous storage trait and core error model

Status: done

## Story

As a MoonBlokz chain developer,
I want a synchronous `no_std` storage API trait in `moonblokz-storage`,
so that chain runtime can call storage deterministically on constrained devices.

## Acceptance Criteria

1. Given `moonblokz-storage` exists, when the public trait API is implemented in `src/lib.rs`, then APIs cover initialization, save by `storage_index`, and read by `storage_index`.
2. Error variants clearly separate invalid index, absent block, integrity failure, and backend IO failures.
3. API remains synchronous and `no_std`, aligned with RP2040 XIP constraints.
4. Simplicity-first implementation discipline is preserved (minimal abstractions, minimal state, embedded-friendly surface).

## Tasks / Subtasks

- [x] Define public storage trait in `moonblokz-storage/src/lib.rs`
  - [x] Add synchronous trait methods for init, save, retrieve
  - [x] Keep method inputs/outputs deterministic and bounded
  - [x] Use canonical `storage_index` naming
- [x] Define core typed error model in `moonblokz-storage/src/error.rs`
  - [x] Add distinct variants for invalid index, block absent, integrity failure, backend IO failure
  - [x] Keep variants backend-agnostic at API level
  - [x] Keep error model simple and explicit for chain-level recovery logic
- [x] Define storage-facing types in `moonblokz-storage/src/types.rs`
  - [x] Keep canonical `StorageIndex` type alias for API consistency
- [x] Wire exports in `moonblokz-storage/src/lib.rs`
  - [x] Export trait and core API types/errors
  - [x] Keep public API concise and documentation-complete
- [x] Add deterministic tests for trait/error contract surface
  - [x] Unit tests for error/category mapping and boundary behavior contracts
  - [x] Compile-time checks preserve `no_std`-friendly API surface
- [x] Add mandatory documentation
  - [x] Module docs at file top
  - [x] Function/struct/field docs with input descriptions
  - [x] At least one usage example for each public function

## Developer Context

### Technical Requirements

- Implement in `moonblokz-storage` only for this story.
- API must be synchronous + `no_std` and deterministic.
- Storage library owns storage mechanics only; chain-policy behavior remains external.
- Hash verification behavior is backend-implemented; canonical hash utility contract comes from `moonblokz-chain-types` (`calculate_hash`, `HASH_SIZE`).
- Keep implementation as simple as possible for embedded CPU/RAM/flash constraints.

### Architecture Compliance

- `moonblokz-storage` depends on `moonblokz-chain-types`; do not redefine canonical block layout or hash semantics.
- Public API is trait-defined in `lib.rs`; chain may use concrete backend structs directly later.
- Avoid unnecessary abstractions and binary-size overhead.
- Logging standard for runtime code is `log` crate macros (if logging is introduced).

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/src/lib.rs`
- `moonblokz-storage/src/error.rs`
- `moonblokz-storage/src/types.rs`
- `moonblokz-storage/Cargo.toml` (only if required for `no_std`/feature setup)

Do not modify backend implementation modules in this story.

### Testing Requirements

- Add deterministic unit tests for API/error/type contracts.
- Validate boundary-focused behavior (index validity and explicit slot outcomes).
- Run full crate tests for `moonblokz-storage`.

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 1, Story 1.3)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`
- `/_bmad-output/implementation-artifacts/1-2-define-chain-types-serialization-and-hash-boundary-contracts.md`

## Dev Agent Record

### Completion Notes

- Added `StorageTrait` in `moonblokz-storage/src/lib.rs` with synchronous, `no_std`-friendly methods:
  - `init`
  - `save_block`
  - `read_block`
- Added core API error model in `moonblokz-storage/src/error.rs`:
  - `InvalidIndex`
  - `BlockAbsent`
  - `IntegrityFailure`
  - `BackendIo`
- Error variants intentionally avoid returning `StorageIndex` payloads to keep memory footprint minimal.
- Added public contract types in `moonblokz-storage/src/types.rs`:
  - `StorageIndex`
- Added exports in `moonblokz-storage/src/lib.rs`.
- Added deterministic unit tests and rustdoc examples.
- Added dependency on `moonblokz-chain-types` for canonical `Block` contract usage.

### Validation

- Executed `cargo fmt && cargo test` in `moonblokz-storage`.
- Result: unit tests pass and doc tests pass.

### File List

- `moonblokz-storage/Cargo.toml`
- `moonblokz-storage/src/lib.rs`
- `moonblokz-storage/src/error.rs`
- `moonblokz-storage/src/types.rs`
- `/_bmad-output/implementation-artifacts/1-3-define-no-std-synchronous-storage-trait-and-core-error-model.md`
