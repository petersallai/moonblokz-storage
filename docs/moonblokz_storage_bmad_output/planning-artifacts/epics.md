---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
---

# moonblokz - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for moonblokz, decomposing the requirements from the PRD and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: MoonBlokz Chain Runtime can initialize storage on supported backend devices.
FR2: MoonBlokz Chain Runtime can perform startup loading by iterating through storage indexes and reading blocks for reconstruction.
FR3: MoonBlokz Chain Runtime can detect whether storage state is usable for chain reconstruction.
FR4: MoonBlokz Chain Runtime can retrieve storage capacity boundaries derived from configured block sizing constraints.
FR5: MoonBlokz Chain Runtime can determine availability of indexed storage slots.
FR6: MoonBlokz Chain Runtime can call `init(private_key, own_node_id, init_params)` before first use to initialize storage state.
FR7: Storage can accept `private_key: [u8; PRIVATE_KEY_SIZE]`, `own_node_id: u32`, and `init_params: [u8; INIT_PARAMS_SIZE]` in `init()`, where `INIT_PARAMS_SIZE = 100`.
FR8: Storage can erase (set to zero) all control-plane pages and all block-storage pages during `init()` before writing initial control-plane data.
FR9: Storage can enforce immutable control-plane initialization data after `init()`, except when `init()` is called again to restart from empty state.
FR10: Storage can persist a set-once chain configuration block via `set_chain_configuration(Block)` and return an explicit error if it was already set.
FR11: Storage can load all control-plane data through `load_control_data()` and return private key, own node id, init parameters, and optional chain configuration block.
FR12: Storage can return an explicit error from `load_control_data()` when storage has not been initialized.
FR13: Storage can persist control-plane schema fields including version constant, persisted size fields, `max_block_size`, reserved `chain_config_block` space, and `crc32`.
FR14: Storage can validate persisted size fields against runtime constants from the binary and return an explicit error on mismatch.
FR15: Storage can reserve and persist chain-configuration block space during `init()` even when no chain configuration is set yet.
FR16: Storage can write control-plane data to `CONTROL_PLANE_COUNT` replicated instances on every modifying operation (`init`, `set_chain_configuration`).
FR17: Storage can read control-plane replicas in deterministic order, validate CRC32, return the first valid instance, and continue scanning on checksum failure.
FR18: Storage can attempt best-effort repair of failed control-plane replicas after reading one valid replica.
FR19: Storage can place control-plane replicas at the start of storage region (`data_storage_start_address`) and map block-storage addresses after the reserved control-plane area.
FR20: Storage can calculate block-storage slot capacity from remaining storage after subtracting reserved replicated control-plane pages.
FR21: MoonBlokz Chain Runtime can persist a blockchain block at a specific `storage_index`.
FR22: MoonBlokz Chain Runtime can retrieve a blockchain block from a specific `storage_index`.
FR23: Storage can map `storage_index` deterministically to physical storage address space.
FR24: Storage can reject persistence operations targeting invalid or out-of-range indices.
FR25: Storage can report whether a requested indexed block is absent, valid, or invalid.
FR26: Storage can recompute and verify block hash integrity on retrieval before returning block data.
FR27: Storage can return explicit integrity errors when computed and stored block hashes differ.
FR28: Storage can detect partial or invalid write artifacts during subsequent read/startup operations.
FR29: Storage can surface explicit error categories suitable for chain-level recovery decisions.
FR30: Storage can avoid returning data that fails integrity validation.
FR31: MoonBlokz Chain Runtime can use storage APIs from `no_std` synchronous Rust execution contexts.
FR32: MoonBlokz Chain Runtime can execute startup reconstruction as a chain-initiated indexed read cycle using documented storage APIs.
FR33: MoonBlokz Chain Runtime can persist chain-accepted blocks through storage APIs during normal ingest flow.
FR34: MoonBlokz Chain Runtime can execute query retrieval flow via storage APIs.
FR35: Storage can preserve strict separation of concerns where chain-policy decisions remain external to storage.
FR36: Storage can expose a backend-agnostic interface supporting multiple device implementations.
FR37: Storage Backend Implementer can implement a new hardware backend without changing chain-level storage semantics.
FR38: Storage can provide an RP2040 backend implementation for MVP usage.
FR39: Storage can provide an in-memory backend implementation for off-target blockchain testing.
FR40: Storage can support conformance validation of backend implementations against shared API semantics.
FR41: MoonBlokz Chain Developer can use block data structures from a dedicated blockchain types crate separated from storage.
FR42: Storage can consume canonical block definitions and the `calculate_hash(&[u8]) -> [u8; HASH_SIZE]` contract via the blockchain types boundary.
FR43: Storage can operate without owning chain-policy metadata semantics beyond required persistence integrity metadata.
FR44: MoonBlokz Chain Developer can integrate storage as a Git dependency during initial adoption.
FR45: MoonBlokz Chain Developer can integrate storage as a published crate after `crates.io` release.
FR46: Storage can provide stable versioned public APIs suitable for downstream dependency management.
FR47: Storage can provide API-level usage documentation sufficient for integrator onboarding.
FR48: Developers can access file-level module documentation at the start of each source file.
FR49: Developers can access function-level documentation including input parameter descriptions.
FR50: Developers can access struct and field documentation for public data models.
FR51: Developers can access at least one usage example per public function.
FR52: Developers can access a `README.md` with API overview and integration guidance.
FR53: Developers can access a guide describing how to add support for a new device/backend.

