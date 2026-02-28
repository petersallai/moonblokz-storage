# Story 1.2: Define chain-types block-byte and hasher boundary contracts

Status: done

## Story

As a MoonBlokz storage implementer,
I want canonical block-byte and hash-function contracts in chain-types,
so that storage backends can consume consistent block/hash semantics.

## Acceptance Criteria

1. Given the Part V block format requirements, when block-byte and hash boundary interfaces are defined in `moonblokz-chain-types`, then little-endian layout, header fields, and payload handling contracts are documented and test-covered.
2. Storage-facing consumers can import and use canonical block contract and hashing API from `moonblokz-chain-types` without redefining semantics.
3. API and module boundaries remain `no_std`, deterministic, and aligned with architecture ownership rules.
4. Binary-size discipline and simplicity-first implementation are preserved (no unnecessary derives/default implementations, helper layers, or heavyweight abstractions).

## Tasks / Subtasks

- [x] Use `Block` internal bytes as canonical serialization boundary in `moonblokz-chain-types`
  - [x] Remove standalone `src/serialization.rs` module
  - [x] Expose canonical serialized slice accessor from `Block` (`serialized_bytes`)
  - [x] Keep little-endian handling in `block.rs` as single source of truth for header parsing
- [x] Replace hash type boundary with generic hasher functionality
  - [x] Define `HASH_SIZE` constant in `src/hash.rs`
  - [x] Implement `calculate_hash(input: &[u8]) -> [u8; HASH_SIZE]` using SHA-256
  - [x] Remove `BlockHash` wrapper type and size-validation constructor APIs
- [x] Wire public exports and boundary surface
  - [x] Update `src/lib.rs` exports to remove serialization-module APIs
  - [x] Update `src/lib.rs` exports to expose `calculate_hash` and `HASH_SIZE`
  - [x] Preserve crate boundary so storage crates import canonical contracts from chain-types only
- [x] Enforce simplicity-first implementation discipline
  - [x] Prefer direct, minimal logic paths for embedded constraints
  - [x] Avoid unnecessary abstractions that increase code size or runtime overhead
- [x] Keep deterministic tests for block/hash boundary contracts
  - [x] Validate `Block::from_bytes` + `Block::as_bytes`/`Block::serialized_bytes` behavior
  - [x] Validate SHA-256 output with known vectors
  - [x] Keep malformed/invalid input coverage for boundary entry points
- [x] Keep mandatory documentation
  - [x] Module-level block comments for modules
  - [x] Public function/struct/field docs with parameter descriptions
  - [x] At least one usage example for each public function

## Developer Context

### Technical Requirements

- Continue implementation in `moonblokz-chain-types` only.
- `Block` byte representation plus `calculate_hash`/`HASH_SIZE` are the canonical downstream storage contracts.
- Preserve `#![no_std]` compatibility and deterministic behavior.
- Maintain fixed-size, explicit byte-level handling compatible with existing `Block` format contracts.
- Keep implementation as simple as possible for embedded CPU/RAM/flash limits.

### Architecture Compliance

- Ownership boundary is strict:
  - `moonblokz-chain-types` owns canonical block types, `Block` byte contract, and hash-function contract.
  - `moonblokz-storage` consumes these contracts and must not redefine semantics.
- Any `Block` binary layout change or hash-function contract change is a cross-crate compatibility event; keep this story explicit.
- Respect enforced implementation rules:
  - binary-size discipline
  - canonical naming
  - no backend logic in chain-types
  - `log` crate as canonical runtime logging mechanism for future runtime code (if logging is added)

### Library/Framework Requirements

- Use Rust stable and `no_std` patterns.
- Avoid introducing new dependencies unless strictly necessary; if needed, require explicit approval.
- Keep public API concise and embedded-friendly.

### File Structure Requirements

Target files for this story:

- `moonblokz-chain-types/src/lib.rs`
- `moonblokz-chain-types/src/hash.rs`
- `moonblokz-chain-types/src/block.rs`
- `moonblokz-chain-types/src/error.rs` (only if error-surface cleanup is required)

Do not modify `moonblokz-storage` in this story.

### Testing Requirements

