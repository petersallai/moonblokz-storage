# Story 2.1: Initialize Storage Crate with Feature Exclusivity Guards

Status: done

## Story

As a MoonBlokz chain developer,  
I want `moonblokz-storage` initialized with compile-time backend feature exclusivity,  
so that exactly one backend is active per build.

## Acceptance Criteria

1. Given storage crate feature configuration, when zero or multiple backend features are enabled, then compilation fails with explicit error messaging.
2. Given storage crate feature configuration, when exactly one backend feature is enabled, then compilation succeeds.

## Tasks / Subtasks

- [x] Validate feature model in `Cargo.toml`
  - [x] Confirm backend features are explicitly declared
  - [x] Confirm default backend selection exists for developer ergonomics
- [x] Enforce exclusivity in crate root
  - [x] Add compile-time error for zero-feature configuration
  - [x] Add compile-time error for multi-feature configuration
- [x] Prevent conflicting symbol definitions under invalid feature combinations
  - [x] Guard backend alias cfgs to avoid duplicate type-definition errors
  - [x] Keep explicit compile-time exclusivity error as primary failure message
- [x] Execute compile-matrix validation
  - [x] `cargo check --no-default-features` fails with explicit exclusivity message
  - [x] `cargo check --no-default-features --features "backend-memory backend-rp2040"` fails with explicit exclusivity message
  - [x] `cargo check --no-default-features --features backend-memory` succeeds
  - [x] `cargo check --no-default-features --features backend-rp2040` succeeds

## Developer Context

### Technical Requirements

- Keep `moonblokz-storage` as `#![no_std]`.
- Enforce exactly one backend at compile time.
- Maintain deterministic compile diagnostics for invalid feature combos.

### Architecture Compliance

- Story maps to backend-agnostic interface setup and backend isolation baseline.
- This story is compile-contract focused; functional storage behavior belongs to later stories.

### File Structure Requirements

- `moonblokz-storage/Cargo.toml`
- `moonblokz-storage/src/lib.rs`

### Testing Requirements

- Compile-time matrix checks are the primary validation method for this story.

## Project Context Reference

- `/_bmad-output/planning-artifacts/epics.md` (Story 2.1)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- Created story artifact with current key naming from sprint tracker.
- Verified compile behavior across zero/one/many feature combinations.
- Updated alias cfg guards in `src/lib.rs` to remove duplicate type-definition noise under invalid multi-feature builds.

### Completion Notes List

- Story 2.1 ACs are satisfied with explicit compile-time feature exclusivity behavior.
- Compile matrix evidence captured for all required combinations.

### File List

- `/_bmad-output/implementation-artifacts/2-1-initialize-storage-crate-with-feature-exclusivity-guards.md`
- `moonblokz-storage/src/lib.rs`

## Change Log

- 2026-02-28: Implemented and validated compile-time backend feature exclusivity guards; set story to `review`.
- 2026-02-28: Code review completed with no findings; story marked `done`.
