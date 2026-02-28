# Story 1.3: Implement Canonical SHA-256 Hash Utility

Status: done

## Story

As a storage backend implementer,  
I want a shared `calculate_hash(&[u8]) -> [u8; HASH_SIZE]` function in chain-types,  
so that integrity checks are deterministic across all backends.

## Acceptance Criteria

1. Given canonical hashing requirements in architecture/PRD, when hash utility is added, then `HASH_SIZE` and SHA-256 behavior are defined in chain-types.
2. Given storage and test integration needs, when hash utility is consumed, then contracts are reusable without duplicate hash implementations.

## Tasks / Subtasks

- [x] Define canonical hash contract
  - [x] Expose `HASH_SIZE` in `moonblokz-chain-types`
  - [x] Expose `calculate_hash(input: &[u8]) -> [u8; HASH_SIZE]`
- [x] Implement SHA-256 behavior
  - [x] Use deterministic SHA-256 implementation suitable for `no_std`
  - [x] Ensure function returns fixed-size array output
- [x] Validate correctness
  - [x] Add deterministic known-vector tests (including empty input and `abc`)
  - [x] Confirm tests pass on stable toolchain
- [x] Validate boundary reuse
  - [x] Confirm hash utility is exported via crate `lib.rs`
  - [x] Confirm no duplicate hash wrapper/type contract is required

## Developer Context

### Technical Requirements

- Scope this story to `moonblokz-chain-types`.
- Preserve `#![no_std]` compatibility.
- Keep the implementation simple and deterministic for embedded constraints.

### Architecture Compliance

- Canonical hashing boundary is owned by `moonblokz-chain-types`.
- `moonblokz-storage` and tests consume this API directly.
- Avoid extra wrapper abstractions that increase code size.

### File Structure Requirements

- `moonblokz-chain-types/src/hash.rs`
- `moonblokz-chain-types/src/lib.rs`
- `moonblokz-chain-types/Cargo.toml` (dependency wiring only if needed)

### Testing Requirements

- Run `cargo check` and `cargo test` in `moonblokz-chain-types`.
- Keep tests deterministic and fixed-input based.

## Project Context Reference

- `/_bmad-output/planning-artifacts/epics.md` (Story 1.3)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- Story context created from current key: `1-3-implement-canonical-sha-256-hash-utility`.
- Verified existing hash implementation and export surface already satisfy Story 1.3 ACs.
- Executed `cargo check` and `cargo test` in `moonblokz-chain-types`.

### Completion Notes List

- Existing implementation already satisfies Story 1.3.
- Story is ready for formal code review.

### File List

- `/_bmad-output/implementation-artifacts/1-3-implement-canonical-sha-256-hash-utility.md`
- `moonblokz-chain-types/src/hash.rs`
- `moonblokz-chain-types/src/lib.rs`

## Change Log

- 2026-02-28: Created Story 1.3 context and validated current implementation against ACs; set status to `review`.
- 2026-02-28: Code review completed with no findings; story marked `done`.
