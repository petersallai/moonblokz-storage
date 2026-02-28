# Story 2.5: Implement Set-Once Chain Configuration Semantics

Status: done

## Story

As a MoonBlokz chain runtime maintainer,  
I want set-once chain configuration behavior in storage,  
so that chain configuration persistence is deterministic and safe.

## Acceptance Criteria

1. Given initialized control-plane state, when `set_chain_configuration(Block)` is called the first time, then chain configuration is persisted.
2. Given initialized control-plane state with chain configuration already set, when `set_chain_configuration(Block)` is called again, then explicit already-set error is returned unless full `init(...)` is executed.

## Tasks / Subtasks

- [x] Implement set-once write path
  - [x] Persist chain configuration on first call
  - [x] Keep deterministic control-plane replica updates
- [x] Enforce already-set error behavior
  - [x] Return `StorageError::ChainConfigurationAlreadySet` on repeated calls
- [x] Ensure re-init resets set-once state
  - [x] `init(...)` clears chain configuration slot
- [x] Validate with deterministic tests
  - [x] first write succeeds
  - [x] second write fails with explicit error
  - [x] loaded control data reflects persisted chain configuration

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- `set_chain_configuration` implemented in memory backend and wired via public trait.
- `ChainConfigurationAlreadySet` error category added and used.
- `init(...)` clears control-plane data including chain configuration reservation.
- Tests verify set-once semantics and control-plane round-trip state.

### File List

- `moonblokz-storage/src/lib.rs`
- `moonblokz-storage/src/error.rs`
- `moonblokz-storage/src/backend_memory.rs`
- `/_bmad-output/implementation-artifacts/2-5-implement-set-once-chain-configuration-semantics.md`

## Change Log

- 2026-02-28: Story context created and marked `review`; implementation and tests already satisfy set-once semantics.
- 2026-02-28: Code review completed with no findings; story marked `done`.
