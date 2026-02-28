# Story 3.2: Implement synchronous RP2040 save path by storage_index

Status: done

## Story

As a MoonBlokz chain runtime,
I want to synchronously save blocks to RP2040 flash by index,
so that accepted blocks are durably persisted during ingest.

## Acceptance Criteria

1. Given a valid `storage_index` and block, when save is invoked on RP2040 backend, then data is written to the mapped page/slot synchronously.
2. Invalid indices return explicit `StorageError::InvalidIndex`.
3. Flash write failures return explicit backend I/O errors.
4. Save path preserves deterministic mapping from Story 3.1 and keeps API unchanged (`save_block`).

## Tasks / Subtasks

- [x] Implement RP2040 save flow in `backend_rp2040.rs`
  - [x] Validate `storage_index` bounds
  - [x] Resolve page/slot mapping via `map_storage_index`
  - [x] Perform synchronous page/slot write operation (stubbed in host tests)
- [x] Add explicit error behavior
  - [x] Invalid index -> `StorageError::InvalidIndex`
  - [x] Flash write failure -> `StorageError::BackendIo { code: ... }`
- [x] Add deterministic tests for save behavior
  - [x] Valid index path success semantics (using RP2040 backend test double/stub)
  - [x] Invalid index rejection test
  - [x] Backend I/O failure propagation test
- [x] Keep implementation minimal
  - [x] No trait/API expansion
  - [x] No async path

## Developer Context

### Technical Requirements

- Continue implementation in `moonblokz-storage`.
- Preserve synchronous API by design (RP2040 XIP constraints).
- Use u32-based arithmetic for RP2040-specific mapping as agreed.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/src/backend_rp2040.rs`
- `/_bmad-output/implementation-artifacts/3-2-implement-synchronous-rp2040-save-path-by-storage-index.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 3, Story 3.2)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`
- `/_bmad-output/implementation-artifacts/3-1-implement-rp2040-flash-geometry-mapping-for-block-slots.md`

## Dev Agent Record

### Completion Notes

- Implemented synchronous RP2040 save path in `backend_rp2040.rs` with:
  - explicit `storage_index` bounds check against backend capacity
  - deterministic mapping via existing `map_storage_index`
  - synchronous slot write hook (`write_slot`) used by `save_block`
- Added explicit save-path errors:
  - out-of-range `storage_index` -> `StorageError::InvalidIndex`
  - simulated flash write failure -> `StorageError::BackendIo { code }`
- Added deterministic RP2040 save tests:
  - valid index save success
  - invalid index rejection
  - backend I/O failure propagation
- Kept implementation minimal and synchronous with no trait changes.

### Validation

- `cargo test` (default backend-memory): pass
- `cargo test --no-default-features --features backend-rp2040`: pass
- `./scripts/check_backend_features.sh`: pass

### File List

- `moonblokz-storage/src/backend_rp2040.rs`
- `/_bmad-output/implementation-artifacts/3-2-implement-synchronous-rp2040-save-path-by-storage-index.md`