### NonFunctional Requirements

NFR1: Storage operations shall use effective algorithms appropriate for constrained embedded hardware.
NFR2: Storage behavior shall remain deterministic and bounded across startup/read/write paths.
NFR3: Performance expectations shall be interpreted relative to hardware capabilities (RP2040 flash/RAM and execution model), not fixed universal SLA numbers.
NFR4: Storage shall enforce integrity-focused behavior by validating block hash consistency on retrieval paths.
NFR5: Storage shall never return data that fails integrity validation.
NFR6: Encryption requirements are out of scope for this storage layer; security focus is integration-safe integrity handling.
NFR7: Storage shall return explicit, actionable errors for integrity mismatches and partial/invalid write artifacts.
NFR8: Storage shall avoid silent fallback behavior when invalid data is encountered.
NFR9: Startup/read behavior shall remain predictable and consistent under failure conditions to support deterministic chain-level recovery.
NFR10: Control-plane modifications (`init`, `set_chain_configuration`) shall write all `CONTROL_PLANE_COUNT` replicas deterministically.
NFR11: Control-plane reads shall validate CRC32 and perform bounded fallback scanning across replicas.
NFR12: When at least one valid control-plane replica is found, storage shall attempt best-effort repair of invalid replicas without breaking deterministic read behavior.
NFR13: Storage integration shall remain compatible with MoonBlokz Embassy-based runtime architecture.
NFR14: Public storage interfaces shall remain Rust `no_std` and synchronous.
NFR15: Although async storage techniques (for example DMA-enabled approaches) may exist, this product shall intentionally use synchronous APIs because RP2040 XIP flash operations block both cores.
NFR16: Storage shall preserve strict architectural boundaries with chain logic and blockchain types crate responsibilities.
NFR17: RP2040 and non-RP2040 backend implementations (including in-memory test backend) shall preserve consistent API semantics and behavioral contracts.
NFR18: Implementation choices shall be simplicity-first for embedded limits, using minimal logic and minimal state needed to satisfy requirements.
NFR19: New abstractions, traits, derives, or helper layers that increase binary size or runtime overhead shall require explicit justification in architecture/task artifacts.
NFR20: Documentation and code reviews shall treat unnecessary complexity as a defect for MVP scope.

### Additional Requirements

- Use two separate repositories: `moonblokz-storage` and `moonblokz-chain-types`.
- Initialize both as Rust library crates; maintain `no_std` and synchronous API model.
- Enforce compile-time backend feature exclusivity (exactly one backend feature enabled; build error otherwise).
- Keep backend implementations isolated (no shared backend implementation code across backend modules).
- Keep integrity/hash/error behavior implemented by each backend while preserving shared API semantics.
- Use stable Rust channel for implementation and CI.
- Follow backend matrix testing pattern from `moonblokz-crypto-lib/run_tests.sh`: one backend feature per run with `--no-default-features`.
- Provide conformance tests to ensure semantic parity across backends.
- Maintain compatibility boundary: `moonblokz-storage` depends on canonical block/hash contracts from `moonblokz-chain-types`.
- Keep chain runtime integration on concrete backend structs while maintaining a shared storage trait contract.
- Use the `log` crate for runtime logging; avoid ad hoc logging patterns.
- Enforce embedded binary-size discipline (avoid unnecessary derives/defaults/helper layers; justify overhead-increasing abstractions).

