# Story 3.3: Implement RP2040 Control-Plane Replication and CRC Recovery

Status: review

## Story

As a MoonBlokz chain runtime maintainer,  
I want RP2040 control-plane replicas with CRC validation and best-effort recovery,  
so that control metadata survives partial corruption and remains deterministic to load.

## Acceptance Criteria

1. Given RP2040 control-plane lifecycle APIs, when `init(...)` runs, then all control-plane replicas are rewritten deterministically and block-storage pages are cleared.
2. Given `load_control_data()`, when replicas are scanned, then first valid CRC-compatible replica is returned and invalid replicas are repaired best-effort.
3. Given incompatible persisted constants, when loading control-plane data, then explicit incompatibility error is returned.
4. Given set-once chain configuration semantics, when `set_chain_configuration` is called repeatedly, then first call persists and subsequent calls return explicit already-set error.

## Tasks / Subtasks

- [x] Implement RP2040 control-plane entry serialization/validation
  - [x] encode/decode schema with version/size fields/max-block-size/chain-config/crc32
  - [x] compatibility checks for persisted constants
- [x] Implement control-plane replica management
  - [x] reserve `CONTROL_PLANE_COUNT` flash pages at `data_storage_start_address`
  - [x] scan replicas in deterministic order
  - [x] repair invalid replicas after finding first valid record
- [x] Implement RP2040 control-plane API behavior
  - [x] `init(...)` erase + initialize replicas
  - [x] `load_control_data()` with CRC and compatibility enforcement
  - [x] `set_chain_configuration()` set-once behavior
- [x] Adjust RP2040 capacity/mapping geometry
  - [x] subtract reserved control-plane pages from block slot capacity
  - [x] map block pages after reserved control-plane area
- [x] Add/adjust deterministic RP2040 tests
  - [x] uninitialized load error
  - [x] init/load round-trip
  - [x] set-once chain configuration
  - [x] replica corruption repair
  - [x] updated capacity/address expectations with reserved pages

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- Implemented control-plane serialization, CRC32 validation, compatibility checks, and replica repair in `backend_rp2040.rs`.
- Integrated control-plane reserved pages into RP2040 slot capacity and flash address mapping.
- Implemented real RP2040 trait behavior for `init`, `set_chain_configuration`, and `load_control_data` (not stubs).
- Updated RP2040/conformance tests to align with reserved control-plane geometry.
- Verified with `cargo test --no-default-features --features backend-rp2040`.

### File List

- `moonblokz-storage/src/backend_rp2040.rs`
- `moonblokz-storage/src/conformance.rs`
- `/_bmad-output/implementation-artifacts/3-3-implement-rp2040-control-plane-replication-and-crc-recovery.md`

## Change Log

- 2026-02-28: Implemented RP2040 control-plane replication and CRC recovery behavior with tests; story set to `review`.
