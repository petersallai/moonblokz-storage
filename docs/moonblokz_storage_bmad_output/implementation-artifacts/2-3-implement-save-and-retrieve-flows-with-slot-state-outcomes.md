# Story 2.3: Implement save and retrieve flows with typed slot outcomes

Status: done

## Story

As a MoonBlokz chain runtime,
I want to save and retrieve blocks by index with clear typed outcomes,
so that ingest and query paths can make correct decisions.

## Acceptance Criteria

1. Given a block and target `storage_index`, when save succeeds and retrieve is called, then retrieve returns the same block bytes for populated valid slots.
2. Retrieve distinguishes typed outcomes via existing error/result contract:
   - populated slot -> `Ok(Block)`
   - empty slot -> `Err(StorageError::BlockAbsent)`
   - invalid index -> `Err(StorageError::InvalidIndex)`
3. Save/retrieve flows remain deterministic, synchronous, and allocation-free for memory backend.
4. Public `StorageTrait` surface remains unchanged (`init`, `save_block`, `read_block`).

## Tasks / Subtasks

- [x] Validate save-retrieve behavior on memory backend
  - [x] Save at valid index and read same bytes back
  - [x] Overwrite behavior remains deterministic for same index
- [x] Validate typed error outcomes
  - [x] Empty slot read returns `StorageError::BlockAbsent`
  - [x] Invalid index save/read return `StorageError::InvalidIndex`
- [x] Strengthen test coverage for ingest/query-like flows
  - [x] Add multi-index save/retrieve tests within compile-time capacity
  - [x] Keep tests deterministic and no-alloc
- [x] Keep implementation minimal and API-stable
  - [x] No new trait methods
  - [x] No shared backend logic introduced

## Developer Context

### Technical Requirements

- Continue implementation in `moonblokz-storage`.
- Use current `MemoryBackend<const BLOCK_STORAGE_SIZE: usize>` storage model.
- Preserve payload-free error model for memory footprint constraints.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/src/backend_memory.rs`
- `/_bmad-output/implementation-artifacts/2-3-implement-save-and-retrieve-flows-with-slot-state-outcomes.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 2, Story 2.3)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`
- `/_bmad-output/implementation-artifacts/2-2-implement-deterministic-storage-index-mapping-and-boundary-checks.md`

## Dev Agent Record

### Completion Notes

- Verified save/retrieve behavior for valid populated slots with byte-equality assertions.
- Added deterministic overwrite test for same `storage_index`:
  - latest saved block is returned.
- Added multi-index ingest/query-like coverage:
  - saving different blocks at different indices
  - retrieving each index returns expected bytes.
- Confirmed typed outcomes:
  - empty slot read -> `StorageError::BlockAbsent`
  - invalid index save/read -> `StorageError::InvalidIndex`
- Preserved minimal API and implementation constraints:
  - no trait expansion
  - no allocation
  - compile-time bounded memory model retained.

### Validation

- `cargo test` in `moonblokz-storage`: pass
- `./scripts/check_backend_features.sh`: pass

### File List

- `moonblokz-storage/src/backend_memory.rs`
- `/_bmad-output/implementation-artifacts/2-3-implement-save-and-retrieve-flows-with-slot-state-outcomes.md`
