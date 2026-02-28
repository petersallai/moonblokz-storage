# Story 3.3: Implement RP2040 retrieve path with mandatory hash verification

Status: done

## Story

As a MoonBlokz chain runtime,
I want retrieved RP2040 blocks to be hash-verified before return,
so that chain logic never receives corrupted block data as valid.

## Acceptance Criteria

1. Given stored block bytes at a valid slot, when retrieve is called, then backend recomputes block hash and compares against stored hash before returning.
2. Hash mismatches return `StorageError::IntegrityFailure` and never return block data.
3. Invalid indices return `StorageError::InvalidIndex`.
4. Backend read failures return explicit `StorageError::BackendIo { code: ... }`.

## Tasks / Subtasks

- [x] Implement RP2040 read flow in `backend_rp2040.rs`
  - [x] Validate `storage_index` bounds
  - [x] Resolve page/slot mapping and read slot bytes synchronously
  - [x] Reconstruct block bytes from slot layout
- [x] Add mandatory hash verification
  - [x] Recompute SHA-256 using `moonblokz_chain_types::calculate_hash`
  - [x] Compare computed hash against stored hash metadata in slot
  - [x] Return `StorageError::IntegrityFailure` on mismatch
- [x] Add deterministic tests for retrieve behavior
  - [x] Valid retrieve with matching hash
  - [x] Hash mismatch returns `IntegrityFailure`
  - [x] Invalid index returns `InvalidIndex`
  - [x] Backend read failure propagates `BackendIo`
- [x] Keep implementation minimal
  - [x] Preserve synchronous API
  - [x] No trait/API expansion

## Developer Context

### Technical Requirements

- Continue implementation in `moonblokz-storage`.
- Preserve RP2040 backend behavior: real flash path on `arm && !test`, mock path on `!arm || test`.
- Use canonical hash API from `moonblokz-chain-types`.

### File Structure Requirements

Target files for this story:

- `moonblokz-storage/src/backend_rp2040.rs`
- `/_bmad-output/implementation-artifacts/3-3-implement-rp2040-retrieve-path-with-mandatory-hash-verification.md`

## References

- `/_bmad-output/planning-artifacts/epics.md` (Epic 3, Story 3.3)
- `/_bmad-output/planning-artifacts/architecture.md`
- `/_bmad-output/planning-artifacts/prd.md`
- `/_bmad-output/implementation-artifacts/3-2-implement-synchronous-rp2040-save-path-by-storage-index.md`

## Dev Agent Record

### Completion Notes

- Implemented RP2040 retrieve path (`read_block`) with deterministic slot mapping and index validation.
- Kept fixed-size block model (`MAX_BLOCK_SIZE = 2016`), and persisted one hash metadata field per slot.
- Slot layout is fixed-size: `[2016 block bytes][32 hash metadata bytes]` (total `2048` bytes per slot).
- Save path persists fixed-size block bytes and stores SHA-256 hash metadata for retrieve-time verification.
- Implemented retrieve-time integrity checks:
  - empty slot -> `BlockAbsent`
  - hash mismatch against stored slot metadata -> `IntegrityFailure`
  - valid hash -> reconstruct and return `Block`
- Preserved backend split behavior:
  - arm + non-test -> real Embassy flash read/write
  - non-arm or test -> mock flash path
- Added/updated RP2040 backend tests for:
  - successful read after save
  - empty-slot read
  - invalid-index read
  - hash mismatch detection
  - backend read I/O failure propagation

### Validation

- `cargo test` (default backend-memory): pass
- `cargo test --no-default-features --features backend-rp2040`: pass
- `./scripts/check_backend_features.sh`: pass

### File List

- `moonblokz-storage/src/backend_rp2040.rs`
- `moonblokz-storage/src/error.rs`
- `moonblokz-storage/README.md`
- `/_bmad-output/implementation-artifacts/3-3-implement-rp2040-retrieve-path-with-mandatory-hash-verification.md`
