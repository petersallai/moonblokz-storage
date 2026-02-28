---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments:
  - _bmad-output/brainstorming/brainstorming-session-2026-02-21-081124.md
date: 2026-02-22
author: Sasa
---

# Product Brief: moonblokz

<!-- Content will be appended sequentially through collaborative workflow steps -->

## Executive Summary

MoonBlokz needs a dedicated onboard storage layer to become operational as a real microcontroller-based blockchain network. Right now, there is no storage implementation, which means the chain cannot persist or recover state and the network cannot function end-to-end.

The product vision is to build a Rust `no_std` storage library for RP2040-first deployment that provides deterministic, bounded-memory block persistence and retrieval. The library is intentionally scoped as a low-level storage primitive: it stores and fetches blocks by explicit `storage_index`, derives flash addresses from index, validates integrity on reads via hash verification, and leaves chain-level policy decisions (for example pruning) to chain management logic.

Success for v1 is defined by correctness and operational safety under embedded constraints: reliable read/write behavior in constrained RAM, startup recovery via full block scan, detection of partial/corrupt writes through hash mismatch, and reclaim of invalid sectors for reuse.

---

## Core Vision

### Problem Statement

MoonBlokz currently lacks onboard blockchain storage, so nodes cannot persist blocks locally and the network cannot operate as intended. A storage subsystem is a hard dependency for initialization, recovery, and chain lifecycle management on microcontrollers.

### Problem Impact

Without storage:
- Nodes cannot maintain blockchain state across reboots or power cycles.
- Chain initialization logic cannot reconstruct required in-memory structures.
- The network cannot operate because only a small number of blocks can be stored in RAM.
- Network participation becomes non-viable in the intended disconnected, constrained deployment model.

### Why Existing Solutions Fall Short

There is no current MoonBlokz storage solution. Generic embedded storage approaches are not yet integrated with MoonBlokz's chain/index semantics, deterministic constraints, and integrity guarantees needed for radio-first microcontroller operation.

### Proposed Solution

Build a Rust `no_std`, synchronous, bounded-memory storage library with a pluggable backend abstraction and RP2040 as the first concrete backend.

Functional model:
- `save_block(storage_index, block)` and `get_block(storage_index)` style access.
- Flash address derivation from `storage_index` is owned by storage implementation.
- On retrieval, recalculate block hash and return block only if it matches stored hash.
- Startup recovery is full-block loading by chain logic; total startup time depends on flash read throughput plus per-block hash validation.
- Partial writes are detected on read/startup and affected sectors are reclaimed (erase/reuse path).
- Pruning/retention strategy is explicitly out of scope for storage and remains in chain logic.

### Key Differentiators

- Strict separation of concerns: storage handles deterministic indexed persistence; chain layer owns policy decisions.
- Integrity-first reads: hash validation on every retrieval prevents silent corruption acceptance.
- Embedded-fit architecture: `no_std`, sync API, bounded RAM footprint (kilobyte-scale runtime state), and deterministic behavior under constrained resources.
- Capacity planning grounded in explicit RP2040 formula:
  `(FLASH_SIZE - APP_BINARY_SIZE - CONTROL_PLANE_SIZE * 3) / BLOCK_SIZE`

## Target Users

### Primary Users

**MoonBlokz Blockchain Runtime (Initialization + Chain Management Logic)**

This storage library serves the MoonBlokz blockchain itself as an internal system dependency, not a direct human-facing user. The runtime operates on RP2040-class hardware with strict memory constraints (264KB RAM), where in-memory-only blockchain handling is not feasible.

**Context and environment**
- Embedded RP2040 deployment
- `no_std`, synchronous, deterministic execution model
- Flash-backed persistence required for practical chain operation

**Goals**
- Reconstruct chain-relevant in-memory structures at boot from persisted blocks
- Persist newly accepted blocks deterministically by `storage_index`
- Retrieve blocks on demand for radio query flows
- Ensure integrity by verifying block hash on reads before returning data

**Current pain/problem experience**
- RAM cannot hold enough blocks to support blockchain operation
- Without onboard storage, the blockchain runtime cannot function end-to-end
- Loss of durable block state prevents reliable startup and participation

**Success vision**
- Boot flow can read stored blocks and initialize internal structures reliably
- New valid blocks are saved predictably at computed flash locations
- Radio-driven block queries can retrieve expected blocks consistently
- Corrupted/partial block data is detected via hash mismatch and handled safely

### Secondary Users

N/A for this product scope.

### User Journey

**Discovery**
- N/A (internal platform component within MoonBlokz architecture)