### FR Coverage Map

FR1: Epic 2 - Storage lifecycle management baseline
FR2: Epic 2 - Startup read-cycle support
FR3: Epic 2 - Storage usability/state checks
FR4: Epic 2 - Capacity boundary reporting
FR5: Epic 2 - Indexed slot availability behavior
FR6: Epic 2 - Init API contract
FR7: Epic 2 - Init parameter contract
FR8: Epic 2 - Destructive init erase semantics
FR9: Epic 2 - Control data immutability until re-init
FR10: Epic 2 - Set-once chain configuration API behavior
FR11: Epic 2 - Control data load API return contract
FR12: Epic 2 - Uninitialized-storage explicit error behavior
FR13: Epic 2 - Control-plane schema persistence contract
FR14: Epic 2 - Persisted constant-size compatibility checks
FR15: Epic 2 - Reserved chain-config slot persistence
FR16: Epic 2 - Replicated control-plane write contract
FR17: Epic 2 - Replicated control-plane read/CRC fallback contract
FR18: Epic 2 - Best-effort replica repair contract
FR19: Epic 2 - Control-plane start-address layout contract
FR20: Epic 2 - Block-capacity calculation excluding control-plane reservation
FR21: Epic 2 - Indexed block save API
FR22: Epic 2 - Indexed block read API
FR23: Epic 2 - Deterministic index-to-address mapping contract
FR24: Epic 2 - Invalid index rejection behavior
FR25: Epic 2 - Absent/valid/invalid slot state contract
FR26: Epic 2 - Read-time hash recomputation contract
FR27: Epic 2 - Hash mismatch explicit error behavior
FR28: Epic 2 - Partial/invalid artifact detection behavior
FR29: Epic 2 - Chain-recoverable typed error surface
FR30: Epic 2 - Never return integrity-failed data
FR31: Epic 2 - no_std synchronous API usage by chain runtime
FR32: Epic 2 - Startup reconstruction read-cycle usage contract
FR33: Epic 2 - Ingest persistence contract
FR34: Epic 2 - Query retrieval contract
FR35: Epic 2 - Storage vs chain policy boundary contract
FR36: Epic 2 - Backend-agnostic interface contract
FR37: Epic 4 - New backend implementer enablement
FR38: Epic 3 - RP2040 backend implementation delivery
FR39: Epic 2 - In-memory backend implementation delivery
FR40: Epic 2 - Cross-backend conformance validation support
FR41: Epic 1 - Dedicated chain-types crate and canonical block ownership
FR42: Epic 1 - Canonical hash utility contract
FR43: Epic 1 - Types/storage responsibility boundary
FR44: Epic 4 - Git dependency integration readiness
FR45: Epic 4 - crates.io integration readiness
FR46: Epic 4 - Stable public API/versioning readiness
FR47: Epic 4 - API onboarding documentation readiness
FR48: Epic 4 - File-level docs requirements
FR49: Epic 4 - Function-level parameter documentation requirements
FR50: Epic 4 - Struct and field documentation requirements
FR51: Epic 4 - Usage example coverage requirements
FR52: Epic 4 - README API/integration documentation requirements
FR53: Epic 4 - New-device support guide requirements

## Epic List

### Epic 1: Canonical Chain Types and Hash Contract
Establish immutable block/type and hash contracts so all storage backends and chain logic use one canonical data model.
**FRs covered:** FR41, FR42, FR43

### Epic 2: Core Storage API and In-Memory Backend
Deliver a usable `no_std` synchronous storage API with deterministic indexed behavior and a full in-memory backend so chain development/testing works without RP2040 hardware.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR7, FR8, FR9, FR10, FR11, FR12, FR13, FR14, FR15, FR16, FR17, FR18, FR19, FR20, FR21, FR22, FR23, FR24, FR25, FR26, FR27, FR28, FR29, FR30, FR31, FR32, FR33, FR34, FR35, FR36, FR39, FR40

### Epic 3: RP2040 Production Backend
Implement RP2040 flash-backed storage with control-plane replication and deterministic slot mapping so MoonBlokz nodes can operate on target hardware.
**FRs covered:** FR38, plus RP2040 realization of FR6-FR35

