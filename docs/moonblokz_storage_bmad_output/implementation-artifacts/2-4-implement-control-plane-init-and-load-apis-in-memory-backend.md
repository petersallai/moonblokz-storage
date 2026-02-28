# Story 2.4: Implement Control-Plane Init and Load APIs in Memory Backend

Status: done

## Story

As a MoonBlokz chain runtime maintainer,  
I want memory-backend support for `init`, `load_control_data`, and control data schema persistence,  
so that control-plane lifecycle flows can be tested before hardware integration.

## Acceptance Criteria

1. Given control-plane FR6-FR20 requirements, when memory backend control-plane APIs are implemented, then init erase semantics, immutable-until-reinit behavior, and control-data loading are enforced.
2. Given uninitialized storage state, when control data is loaded, then explicit typed error is returned.

## Tasks / Subtasks

- [x] Extend public storage trait with control-plane API
  - [x] `init(private_key, own_node_id, init_params)` signature added
  - [x] `load_control_data()` added
  - [x] `set_chain_configuration(Block)` added
- [x] Define control-plane constants and return type
  - [x] `INIT_PARAMS_SIZE = 100`
  - [x] `CONTROL_PLANE_COUNT = 3`
  - [x] `CONTROL_PLANE_VERSION` constant
  - [x] `ControlPlaneData` return struct
- [x] Implement memory backend control-plane persistence
  - [x] `init` clears block storage and rewrites all control-plane replicas
  - [x] persisted schema includes version, size fields, max block size, chain config reserve, and CRC32
  - [x] `load_control_data` validates CRC and size compatibility
  - [x] load path performs best-effort repair for invalid replicas after first valid replica is found
  - [x] `set_chain_configuration` is set-once unless re-init
- [x] Extend error model for control-plane outcomes
  - [x] `ControlPlaneUninitialized`
  - [x] `ChainConfigurationAlreadySet`
  - [x] `ControlPlaneCorrupted`
  - [x] `ControlPlaneIncompatible`
- [x] Add deterministic tests
  - [x] uninitialized load returns explicit error
  - [x] init stores control data and clears block slots
  - [x] set-once chain configuration behavior
  - [x] replica corruption repair behavior

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- Implemented control-plane API and data model in `src/lib.rs`.
- Added control-plane error categories in `src/error.rs`.
- Integrated `moonblokz-crypto` dependency for `PRIVATE_KEY_SIZE`.
- Implemented memory-backend control-plane replication, CRC32 validation, compatibility checks, and repair logic.
- Updated RP2040 backend trait implementation with temporary stubs for new methods to keep compile contract intact.
- Updated conformance tests to use new `init(...)` signature.
- Executed `cargo test` in `moonblokz-storage` (all unit and doc tests pass).

### File List

- `moonblokz-storage/Cargo.toml`
- `moonblokz-storage/src/lib.rs`
- `moonblokz-storage/src/error.rs`
- `moonblokz-storage/src/backend_memory.rs`
- `moonblokz-storage/src/backend_rp2040.rs`
- `moonblokz-storage/src/conformance.rs`
- `/_bmad-output/implementation-artifacts/2-4-implement-control-plane-init-and-load-apis-in-memory-backend.md`

## Change Log

- 2026-02-28: Implemented control-plane init/load/set-once APIs in memory backend with replica CRC validation and best-effort repair; story set to `review`.
- 2026-02-28: Code review completed with fixes applied; story marked `done`.
