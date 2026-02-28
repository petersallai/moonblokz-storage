# Story 2.8: Add Backend Matrix Test Runner

Status: done

## Story

As a maintainer,  
I want backend-by-backend test execution aligned with `moonblokz-crypto-lib` style,  
so that feature-isolated behavior is continuously verified.

## Acceptance Criteria

1. Given multiple backend features, when matrix test script runs with `--no-default-features` and one backend at a time, then each backend test set executes successfully in isolation.
2. Given backend-specific failures, when matrix script is used, then failures are clearly attributable to the active backend run.

## Tasks / Subtasks

- [x] Add dedicated backend matrix script
  - [x] Run memory backend tests with feature-isolated command
  - [x] Run rp2040 backend tests with feature-isolated command
- [x] Integrate matrix runner into test entrypoint
  - [x] `run_tests.sh` delegates backend runs to matrix script
  - [x] Existing feature-exclusivity checks remain included
- [x] Validate execution
  - [x] `./run_tests.sh` passes locally with matrix runner

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- Added `scripts/run_backend_matrix.sh` with one-feature-per-run commands.
- Updated `run_tests.sh` to call matrix script, then feature exclusivity guard script.
- Verified matrix execution and full test script pass.

### File List

- `moonblokz-storage/scripts/run_backend_matrix.sh`
- `moonblokz-storage/run_tests.sh`
- `/_bmad-output/implementation-artifacts/2-8-add-backend-matrix-test-runner.md`

## Change Log

- 2026-02-28: Implemented backend matrix runner and integrated it into repository test entrypoint; story set to `review`.
- 2026-02-28: Code review completed with no findings; story marked `done`.
