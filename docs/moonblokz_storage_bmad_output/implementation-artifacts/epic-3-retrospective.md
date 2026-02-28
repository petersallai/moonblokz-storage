# Epic 3 Retrospective

Status: done

## Scope Completed

- Story 3.1: RP2040 flash geometry mapping for block slots
- Story 3.2: synchronous RP2040 save path by storage index
- Story 3.3: retrieve path with mandatory hash verification
- Story 3.4: detection and explicit errors for partial/invalid writes
- Story 3.5: RP2040 integration-style startup/ingest/query tests

## What Went Well

- Deterministic slot/page mapping is stable and test-covered.
- Save/retrieve semantics follow fixed-size slot model with hash metadata verification.
- Typed error handling remained consistent with `StorageTrait` contract.
- Startup/ingest/query integration-style test coverage improved confidence in runtime flows.

## Issues Encountered

- Constructor and const-generic refactors caused temporary test mismatches and required call-site updates.
- Documentation/tests needed updates after enforcing `version != 0` invariant in chain types.
- Cross-target test-compatibility edge cases appeared in feature-gated constructor paths.

## Mitigations Applied

- Aligned backend constructors with const-generic storage-size model.
- Added dedicated tests for partial writes, malformed slots, and integrity mismatch outcomes.
- Updated docs/examples and conformance setup to be test-target safe.
- Ran both backend feature modes continuously during changes.

## Remaining Risks

- Real RP2040 flash behavior (timing, erase/write edge conditions) is still not fully validated on-device.
- Mock-backed tests cannot fully replicate hardware-specific side effects.

## Recommended Next Actions

1. Execute on-device RP2040 validation suite for flash operation edge cases.
2. Add telemetry/log assertions around storage error code paths on target hardware.
3. Keep integration tests synchronized with any future block format/hash-contract changes.