**Onboarding (integration)**
- Chain runtime binds to storage API and backend implementation (RP2040 first)
- Startup path invokes full-block load process to rebuild runtime structures

**Core Usage**
- Boot: iterate stored blocks, validate/read, initialize chain data structures
- Ingest: on incoming block, run chain checks against existing data, then persist accepted block by `storage_index`
- Query: handle radio block requests by indexed retrieval + integrity validation before return

**Success Moment**
- First full boot-to-participation cycle completes on-device with persisted data and verified retrievals under RAM limits

**Long-term**
- Storage remains a stable low-level primitive while chain logic evolves independently (for example pruning/policy outside storage scope)

## Success Metrics

Success for the MoonBlokz storage library is defined by whether the blockchain runtime can reliably persist and reconstruct chain state on RP2040 under embedded constraints.

### User Success Metrics

- **Boot reconstruction correctness**
  - At startup, chain logic can load blocks from storage and initialize required internal data structures successfully.
  - No fixed startup-time SLA is required; runtime is expected to scale with total blocks, flash read performance, and per-block hash computation.

- **Read integrity enforcement**
  - On every retrieval, storage recomputes the block hash and compares it to the stored block hash.
  - A block is returned only when hashes match; otherwise, retrieval returns an error.

- **Write/recovery safety behavior**
  - If a block write is partial/invalid, the condition must be detectable on subsequent read/startup.
  - Storage reports an explicit error back to chain logic (no automatic reclaim required in this scope).

- **Capacity effectiveness**
  - Effective supported block slots are determined by:
    `floor((FLASH_SIZE - APP_BINARY_SIZE - CONTROL_PLANE_SIZE * 3) / BLOCK_SIZE)`

- **RAM usage discipline**
  - Storage-owned RAM usage is minimized to only what is strictly required for functionality (no unnecessary runtime-resident large structures).

### Business Objectives

- Enable a deployable MoonBlokz node on RP2040 by providing production-usable onboard blockchain persistence.
- Provide a stable, deterministic, `no_std` synchronous storage API that cleanly supports chain initialization and query flows.
- Preserve separation of concerns: storage handles indexed persistence/retrieval and integrity checks; chain logic owns chain-level policy decisions.

### Key Performance Indicators

- **KPI-1: Startup initialization reliability**
  - Chain startup successfully reconstructs required internal structures from persisted blocks in validation scenarios.

- **KPI-2: Retrieval integrity compliance**
  - Retrieval path enforces hash verification for every read and rejects mismatches with explicit error signaling.

- **KPI-3: Partial-write detection behavior**
  - Simulated/observed partial write cases are detected on later read/startup and surfaced to chain logic as errors.

- **KPI-4: Capacity conformance**
  - Storage implementation supports indexing and access across the computed capacity envelope:
    `floor((FLASH_SIZE - APP_BINARY_SIZE - CONTROL_PLANE_SIZE * 3) / BLOCK_SIZE)`.

- **KPI-5: Memory efficiency**
  - Storage runtime maintains minimal RAM footprint consistent with required functionality.

## MVP Scope

### Core Features

- Full MoonBlokz storage functionality implemented for RP2040.
- `no_std`, synchronous storage API for chain runtime integration.
- Indexed persistence model:
  - Save/retrieve operations use `storage_index`.
  - Flash address is derived from `storage_index`.
- Read-time integrity enforcement:
  - Recompute block hash on retrieval.
  - Return block only if computed hash matches stored hash.
- Startup support for chain initialization:
  - Chain logic can load stored blocks to rebuild internal data structures.
- Error signaling for invalid/partial data:
  - Partial/invalid block conditions are surfaced to chain logic as errors.

### Out of Scope for MVP

- Non-RP2040 backend implementations.
- Chain-level policies such as pruning/retention (owned by chain logic, not storage).
- Additional platform abstractions beyond what is required for RP2040 delivery.
- Human-facing tooling/UI around storage internals.

### MVP Success Criteria

- RP2040 implementation is complete enough for MoonBlokz chain runtime to operate with flash-backed storage.
- Boot-time block loading works reliably for chain structure initialization.
- Retrieval path consistently enforces hash verification and error signaling on mismatch.
- Effective capacity behavior conforms to:
  `floor((FLASH_SIZE - APP_BINARY_SIZE - CONTROL_PLANE_SIZE * 3) / BLOCK_SIZE)`.
- Implementation remains minimal in RAM usage and consistent with embedded constraints.

### Future Vision

- Add additional backend implementations for other hardware targets after RP2040 MVP is stable.
- Preserve the same deterministic API contract and storage semantics across backends.
- Evolve toward a multi-platform storage library while keeping chain-policy responsibilities outside storage.