### Epic 4: Developer Adoption, Conformance, and Distribution
Make the libraries consumable and maintainable with docs, examples, conformance validation, and packaging workflow for Git now and crates.io later.
**FRs covered:** FR37, FR44, FR45, FR46, FR47, FR48, FR49, FR50, FR51, FR52, FR53

## Epic 1: Canonical Chain Types and Hash Contract

Establish immutable block/type and hash contracts so all storage backends and chain logic use one canonical data model.

### Story 1.1: Set Up Initial Projects from Starter Template

As a MoonBlokz chain developer,  
I want both `moonblokz-storage` and `moonblokz-chain-types` initialized from the selected Cargo library starter approach,  
So that implementation begins from the required architecture baseline.

**Implements:** FR41, FR44

**Acceptance Criteria:**

**Given** a fresh repository for `moonblokz-chain-types`  
**When** the crate is initialized on stable Rust  
**Then** both crates build in `no_std` mode on stable Rust  
**And** each crate exposes a documented public API entrypoint in `lib.rs`.

### Story 1.2: Implement Immutable Block Representation Contract

As a storage backend implementer,  
I want `Block` and `BlockBuilder` contracts with canonical serialized-byte access,  
So that all storage code uses a single validated block format.

**Implements:** FR41, FR43

**Acceptance Criteria:**

**Given** the chain-types crate  
**When** block contracts are implemented  
**Then** `Block` is immutable after creation and supports binary-form construction  
**And** canonical serialized bytes can be retrieved without backend-specific reinterpretation.

### Story 1.3: Implement Canonical SHA-256 Hash Utility

As a storage backend implementer,  
I want a shared `calculate_hash(&[u8]) -> [u8; HASH_SIZE]` function in chain-types,  
So that integrity checks are deterministic across all backends.

**Implements:** FR42

**Acceptance Criteria:**

**Given** canonical hashing requirements in architecture/PRD  
**When** hash utility is added  
**Then** `HASH_SIZE` and SHA-256 behavior are defined in chain-types  
**And** storage and tests can consume this contract without duplicate hash implementations.

## Epic 2: Core Storage API and In-Memory Backend

Deliver a usable `no_std` synchronous storage API with deterministic indexed behavior and a full in-memory backend so chain development/testing works without RP2040 hardware.

### Story 2.1: Initialize Storage Crate with Feature Exclusivity Guards

As a MoonBlokz chain developer,  
I want `moonblokz-storage` initialized with compile-time backend feature exclusivity,  
So that exactly one backend is active per build.

**Implements:** FR36, FR39, FR40

**Acceptance Criteria:**

**Given** storage crate feature configuration  
**When** zero or multiple backend features are enabled  
**Then** compilation fails with explicit error messaging  
**And** compilation succeeds when exactly one backend feature is enabled.

### Story 2.2: Define Public Storage Trait and Error Contract

As a MoonBlokz chain runtime maintainer,  
I want a synchronous `no_std` storage trait and typed error surface,  
So that startup/ingest/query flows have deterministic contracts.

**Implements:** FR1, FR2, FR3, FR4, FR5, FR21, FR22, FR23, FR24, FR25, FR31, FR32, FR33, FR34, FR35

**Acceptance Criteria:**

**Given** FR1-FR5 and FR21-FR35 contracts  
**When** the public storage trait and errors are defined  
**Then** lifecycle, indexed IO, and integrity/result semantics are represented explicitly  
**And** no chain-policy responsibilities are encoded in storage API behavior.

### Story 2.3: Implement In-Memory Backend for Indexed Block IO

As a MoonBlokz chain developer,  
I want an in-memory backend implementing the storage trait,  
So that blockchain behavior can be tested off-target without RP2040 hardware.

**Implements:** FR39, FR40

**Acceptance Criteria:**

**Given** the storage trait contract  
**When** the memory backend is implemented  
**Then** save/retrieve/index validation semantics match the public contract  
**And** behavior remains deterministic with bounded in-memory state.

### Story 2.4: Implement Control-Plane Init and Load APIs in Memory Backend

As a MoonBlokz chain runtime maintainer,  
I want memory-backend support for `init`, `load_control_data`, and control data schema persistence,  
So that control-plane lifecycle flows can be tested before hardware integration.

**Implements:** FR6, FR7, FR8, FR9, FR11, FR12, FR13, FR14, FR15, FR16, FR17, FR18, FR19, FR20

**Acceptance Criteria:**

