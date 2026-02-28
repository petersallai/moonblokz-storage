# Story 3.1: Implement RP2040 flash geometry mapping for block slots

Status: done

## Story

As a MoonBlokz storage implementer,
I want compile-time RP2040 page/slot mapping based on `MAX_BLOCK_SIZE`,
so that block placement is deterministic and never crosses page boundaries.

## Acceptance Criteria

1. Given `FLASH_PAGE_SIZE = 4096` and compile-time `MAX_BLOCK_SIZE`, when slot mapping formulas are implemented, then `BLOCKS_PER_PAGE`, `page_index`, `slot_index`, and `byte_offset_in_page` are computed deterministically.
2. Configuration is rejected when `MAX_BLOCK_SIZE` cannot fit at least one block per page.
3. Mapping formulas are documented and test-covered.
4. Implementation stays synchronous, deterministic, and minimal for embedded constraints.

## Tasks / Subtasks

- [x] Add RP2040 geometry constants and mapping helpers
  - [x] `FLASH_PAGE_SIZE = 4096`
  - [x] `BLOCKS_PER_PAGE = FLASH_PAGE_SIZE / MAX_BLOCK_SIZE`
  - [x] `page_index = storage_index / BLOCKS_PER_PAGE`
  - [x] `slot_index = storage_index % BLOCKS_PER_PAGE`
  - [x] `byte_offset_in_page = slot_index * MAX_BLOCK_SIZE`
- [x] Add compile-time guard for invalid geometry
  - [x] Reject config when `BLOCKS_PER_PAGE == 0`
- [x] Add deterministic tests
  - [x] Mapping examples for representative indexes
  - [x] Boundary checks around page transitions
- [x] Keep implementation isolated
  - [x] Place logic in RP2040 backend module only
  - [x] No trait/API expansion

## Developer Context

### Technical Requirements

- Continue implementation in `moonblokz-storage`.
- Use compile-time constants from `moonblokz-chain-types` (`MAX_BLOCK_SIZE`).
- Keep memory footprint minimal and avoid unnecessary abstractions.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/src/backend_rp2040.rs`
- `moonblokz-storage/src/lib.rs` (only if backend wiring/docs need updates)
- `/_bmad-output/implementation-artifacts/3-1-implement-rp2040-flash-geometry-mapping-for-block-slots.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 3, Story 3.1)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`
- `/_bmad-output/implementation-artifacts/2-5-add-ingest-query-integration-tests-on-memory-backend.md`

## Dev Agent Record

### Completion Notes

- Added RP2040 geometry constants in `backend_rp2040.rs`:
  - `FLASH_PAGE_SIZE`
  - `BLOCKS_PER_PAGE`
- Added compile-time geometry guard:
  - rejects build when `BLOCKS_PER_PAGE == 0`.
- Added deterministic mapping contract type and helper:
  - `Rp2040SlotMapping`
  - `map_storage_index(storage_index)`
- Mapping formulas implemented as specified:
  - `page_index = storage_index / BLOCKS_PER_PAGE`
  - `slot_index = storage_index % BLOCKS_PER_PAGE`
  - `byte_offset_in_page = slot_index * MAX_BLOCK_SIZE`
- Added RP2040-feature unit tests:
  - first slot mapping
  - last slot within page mapping
  - page boundary transition mapping

### Validation

- `cargo test` (default backend-memory): pass
- `cargo test --no-default-features --features backend-rp2040`: pass
- `./scripts/check_backend_features.sh`: pass

### File List

- `moonblokz-storage/src/backend_rp2040.rs`
- `/_bmad-output/implementation-artifacts/3-1-implement-rp2040-flash-geometry-mapping-for-block-slots.md`
