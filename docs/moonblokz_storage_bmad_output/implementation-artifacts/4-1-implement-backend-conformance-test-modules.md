# Story 4.1: Implement backend conformance test modules

Status: done

## Story

As a backend implementer,
I want conformance tests that run against each backend feature,
so that semantic parity is enforced across implementations.

## Acceptance Criteria

1. Given conformance modules are defined, when tests run against memory and RP2040 backends separately, then both backends pass the same scenarios for index mapping, integrity, and error semantics.
2. Conformance failures identify backend-specific deviations clearly.

## Tasks / Subtasks

- [x] Add shared conformance module in `moonblokz-storage`
  - [x] Add feature-gated backend constructors for memory and RP2040
  - [x] Keep conformance tests backend-agnostic after construction
- [x] Add conformance scenarios
  - [x] Save/read round-trip returns exact saved block
  - [x] Empty slot returns `BlockAbsent`
  - [x] Invalid index returns `InvalidIndex` for read and save
  - [x] Startup-style mixed slot scan preserves typed outcomes
- [x] Wire conformance module into crate test compilation
- [x] Validate both backend modes

## Developer Context

### Technical Requirements

- Keep tests deterministic and synchronous.
- Use public storage contract (`StorageTrait`, `StorageError`, `MoonblokzStorage`) for conformance checks.
- Avoid backend-specific mutation hooks in conformance module.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/src/conformance.rs`
- `moonblokz-storage/src/lib.rs`
- `/_bmad-output/implementation-artifacts/4-1-implement-backend-conformance-test-modules.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 4, Story 4.1)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`

## Dev Agent Record

### Completion Notes

- Added `src/conformance.rs` with shared conformance tests that compile and run under each backend feature selection.
- Implemented feature-gated `new_backend()` for:
  - `backend-memory`: `MoonblokzStorage::<STORAGE_SIZE>::new()`
  - `backend-rp2040`: `MoonblokzStorage::<STORAGE_SIZE>::new(0)`
- Added tests covering round-trip integrity, absent-slot behavior, invalid-index behavior, and startup-like mixed-slot outcomes.
- Added `#[cfg(test)] mod conformance;` in `src/lib.rs` so tests are included in crate test runs.

### Validation

- `cd moonblokz-storage && cargo test`: pass
- `cd moonblokz-storage && cargo test --no-default-features --features backend-rp2040`: pass

### File List

- `moonblokz-storage/src/conformance.rs`
- `moonblokz-storage/src/lib.rs`
- `/_bmad-output/implementation-artifacts/sprint-status.yaml`
- `/_bmad-output/implementation-artifacts/3-5-add-rp2040-integration-tests-for-startup-ingest-query-flows.md`
- `/_bmad-output/implementation-artifacts/4-1-implement-backend-conformance-test-modules.md`