**Given** control-plane FR6-FR20 requirements  
**When** memory backend control-plane APIs are implemented  
**Then** init erase semantics, immutable-until-reinit behavior, and control-data loading are enforced  
**And** uninitialized access returns explicit typed errors.

### Story 2.5: Implement Set-Once Chain Configuration Semantics

As a MoonBlokz chain runtime maintainer,  
I want set-once chain configuration behavior in storage,  
So that chain configuration persistence is deterministic and safe.

**Implements:** FR10

**Acceptance Criteria:**

**Given** initialized control-plane state  
**When** `set_chain_configuration(Block)` is called the first time  
**Then** chain configuration is persisted  
**And** subsequent calls return explicit already-set error unless full re-init occurs.

### Story 2.6: Implement Integrity Verification and Invalid Data Rejection

As a MoonBlokz node operator,  
I want retrieval to verify integrity before returning block data,  
So that corrupted/partial data is never silently accepted.

**Implements:** FR26, FR27, FR28, FR29, FR30

**Acceptance Criteria:**

**Given** persisted block and hash metadata  
**When** block retrieval occurs  
**Then** storage recomputes hash and compares expected values  
**And** hash mismatch/invalid artifacts return explicit errors and no block payload.

### Story 2.7: Implement Backend Conformance Test Suite

As a backend implementer,  
I want conformance tests that validate shared semantics,  
So that all backend implementations remain behaviorally consistent.

**Implements:** FR40

**Acceptance Criteria:**

**Given** storage trait and at least one backend  
**When** conformance tests run  
**Then** index mapping, integrity behavior, and error semantics are verified  
**And** tests are reusable across backend implementations.

### Story 2.8: Add Backend Matrix Test Runner

As a maintainer,  
I want backend-by-backend test execution aligned with `moonblokz-crypto-lib` style,  
So that feature-isolated behavior is continuously verified.

**Implements:** FR40

**Acceptance Criteria:**

**Given** multiple backend features  
**When** matrix test script runs with `--no-default-features` and one backend at a time  
**Then** each backend test set executes successfully in isolation  
**And** failures identify backend-specific contract breaks.

## Epic 3: RP2040 Production Backend

Implement RP2040 flash-backed storage with control-plane replication and deterministic slot mapping so MoonBlokz nodes can operate on target hardware.

### Story 3.1: Implement RP2040 Backend Skeleton and Flash Integration

As an embedded Rust engineer,  
I want an RP2040 backend wired to Embassy-compatible flash APIs,  
So that storage operations execute against real device flash.

**Implements:** FR38

**Acceptance Criteria:**

**Given** RP2040 backend feature selection  
**When** backend struct is implemented  
**Then** flash peripheral integration compiles for ARM target builds  
**And** non-ARM test builds use test-safe backend behavior.

### Story 3.2: Implement Deterministic RP2040 Slot Mapping

As an embedded Rust engineer,  
I want deterministic mapping from `storage_index` to RP2040 flash addresses,  
So that block placement is predictable and bounded.

**Implements:** FR23, FR19, FR20

**Acceptance Criteria:**

**Given** compile-time storage constants and data storage start address  
**When** block address mapping is calculated  
**Then** mapping is deterministic and index bounds are enforced  
**And** control-plane reserved area is excluded from block slot capacity.

### Story 3.3: Implement RP2040 Control-Plane Replication and CRC Recovery

As a MoonBlokz chain runtime maintainer,  
I want control-plane data replicated with CRC32 validation and fallback,  
So that boot can recover from partial flash corruption.

**Implements:** FR16, FR17, FR18

**Acceptance Criteria:**

**Given** `CONTROL_PLANE_COUNT` replicas  
**When** `init` or `set_chain_configuration` modifies control data  
**Then** all replicas are written deterministically  
**And** reads scan replicas in order, use first valid CRC entry, and attempt best-effort repair of invalid replicas.

### Story 3.4: Implement RP2040 Block IO with Integrity Enforcement

As a MoonBlokz node operator,  
I want RP2040 block save/retrieve behavior to enforce integrity guarantees,  
So that node operation remains safe under flash faults.

**Implements:** FR21, FR22, FR24, FR25, FR26, FR27, FR28, FR29, FR30, FR38

**Acceptance Criteria:**

**Given** saved RP2040 block data  
**When** retrieval executes  
**Then** hash verification and error semantics match shared contract behavior  
**And** invalid/partial data is rejected with explicit typed errors.

