# Story 4.5: Write add-new-device-support guide with backend implementation checklist

Status: done

## Story

As a backend implementer,
I want a concrete guide for adding new device backends,
so that future implementations preserve contract and conformance expectations.

## Acceptance Criteria

1. Given documentation in `docs/add-new-device-support.md`, when a developer follows the guide, then they can add a new backend feature, implement the trait, and wire tests without changing chain-level semantics.
2. Guide includes required conformance, docs, and CI checklist items.

## Tasks / Subtasks

- [x] Create `docs/add-new-device-support.md` in `moonblokz-storage`
  - [x] Include backend feature wiring steps
  - [x] Include backend module + `StorageTrait` implementation steps
  - [x] Include error-model and integrity semantics requirements
- [x] Include verification and quality gates
  - [x] conformance integration requirements (`src/conformance.rs`)
  - [x] CI/matrix integration requirements (`run_tests.sh`, workflow)
  - [x] docs coverage checklist requirements
- [x] Link guide from crate README
  - [x] Add `Developer Guides` section in `moonblokz-storage/README.md`
- [x] Validate no regressions after docs updates

## Developer Context

### Technical Requirements

- Keep guide aligned with current architecture decisions:
  - synchronous API
  - backend-only responsibility (no chain policy)
  - fixed-size slot handling + integrity checks
- Keep guide explicit and executable as a checklist.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/docs/add-new-device-support.md`
- `moonblokz-storage/README.md`
- `/_bmad-output/implementation-artifacts/4-5-write-add-new-device-support-guide-with-backend-implementation-checklist.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 4, Story 4.5)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`

## Dev Agent Record

### Completion Notes

- Added comprehensive backend-implementation guide at
  `moonblokz-storage/docs/add-new-device-support.md`.
- Guide includes:
  - feature flag wiring
  - backend module and alias wiring
  - `StorageTrait` semantic obligations
  - typed error model expectations
  - conformance module integration
  - matrix runner + CI integration steps
  - documentation and final validation checklists
- Added README pointer in `moonblokz-storage/README.md` under `Developer Guides`.

### Validation

- `cd moonblokz-storage && cargo test --no-default-features --features backend-memory`: pass
- `cd moonblokz-storage && cargo test --no-default-features --features backend-rp2040`: pass

### File List

- `moonblokz-storage/docs/add-new-device-support.md`
- `moonblokz-storage/README.md`
- `/_bmad-output/implementation-artifacts/sprint-status.yaml`
- `/_bmad-output/implementation-artifacts/4-4-complete-mandatory-api-and-source-documentation-coverage.md`
- `/_bmad-output/implementation-artifacts/4-5-write-add-new-device-support-guide-with-backend-implementation-checklist.md`
