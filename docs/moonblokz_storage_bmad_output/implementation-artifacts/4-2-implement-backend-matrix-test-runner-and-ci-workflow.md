# Story 4.2: Implement backend-matrix test runner and CI workflow

Status: done

## Story

As a MoonBlokz maintainer,
I want automated backend-matrix test execution aligned with moonblokz-crypto-lib patterns,
so that feature-isolated correctness is continuously verified.

## Acceptance Criteria

1. Given `run_tests.sh` and CI workflow are added, when test automation runs, then each backend is tested with `--no-default-features` and one backend feature enabled.
2. Pipeline fails on feature exclusivity violations or test regressions.

## Tasks / Subtasks

- [x] Add local backend-matrix runner script
  - [x] Test memory backend with `--no-default-features --features backend-memory`
  - [x] Test RP2040 backend with `--no-default-features --features backend-rp2040`
  - [x] Run backend feature exclusivity checks
- [x] Add CI workflow for backend matrix
  - [x] Add GitHub Actions matrix job for backend features
  - [x] Add separate job for feature exclusivity check script
  - [x] Use stable toolchain
- [x] Validate runner locally

## Developer Context

### Technical Requirements

- Follow project pattern from `moonblokz-crypto-lib/run_tests.sh` for script-first workflow.
- Keep backend tests feature-isolated (`--no-default-features` + exactly one backend feature).
- Keep CI deterministic and fail-fast on regression.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/run_tests.sh`
- `moonblokz-storage/.github/workflows/rust.yml`
- `/_bmad-output/implementation-artifacts/4-2-implement-backend-matrix-test-runner-and-ci-workflow.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 4, Story 4.2)
- `moonblokz-crypto-lib/run_tests.sh`

## Dev Agent Record

### Completion Notes

- Added `run_tests.sh` in `moonblokz-storage`:
  - runs memory backend tests with `--no-default-features --features backend-memory`
  - runs RP2040 backend tests with `--no-default-features --features backend-rp2040`
  - runs `./scripts/check_backend_features.sh`
- Added GitHub Actions workflow at `.github/workflows/rust.yml`:
  - `backend-matrix` job with feature matrix (`backend-memory`, `backend-rp2040`)
  - `feature-exclusivity` job to run `./scripts/check_backend_features.sh`

### Validation

- `cd moonblokz-storage && ./run_tests.sh`: pass

### File List

- `moonblokz-storage/run_tests.sh`
- `moonblokz-storage/.github/workflows/rust.yml`
- `/_bmad-output/implementation-artifacts/sprint-status.yaml`
- `/_bmad-output/implementation-artifacts/4-1-implement-backend-conformance-test-modules.md`
- `/_bmad-output/implementation-artifacts/4-2-implement-backend-matrix-test-runner-and-ci-workflow.md`
