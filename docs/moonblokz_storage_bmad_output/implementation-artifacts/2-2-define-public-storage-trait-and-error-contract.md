# Story 2.2: Define Public Storage Trait and Error Contract

Status: done

## Story

As a MoonBlokz chain runtime maintainer,  
I want a synchronous `no_std` storage trait and typed error surface,  
so that startup/ingest/query flows have deterministic contracts.

## Acceptance Criteria

1. Given FR1-FR5 and FR21-FR35 contracts, when the public storage trait and errors are defined, then lifecycle, indexed IO, and integrity/result semantics are represented explicitly.
2. Given architecture boundary rules, when storage APIs are consumed, then no chain-policy responsibilities are encoded in storage API behavior.

## Tasks / Subtasks

- [x] Define core public API contract
  - [x] Expose synchronous `StorageTrait` in `no_std`
  - [x] Include lifecycle function (`init`) and indexed block IO (`save_block`, `read_block`)
  - [x] Use canonical `StorageIndex` alias type
- [x] Define typed error model
  - [x] Expose `StorageError` with explicit categories
  - [x] Include invalid index, block absent, integrity failure, backend IO categories
  - [x] Keep error payload footprint minimal
- [x] Ensure boundary cleanliness
  - [x] Keep API free from chain-policy logic
  - [x] Keep contract backend-agnostic at trait/error level
- [x] Validate documentation quality
  - [x] Public trait and functions include parameter docs and examples
  - [x] Error model is documented for integrators
- [x] Execute verification
  - [x] Run `cargo check` in `moonblokz-storage`
  - [x] Run `cargo test` in `moonblokz-storage`

## Developer Context

### Technical Requirements

- `moonblokz-storage` remains `#![no_std]`.
- APIs remain synchronous by design.
- Contract must remain deterministic and embedded-friendly.

### Architecture Compliance

- Storage owns persistence and integrity contract semantics only.
- Chain-level policy decisions remain outside storage trait behavior.
- Trait/error contract must be reusable across multiple backend implementations.

### File Structure Requirements

- `moonblokz-storage/src/lib.rs`
- `moonblokz-storage/src/error.rs`
- `moonblokz-storage/src/types.rs`

### Testing Requirements

- `cargo check` and `cargo test` must pass.
- Doc tests should validate public API examples.

## Project Context Reference

- `/_bmad-output/planning-artifacts/epics.md` (Story 2.2)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- Created story artifact with refreshed key `2-2-define-public-storage-trait-and-error-contract`.
- Verified existing trait/error/types implementation satisfies Story 2.2 acceptance criteria.
- Executed `cargo check` and `cargo test` in `moonblokz-storage`.

### Completion Notes List

- Existing implementation already satisfies Story 2.2 scope.
- Story is ready for formal code review.

### File List

- `/_bmad-output/implementation-artifacts/2-2-define-public-storage-trait-and-error-contract.md`
- `moonblokz-storage/src/lib.rs`
- `moonblokz-storage/src/error.rs`
- `moonblokz-storage/src/types.rs`

## Change Log

- 2026-02-28: Created Story 2.2 context and validated current implementation against ACs; set status to `review`.
- 2026-02-28: Code review completed with no findings; story marked `done`.
