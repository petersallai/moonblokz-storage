# Story 3.5: Add RP2040 integration tests for startup/ingest/query flows

Status: done

## Story

As a MoonBlokz storage maintainer,
I want RP2040-focused integration tests covering startup, ingest, and query semantics,
so that MVP behavior is validated on the target backend.

## Acceptance Criteria

1. Given RP2040 backend tests are executed, when startup read-cycle, ingest writes, and query reads run with valid and corrupted data sets, then behavior matches documented deterministic and integrity requirements.
2. Error categories remain consistent with the storage trait contract.

## Tasks / Subtasks

- [x] Add integration-style valid-dataset flow test for RP2040 backend
  - [x] Cover ingest writes
  - [x] Cover startup read-cycle typed outcomes
  - [x] Cover query reads returning stored blocks
- [x] Add integration-style corrupted-dataset flow test for RP2040 backend
  - [x] Include hash-corrupted stored slot
  - [x] Include partial slot bytes
  - [x] Assert typed outcomes for startup/query (`IntegrityFailure`, `BlockAbsent`, `InvalidIndex`)
- [x] Validate feature matrix remains stable
  - [x] `cargo test` (default memory backend)
  - [x] `cargo test --no-default-features --features backend-rp2040`
  - [x] backend feature exclusivity check script

## Developer Context

### Technical Requirements

- Continue implementation in `moonblokz-storage`.
- Keep synchronous API model and current RP2040 mapping/hash behavior unchanged.
- Integration tests should stay deterministic and avoid timing dependencies.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/src/backend_rp2040.rs`
- `/_bmad-output/implementation-artifacts/3-5-add-rp2040-integration-tests-for-startup-ingest-query-flows.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 3, Story 3.5)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`

## Dev Agent Record

### Completion Notes

- Added two RP2040 integration-style tests in `backend_rp2040.rs`:
  - `integration_startup_ingest_query_flow_with_valid_dataset`
  - `integration_startup_and_query_flow_reports_integrity_on_corrupted_dataset`
- Valid-dataset test covers:
  - ingest writes at deterministic indices
  - startup scan across populated and empty slots with typed outcomes
  - query retrieval of exact stored blocks
- Corrupted-dataset test covers:
  - hash-corrupted slot -> `IntegrityFailure`
  - partial slot bytes -> `IntegrityFailure`
  - empty slot -> `BlockAbsent`
  - out-of-range query -> `InvalidIndex`

### Validation

- `cd moonblokz-storage && cargo test`: pass
- `cd moonblokz-storage && cargo test --no-default-features --features backend-rp2040`: pass
- `cd moonblokz-storage && ./scripts/check_backend_features.sh`: pass

### File List

- `moonblokz-storage/src/backend_rp2040.rs`
- `/_bmad-output/implementation-artifacts/sprint-status.yaml`
- `/_bmad-output/implementation-artifacts/3-4-implement-detection-and-explicit-errors-for-partial-invalid-writes.md`
- `/_bmad-output/implementation-artifacts/3-5-add-rp2040-integration-tests-for-startup-ingest-query-flows.md`
