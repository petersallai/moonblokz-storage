# Epic 2 Retrospective

Status: done

## Scope Completed

- Story 2.1: in-memory backend lifecycle and capacity model
- Story 2.2: deterministic storage-index mapping and boundary checks
- Story 2.3: save/retrieve flows with slot-state outcomes
- Story 2.4: startup reconstruction read-cycle support
- Story 2.5: ingest/query integration tests on memory backend

## What Went Well

- In-memory backend contract behavior is deterministic and well-covered.
- Startup-style scan semantics are explicit and stable.
- Typed error outcomes (`BlockAbsent`, `InvalidIndex`) are consistently enforced.
- Integration-style tests captured both positive and negative flow behavior.

## Issues Encountered

- Capacity semantics evolved from block-count to total-storage-bytes.
- Empty-slot marker and version invariant alignment required follow-up adjustments.
- Some tests/docs initially assumed outdated block-version behavior.

## Mitigations Applied

- Unified const-generic parameter semantics (`STORAGE_SIZE`) across backends.
- Switched empty-slot interpretation to first-byte/version marker policy.
- Updated tests and documentation after enforcing `version != 0` invariant.

## Remaining Risks

- Memory backend is a functional test backend, not a hardware-accurate model.
- Future block-structure changes require synchronized updates across storage tests/docs.

## Recommended Next Actions

1. Keep conformance tests as mandatory gate for future backend changes.
2. Track memory-backend assumptions explicitly when adding new protocol fields.
3. Validate new semantics first in memory backend, then in RP2040 backend.
