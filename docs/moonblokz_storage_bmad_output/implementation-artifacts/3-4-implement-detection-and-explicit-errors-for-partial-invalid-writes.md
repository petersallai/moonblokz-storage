# Story 3.4: Implement detection and explicit errors for partial/invalid writes

Status: done

## Story

As a MoonBlokz chain runtime,
I want partial or malformed persisted data to be detected on read/startup,
so that recovery decisions can be handled safely by chain logic.

## Acceptance Criteria

1. Given corrupted or partially written flash slot data, when retrieve or startup read-cycle accesses that slot, then backend returns explicit typed errors for invalid/partial data.
2. Backend does not perform chain-policy actions such as pruning or reclaim.

## Tasks / Subtasks

- [x] Add RP2040 test hooks for deterministic partial/malformed slot injection
  - [x] Add internal test helper to write raw slot bytes in mock flash
- [x] Add explicit partial/invalid write detection tests
  - [x] Partial write pattern returns `StorageError::IntegrityFailure`
  - [x] Malformed slot with matching hash still returns `StorageError::IntegrityFailure`
  - [x] Startup-style read cycle over mixed slot states returns typed outcomes (`Ok`, `IntegrityFailure`, `BlockAbsent`)
- [x] Preserve scope boundaries
  - [x] No backend reclaim/pruning behavior added
  - [x] No storage trait expansion

## Developer Context

### Technical Requirements

- Continue implementation in `moonblokz-storage`.
- Keep synchronous API and deterministic mapping unchanged.
- Keep policy ownership in chain logic (backend only reports typed errors).

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/src/backend_rp2040.rs`
- `moonblokz-storage/src/error.rs`
- `moonblokz-storage/README.md`
- `/_bmad-output/implementation-artifacts/3-4-implement-detection-and-explicit-errors-for-partial-invalid-writes.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 3, Story 3.4)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`

## Dev Agent Record

### Completion Notes

- Added RP2040 mock-slot raw write helper for deterministic corruption patterns in tests.
- Added retrieval-path tests for:
  - partial slot bytes -> `IntegrityFailure`
  - malformed slot bytes with matching stored hash -> `IntegrityFailure`
  - startup-style mixed slot states -> typed outcomes preserved
- Confirmed backend remains policy-neutral: no reclaim/pruning added.
- Updated memory-backend error code documentation in both source and README to match current implementation.

### Validation

- `cd moonblokz-storage && cargo test`: pass
- `cd moonblokz-storage && cargo test --no-default-features --features backend-rp2040`: pass

### File List

- `moonblokz-storage/src/backend_rp2040.rs`
- `moonblokz-storage/src/error.rs`
- `moonblokz-storage/README.md`
- `/_bmad-output/implementation-artifacts/sprint-status.yaml`
- `/_bmad-output/implementation-artifacts/3-3-implement-rp2040-retrieve-path-with-mandatory-hash-verification.md`
- `/_bmad-output/implementation-artifacts/3-4-implement-detection-and-explicit-errors-for-partial-invalid-writes.md`
