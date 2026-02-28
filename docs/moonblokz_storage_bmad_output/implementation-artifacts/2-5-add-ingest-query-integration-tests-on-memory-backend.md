# Story 2.5: Add ingest/query integration tests on memory backend

Status: done

## Story

As a MoonBlokz chain developer,
I want integration-focused tests for ingest and query usage patterns,
so that runtime behaviors are validated before RP2040 deployment.

## Acceptance Criteria

1. Given memory backend test setup, when ingest-style save and query-style read flows are executed, then expected blocks are retrievable by index with deterministic behavior.
2. Integration tests cover invalid-index and empty-slot scenarios with typed outcomes:
   - `StorageError::InvalidIndex`
   - `StorageError::BlockAbsent`
3. Tests stay aligned with current simplified API (`init`, `save_block`, `read_block`) and compile-time bounded memory model.
4. Test coverage remains deterministic and allocation-free.

## Tasks / Subtasks

- [x] Add ingest/query integration-focused test cases in memory backend module
  - [x] Ingest sequence saving multiple blocks at distinct indexes
  - [x] Query sequence reading those blocks and verifying byte equality
- [x] Add negative-path integration checks
  - [x] Query empty valid slot -> `BlockAbsent`
  - [x] Save/read out-of-range -> `InvalidIndex`
- [x] Add combined flow test
  - [x] Startup-style read cycle followed by ingest saves and query reads
  - [x] Ensure no hidden state reset in read path
- [x] Keep test design minimal
  - [x] No trait/API changes
  - [x] No shared backend logic introduced

## Developer Context

### Technical Requirements

- Continue implementation in `moonblokz-storage`.
- Preserve compile-time bounded memory backend and payload-free errors.
- Keep tests deterministic and embedded-friendly.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/src/backend_memory.rs`
- `/_bmad-output/implementation-artifacts/2-5-add-ingest-query-integration-tests-on-memory-backend.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 2, Story 2.5)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`
- `/_bmad-output/implementation-artifacts/2-4-implement-startup-reconstruction-read-cycle-support-apis.md`

## Dev Agent Record

### Completion Notes

- Added integration-focused memory-backend coverage for ingest/query flows:
  - multi-index save/retrieve with byte equality checks
  - explicit overwrite and mixed-slot query behavior validation
- Added dedicated integration test:
  - `ingest_query_integration_flow_covers_positive_and_negative_paths`
  - includes startup-empty reads, ingest saves, query reads, and negative path checks
- Confirmed typed outcome behavior remains stable:
  - empty slot -> `BlockAbsent`
  - out-of-range save/read -> `InvalidIndex`
- Kept implementation minimal:
  - no API/trait changes
  - no shared backend logic

### Validation

- `cargo test` in `moonblokz-storage`: pass
- `./scripts/check_backend_features.sh`: pass

### File List

- `moonblokz-storage/src/backend_memory.rs`
- `/_bmad-output/implementation-artifacts/2-5-add-ingest-query-integration-tests-on-memory-backend.md`
