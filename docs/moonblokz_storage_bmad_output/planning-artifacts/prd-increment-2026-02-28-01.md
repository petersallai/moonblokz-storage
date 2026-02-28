# PRD Increment 01 (Completed)

- Date: 2026-02-28
- Baseline PRD: `_bmad-output/planning-artifacts/prd.md`
- Status: Completed (Implemented and validated)

## Objective

Add two runnable `moonblokz-storage` example projects (`std` and RP2040 embedded) that demonstrate the canonical storage lifecycle in a minimal, deterministic flow.

## Scope

- In scope:
  - Add a `std` example project under `moonblokz-storage/examples/` showing end-to-end storage usage.
  - Add an RP2040 embedded example project under `moonblokz-storage/examples/` following the setup pattern used in `moonblokz-radio-lib` embedded example.
  - Use the same high-level runtime behavior in both examples:
    - Check whether storage is initialized.
    - If not initialized: call `init(private_key, own_node_id, init_params)`.
    - Then attempt block write and block read.
  - Embedded UX requirement:
    - If write+read succeeds: blink LED once for 0.5 seconds.
    - If write or read fails: blink LED three times.
  - Add build/run instructions for both examples in `moonblokz-storage/README.md`.
- Out of scope:
  - New storage backends.
  - Chain-level policy features (pruning, consensus, payload parsing).
  - Async storage API redesign.
  - Cryptographic behavior changes.

## Problem Statement

Current baseline PRD and implemented repositories cover:
- RP2040 storage backend MVP
- Control-plane initialization/load/set-chain-config flow
- Chain-types immutable block and hash utility
- CI/documentation/release-process baseline

This increment adds executable onboarding examples so developers can validate integration quickly in host and RP2040 environments.

## Functional Requirements Delta

- FR-INC-1: `moonblokz-storage` can provide a `std` example demonstrating initialization detection, init path, block save, and block read using current public API.
- FR-INC-2: `moonblokz-storage` can provide an RP2040 embedded example demonstrating the same lifecycle using Embassy/RP2040 setup conventions aligned with `moonblokz-radio-lib`.
- FR-INC-3: Embedded example can signal operation result through LED behavior (single 0.5s blink on success, three blinks on failure).
- FR-INC-4: Documentation can provide explicit commands to build/run the examples.

## Non-Functional Requirements Delta

- NFR-INC-1: Maintain embedded constraints (`no_std`, bounded memory)
- NFR-INC-2: Maintain deterministic behavior and typed error contracts
- NFR-INC-3: Maintain CI gates and documentation quality baseline
- NFR-INC-4: Examples should remain minimal and avoid unnecessary binary-size overhead.

## API / Data Model Impact

- `moonblokz-storage`:
  - Add two example projects and any required example-specific `Cargo.toml` configuration.
  - Reuse current `StorageTrait` APIs; no required public API expansion for this increment.
- `moonblokz-chain-types`:
  - No API/data-model change required.

## Architecture Impact

- Required architecture updates:
  - Document example runtime flow and RP2040 hardware signaling behavior.
  - Reference `moonblokz-radio-lib` example structure for embedded project scaffolding.
- Backward compatibility expectations:
  - No breaking API changes in `moonblokz-storage` or `moonblokz-chain-types`.

## Stories Impact (Epics Mapping)

- New/updated stories:
  - Story INC-1: Implement `std` example project.
  - Story INC-2: Implement RP2040 embedded example project.
  - Story INC-3: Document example usage in README/docs.
- Dependencies and sequencing:
  - INC-1 before INC-2 is preferred for faster feedback.
  - INC-3 after both implementations.

## Validation Plan

- Unit tests:
  - Not mandatory for example-only logic beyond existing library tests.
- Integration/conformance tests:
  - Existing library conformance suite remains required to pass.
  - Example build checks should be added for host and embedded targets where feasible.
- Documentation updates:
  - Required (`README`, `docs`, changelog).

## Implementation Result

- Delivered `std` example project:
  - `moonblokz-storage/examples/moonblokz-storage-std-example`
- Delivered RP2040 embedded example project:
  - `moonblokz-storage/examples/moonblokz-storage-embedded-example`
- Delivered README usage commands and behavior documentation.
- Embedded signaling behavior implemented:
  - success: one 0.5s blink
  - failure: three blinks

## Validation Result

- `moonblokz-storage/./run_tests.sh`: pass
- `examples/moonblokz-storage-std-example` `cargo run`: pass
- `examples/moonblokz-storage-embedded-example` `cargo build --target thumbv6m-none-eabi`: pass

## Acceptance Criteria

1. `std` example compiles and demonstrates: uninitialized detection -> init (if needed) -> save block -> read block.
2. RP2040 embedded example compiles for `thumbv6m-none-eabi` and demonstrates identical logical flow.
3. Embedded LED behavior matches contract:
   - success path: one blink, 0.5s
   - failure path: three blinks
4. `moonblokz-storage` README includes build/run instructions for both examples.
5. Existing storage test matrix remains green.

## Decision Log

- 2026-02-28: Increment opened from published baseline.
- 2026-02-28: Scope approved to add `std` + RP2040 embedded examples for storage lifecycle demonstration, with LED signaling on embedded result.
- 2026-02-28: Increment implemented and validated; stories 4.5/4.6/4.7 completed.
