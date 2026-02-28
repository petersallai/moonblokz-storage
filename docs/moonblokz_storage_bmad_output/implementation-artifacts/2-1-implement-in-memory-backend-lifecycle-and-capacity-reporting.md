# Story 2.1: Implement in-memory backend lifecycle with compile-time capacity

Status: done

## Story

As a MoonBlokz chain developer,
I want an in-memory backend implementing the storage trait,
so that chain logic can be tested off-target without RP2040 hardware.

## Acceptance Criteria

1. Given backend feature `backend-memory` is enabled, when memory backend lifecycle is implemented, then backend initializes/reset state deterministically via `init`.
2. Memory capacity is compile-time bounded by `BLOCK_STORAGE_SIZE` and does not allocate unbounded runtime memory.
3. Save/read operations use deterministic index checks against compile-time bounds and return explicit errors for out-of-range indices.
4. Public `StorageTrait` surface remains unchanged (`init`, `save_block`, `read_block`) with no additional capacity/slot query methods.

## Tasks / Subtasks

- [x] Finalize `MemoryBackend<const BLOCK_STORAGE_SIZE: usize>` behavior
  - [x] Keep storage array compile-time bounded (`[Option<Block>; BLOCK_STORAGE_SIZE]`)
  - [x] Ensure `init` resets state predictably to empty
- [x] Validate deterministic index-boundary handling
  - [x] Save rejects out-of-range index with `StorageError::InvalidIndex`
  - [x] Read rejects out-of-range index with `StorageError::InvalidIndex`
  - [x] Read returns `StorageError::BlockAbsent` for empty valid slot
- [x] Keep backend isolated and simple
  - [x] No shared backend implementation modules
  - [x] No unnecessary abstractions or dynamic allocation
- [x] Add/adjust tests for memory backend contract
  - [x] Compile-time capacity enforcement test
  - [x] Init-reset behavior test
  - [x] Save/read success and error path coverage
- [x] Keep docs aligned with simplified API
  - [x] Rustdoc examples for `MemoryBackend::new()` and core flows
  - [x] Update Story records with implemented behavior

## Developer Context

### Technical Requirements

- Continue implementation in `moonblokz-storage`.
- Preserve `StorageTrait` as synchronous + `no_std`.
- Maintain minimal memory footprint and deterministic behavior.
- Do not reintroduce runtime capacity or slot-state trait methods.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/src/backend_memory.rs`
- `moonblokz-storage/src/lib.rs` (only if wiring/docs need adjustments)
- `/_bmad-output/implementation-artifacts/2-1-implement-in-memory-backend-lifecycle-and-capacity-reporting.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 2, Story 2.1)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`
- `/_bmad-output/implementation-artifacts/1-4-enforce-backend-feature-exclusivity-and-backend-module-isolation.md`

## Dev Agent Record

### Completion Notes

- Implemented compile-time bounded memory backend storage using:
  - `MemoryBackend<const BLOCK_STORAGE_SIZE: usize>`
  - internal storage: `[Option<Block>; BLOCK_STORAGE_SIZE]`
- Lifecycle behavior:
  - `new()` initializes all slots to `None`
  - `init()` resets all slots to `None`
- Deterministic index handling:
  - out-of-range `save_block` -> `StorageError::InvalidIndex`
  - out-of-range `read_block` -> `StorageError::InvalidIndex`
  - empty valid slot `read_block` -> `StorageError::BlockAbsent`
- Added/updated unit tests in `backend_memory.rs`:
  - compile-time capacity enforcement
  - absent read behavior
  - save/read round-trip
  - init-reset behavior

### Validation

- `cargo test` in `moonblokz-storage`: pass
- `./scripts/check_backend_features.sh`: pass

### File List

- `moonblokz-storage/src/backend_memory.rs`
- `/_bmad-output/implementation-artifacts/2-1-implement-in-memory-backend-lifecycle-and-capacity-reporting.md`