- Add deterministic unit tests in chain-types modules.
- Required validation focus:
  - block-byte boundary behavior (`from_bytes` and byte-slice accessors)
  - little-endian correctness for header fields
  - malformed input handling
  - SHA-256 hashing contract integrity
- Run full crate test suite to prevent regressions from Story 1.1.

## Previous Story Intelligence

From Story 1.1 (`done`):

- Immutable block model with internal `[u8; MAX_BLOCK_SIZE]` + `len` is established.
- Header parsing and byte offsets are already implemented and tested.
- Structural semantic validation beyond size/min-header was intentionally deferred.
- Binary-size discipline has been explicitly enforced:
  - removed nonessential derives/default implementations
  - avoided panic-prone parsing helpers in runtime logic

Use these as baseline patterns; do not reintroduce removed overhead.

## Git Intelligence Summary

- Story 1.1 is complete and reviewed as `done`.
- Current next story key in sprint tracking is `1-2-define-chain-types-serialization-and-hash-boundary-contracts`.
- No conflicting in-progress changes are required for this story setup phase.

## Latest Tech Information

No external API/version research is required for this story. This work is internal contract definition within current architecture constraints.

## Project Context Reference

No `project-context.md` file detected. Source documents for this story:

- `/_bmad-output/planning-artifacts/epics.md`
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`
- `/_bmad-output/implementation-artifacts/1-1-create-immutable-block-and-blockbuilder-in-moonblokz-chain-types.md`

## Dev Notes

- Canonical block-byte contract in chain-types is `Block` internal serialized bytes.
- Canonical hash contract in chain-types is `calculate_hash` over `&[u8]` returning `[u8; HASH_SIZE]` (SHA-256).
- Keep little-endian rules explicit for all multi-byte fields in `block.rs`.
- Keep implementation lean for embedded targets.

### Project Structure Notes

- Repositories in active use:
  - `moonblokz-chain-types`
  - `moonblokz-storage`
- This story modifies only `moonblokz-chain-types`.

### References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 1, Story 1.2)
- `/_bmad-output/planning-artifacts/architecture.md` (Ownership Boundaries, Block-byte Contract, Hashing Contract, Binary Size Discipline Pattern, Logging Pattern)
- `/_bmad-output/planning-artifacts/prd.md` (FR26, FR27, FR33-FR36, NFR11, NFR13, NFR14)
- `/_bmad-output/implementation-artifacts/1-1-create-immutable-block-and-blockbuilder-in-moonblokz-chain-types.md`

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- Story selected from sprint status first backlog key: `1-2-define-chain-types-serialization-and-hash-boundary-contracts`
- Sprint status updated for story key: `ready-for-dev` -> `in-progress`
- Context loaded from epics, architecture, PRD, and previous Story 1.1 implementation record
- Switched canonical serialization boundary to `Block` byte representation
- Replaced hash wrapper type boundary with SHA-256 hasher function boundary
- Executed full `cargo fmt && cargo test` in `moonblokz-chain-types` (unit + doc tests pass)

### Completion Notes List

- Story context updated to block-byte/hasher boundary implementation.
- Includes architecture guardrails for `no_std`, ownership boundaries, logging standard, and binary-size discipline.
- Removed `src/serialization.rs`; serialization is represented by `Block` bytes in `src/block.rs`.
- Added explicit `Block::serialized_bytes()` accessor for canonical storage boundary usage.
- Replaced previous hash wrapper type with canonical hashing utility in `src/hash.rs`:
  - `HASH_SIZE`
  - `calculate_hash`
- Simplified `BlockError` by removing serialization/hash-wrapper-only variants no longer needed.

### File List

- `moonblokz-chain-types/src/block.rs`
- `moonblokz-chain-types/src/hash.rs`
- `moonblokz-chain-types/src/error.rs`
- `moonblokz-chain-types/src/lib.rs`
- `moonblokz-chain-types/Cargo.toml`
- `/_bmad-output/implementation-artifacts/1-2-define-chain-types-serialization-and-hash-boundary-contracts.md`
- `/_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-02-26: Updated Story 1.2 to use `Block` serialized bytes as canonical boundary and `calculate_hash`/`HASH_SIZE` as canonical SHA-256 contract; removed `serialization.rs` and `BlockHash` wrapper APIs; status remains `review`.
