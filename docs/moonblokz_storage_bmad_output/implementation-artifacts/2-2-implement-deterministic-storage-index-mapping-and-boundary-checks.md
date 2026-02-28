# Story 2.2: Implement deterministic storage_index mapping and boundary checks

Status: done

## Story

As a MoonBlokz chain developer,
I want deterministic index mapping logic shared by the memory backend contract behavior,
so that save/read operations are predictable and testable.

## Acceptance Criteria

1. Given compile-time memory capacity (`BLOCK_STORAGE_SIZE`), when save/read requests are made with `storage_index`, then valid indices map deterministically to exactly one slot.
2. Out-of-range indices are rejected with explicit `StorageError::InvalidIndex` for both save and read paths.
3. Mapping behavior remains allocation-free and bounded for embedded constraints.
4. Public `StorageTrait` surface remains unchanged (`init`, `save_block`, `read_block`).

## Tasks / Subtasks

- [x] Formalize deterministic index mapping in memory backend
  - [x] Define canonical mapping rule: `slot_index = storage_index as usize`
  - [x] Keep index mapping behavior identical across save/read paths
- [x] Enforce boundary checks consistently
  - [x] Reject out-of-range save indexes with `StorageError::InvalidIndex`
  - [x] Reject out-of-range read indexes with `StorageError::InvalidIndex`
- [x] Strengthen boundary-focused tests
  - [x] Add explicit read-out-of-range test
  - [x] Keep save-out-of-range and round-trip tests deterministic
- [x] Keep implementation minimal
  - [x] No new trait methods or runtime allocation
  - [x] No shared backend implementation code

## Developer Context

### Technical Requirements

- Continue implementation in `moonblokz-storage`.
- Preserve compile-time bounded memory storage model.
- Keep error model payload-free for memory footprint constraints.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/src/backend_memory.rs`
- `/_bmad-output/implementation-artifacts/2-2-implement-deterministic-storage-index-mapping-and-boundary-checks.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 2, Story 2.2)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`
- `/_bmad-output/implementation-artifacts/2-1-implement-in-memory-backend-lifecycle-and-capacity-reporting.md`

## Dev Agent Record

### Completion Notes

- Kept deterministic mapping as direct conversion:
  - `slot_index = storage_index as usize`
- Mapping behavior is identical in both paths:
  - save path bounds-checks `slot_index`
  - read path bounds-checks `slot_index`
- Out-of-range behavior is explicit and deterministic:
  - save out-of-range -> `StorageError::InvalidIndex`
  - read out-of-range -> `StorageError::InvalidIndex`
- Added boundary-focused test:
  - `read_reports_invalid_index_for_out_of_range_slot`
- Maintained minimal implementation:
  - no trait expansion
  - no allocation
  - no shared backend implementation modules

### Validation

- `cargo test` in `moonblokz-storage`: pass
- `./scripts/check_backend_features.sh`: pass

### File List

- `moonblokz-storage/src/backend_memory.rs`
- `/_bmad-output/implementation-artifacts/2-2-implement-deterministic-storage-index-mapping-and-boundary-checks.md`
