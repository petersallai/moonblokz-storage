# Story 2.7: Implement Backend Conformance Test Suite

Status: done

## Story

As a backend implementer,  
I want conformance tests that validate shared semantics,  
so that all backend implementations remain behaviorally consistent.

## Acceptance Criteria

1. Given storage trait and at least one backend, when conformance tests run, then index mapping, integrity behavior, and error semantics are verified.
2. Given multiple backends, when conformance tests run per backend feature, then the same semantic contract is validated with reusable tests.

## Tasks / Subtasks

- [x] Provide reusable conformance module
  - [x] Add shared trait-level semantic tests under `src/conformance.rs`
  - [x] Keep tests backend-selected through crate feature configuration
- [x] Validate core contract behavior in conformance tests
  - [x] save/read round-trip behavior
  - [x] empty-slot behavior
  - [x] invalid-index behavior
  - [x] startup scan typed outcomes for mixed slots
- [x] Ensure conformance suite runs on backend matrix
  - [x] memory backend run validates conformance tests
  - [x] rp2040 backend run validates conformance tests

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- Conformance tests are implemented in `moonblokz-storage/src/conformance.rs`.
- Tests instantiate backend by active feature and run shared semantic assertions.
- Verified passing runs with both:
  - `cargo test --no-default-features --features backend-memory`
  - `cargo test --no-default-features --features backend-rp2040`

### File List

- `moonblokz-storage/src/conformance.rs`
- `/_bmad-output/implementation-artifacts/2-7-implement-backend-conformance-test-suite.md`

## Change Log

- 2026-02-28: Story artifact created and marked `review`; conformance suite already implemented and passing on both backends.
- 2026-02-28: Code review completed with no findings; story marked `done`.
