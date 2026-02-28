# Epic 4 Retrospective

Status: done

## Scope Completed

- Story 4.1: backend conformance test modules
- Story 4.2: backend-matrix test runner + CI workflow
- Story 4.3: integration/distribution metadata for Git + future crates.io
- Story 4.4: mandatory API/source documentation coverage
- Story 4.5: add-new-device-support implementation guide

## What Went Well

- Feature-isolated backend testing is now reproducible locally (`run_tests.sh`) and in CI (`.github/workflows/rust.yml`).
- Shared conformance scenarios reduced backend semantic drift risk.
- Documentation coverage and onboarding quality improved with explicit API docs and backend extension guide.
- Integration/distribution metadata now aligns with current Git workflow and future crates.io plan.

## Issues Encountered

- Version-invariant enforcement (`version != 0`) required follow-up updates in tests and rustdoc examples.
- Early metadata/docs used incorrect repository owner URLs and needed correction.
- Conformance constructor had a target-conditional compile gap for RP2040 test mode and was fixed.

## Mitigations Applied

- Added explicit tests for version-invariant rejection in `moonblokz-chain-types`.
- Updated all affected examples/tests to set non-zero version in block bytes.
- Corrected repository URLs in crate metadata and README dependency examples.
- Switched RP2040 conformance constructor path to `new_for_tests(0)` and removed bounded invalid-index probing.

## Remaining Risks

- RP2040 behavior is still primarily validated via host/mock path; real-device flash behavior validation remains a separate hardware phase.
- Future backend additions must follow conformance + CI checklist to avoid contract drift.

## Recommended Next Actions

1. Run on-device RP2040 validation for flash read/erase/write edge cases.
2. Add CI branch protection requiring backend-matrix and exclusivity jobs.
3. Start release hardening checklist before first crates.io publication.