### Story 3.5: Validate RP2040 Behavior with Integration Tests

As a maintainer,  
I want integration tests for RP2040 startup/ingest/query flows,  
So that production backend behavior is validated end-to-end.

**Implements:** FR32, FR33, FR34, FR38

**Acceptance Criteria:**

**Given** RP2040 backend feature enabled in tests  
**When** startup cycle, ingest persistence, and query retrieval scenarios run  
**Then** outcomes align with FR and NFR contracts  
**And** control-plane lifecycle behavior is covered by tests.

## Epic 4: Developer Adoption, Conformance, and Distribution

Make the libraries consumable and maintainable with docs, examples, conformance validation, and packaging workflow for Git now and crates.io later.

### Story 4.1: Produce Required API Documentation Baseline

As a MoonBlokz chain developer,  
I want complete source-level API documentation coverage,  
So that integration usage is clear without reverse-engineering internals.

**Implements:** FR48, FR49, FR50, FR51

**Acceptance Criteria:**

**Given** both crates  
**When** documentation pass is completed  
**Then** file/module/function/struct/field comments satisfy PRD documentation requirements  
**And** every public function includes parameter descriptions and at least one usage example.

### Story 4.2: Publish README and New-Device Support Guides

As a backend implementer,  
I want practical README and device-extension guides,  
So that onboarding and new backend development are straightforward.

**Implements:** FR52, FR53

**Acceptance Criteria:**

**Given** storage and chain-types crates  
**When** repository documentation is finalized  
**Then** README files describe API contracts and integration usage  
**And** docs include clear process for adding a new backend/device.

### Story 4.3: Finalize CI and Conformance Gates

As a maintainer,  
I want CI gates enforcing feature exclusivity and backend matrix conformance,  
So that regressions are detected before integration.

**Implements:** FR37, FR40

**Acceptance Criteria:**

**Given** CI workflows and test scripts  
**When** pull request validation runs  
**Then** exclusivity checks, backend matrix tests, and conformance suites execute  
**And** failures block merge until contract compliance is restored.

### Story 4.4: Prepare Git Dependency and crates.io Metadata

As a downstream integrator,  
I want correct package metadata and dependency instructions,  
So that adoption is smooth now via Git and later via crates.io.

**Implements:** FR44, FR45, FR46, FR47

**Acceptance Criteria:**

**Given** both crate manifests and READMEs  
**When** distribution metadata is reviewed  
**Then** repository links, package metadata, and dependency examples are correct  
**And** release-readiness notes for future crates.io publication are documented.

### Story 4.5: Add `std` Storage Lifecycle Example Project

As a MoonBlokz chain developer,  
I want a host-runnable `std` example project for `moonblokz-storage`,  
So that storage integration flow can be validated quickly without RP2040 hardware.

**Implements:** FR47, FR52
**Status:** Completed (2026-02-28)

**Acceptance Criteria:**

**Given** the `moonblokz-storage/examples/moonblokz-storage-std-example` project  
**When** it is run on host  
**Then** it checks initialization state using `load_control_data()`  
**And** it initializes storage when uninitialized, then performs block save+read flow using public APIs.

### Story 4.6: Add RP2040 Embedded Storage Lifecycle Example Project

As a MoonBlokz node operator,  
I want an RP2040 embedded example project for `moonblokz-storage`,  
So that target-hardware integration can be validated with simple device feedback.

**Implements:** FR37, FR38, FR47, FR52, FR53
**Status:** Completed (2026-02-28)

**Acceptance Criteria:**

**Given** the `moonblokz-storage/examples/moonblokz-storage-embedded-example` project  
**When** it is built for `thumbv6m-none-eabi`  
**Then** it demonstrates initialization detection, conditional init, block save, and block read using RP2040 backend  
**And** it blinks LED once for 0.5 seconds on success or blinks LED three times on failure.

### Story 4.7: Document Example Build and Run Paths

As a backend implementer,  
I want clear commands for both example projects,  
So that onboarding and validation are deterministic.

**Implements:** FR47, FR52, FR53
**Status:** Completed (2026-02-28)

**Acceptance Criteria:**

**Given** `moonblokz-storage` documentation  
**When** a developer follows README instructions  
**Then** they can build/run the `std` example and build the RP2040 embedded example with explicit commands  
**And** the example behavior contract (init check, save/read, LED signaling) is documented.
