# Epic 1 Retrospective

Status: done

## Scope Completed

- Story 1.1: immutable `Block` and `BlockBuilder` in `moonblokz-chain-types`
- Story 1.2: canonical serialization/hash boundary contracts
- Story 1.3: `no_std` synchronous storage trait and core error model
- Story 1.4: backend feature exclusivity and backend module isolation

## What Went Well

- Core contracts were established early and kept stable through later epics.
- Feature exclusivity prevented ambiguous backend selection paths.
- Hash and serialization boundaries were simplified for embedded constraints.
- `no_std` API shape remained lean and compatible with constrained targets.

## Issues Encountered

- Initial contract wording and examples needed iterative alignment with evolving design decisions.
- Version and empty-slot semantics required explicit policy clarification across crates.

## Mitigations Applied

- Consolidated canonical APIs and removed redundant abstraction layers.
- Added explicit documentation and tests for contract invariants.
- Enforced compile-time backend selection rules and validated them continuously.

## Remaining Risks

- Any future block-format changes require coordinated updates across chain-types and storage crates.
- Embedded constraints still require careful review for code-size/memory regressions.

## Recommended Next Actions

1. Keep contract tests and conformance tests as mandatory change gates.
2. Require explicit compatibility notes for any block/hash contract changes.
3. Continue favoring simple, deterministic APIs for embedded targets.
