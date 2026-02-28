# Story 2.6: Implement Integrity Verification and Invalid Data Rejection

Status: done

## Story

As a MoonBlokz node operator,  
I want retrieval to verify integrity before returning block data,  
so that corrupted/partial data is never silently accepted.

## Acceptance Criteria

1. Given persisted block and hash metadata, when block retrieval occurs, then storage recomputes hash and compares expected values.
2. Given integrity mismatch or invalid artifacts, when retrieval occurs, then explicit errors are returned and no block payload is returned.

## Tasks / Subtasks

- [x] Implement retrieval-time integrity verification in backend(s) with hash metadata
  - [x] Recompute hash from stored block bytes
  - [x] Compare against stored hash metadata before returning block
- [x] Enforce invalid-data rejection
  - [x] Return explicit `StorageError::IntegrityFailure` on mismatch/invalid artifacts
  - [x] Never return corrupted block payload
- [x] Validate with deterministic tests
  - [x] positive path returns exact block
  - [x] corruption/partial-write style path returns integrity error

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- Integrity verification exists in RP2040 backend decode path (`decode_slot_block`) with SHA-256 check.
- Retrieval path returns `StorageError::IntegrityFailure` on hash mismatch or block parse failure.
- Existing RP2040 tests cover valid round-trip, corruption detection, and partial-write/invalid artifact detection.

### File List

- `moonblokz-storage/src/backend_rp2040.rs`
- `/_bmad-output/implementation-artifacts/2-6-implement-integrity-verification-and-invalid-data-rejection.md`

## Change Log

- 2026-02-28: Story artifact created; current implementation already satisfies integrity verification and explicit rejection semantics.
- 2026-02-28: Code review completed with no findings; story marked `done`.
