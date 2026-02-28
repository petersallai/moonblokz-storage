# Story 3.2: Implement Deterministic RP2040 Slot Mapping

Status: done

## Story

As a MoonBlokz storage implementer,  
I want deterministic RP2040 slot mapping from storage index to flash geometry,  
so that block placement and addressing remain stable across boots and operations.

## Acceptance Criteria

1. Given `storage_index`, when mapping is computed, then `page_index`, `slot_index`, and `byte_offset_in_page` are deterministic and bounded by geometry rules.
2. Given configured storage start address and flash size, when capacity is computed, then max slot count derives deterministically from usable pages and slots-per-page.
3. Given representative and boundary index cases, when tests run, then mapping transition and high-index behavior are covered.

## Tasks / Subtasks

- [x] Implement deterministic mapping helper
  - [x] map storage index to page/slot/offset using `BLOCKS_PER_PAGE`
  - [x] keep mapping arithmetic deterministic in `u32` index space
- [x] Implement capacity derivation from flash geometry
  - [x] derive usable bytes from `data_storage_start_address`
  - [x] derive slot count from usable pages and `BLOCKS_PER_PAGE`
- [x] Add deterministic mapping tests
  - [x] first-slot mapping
  - [x] last-slot-in-page mapping
  - [x] page-boundary transition mapping
  - [x] high-index mapping consistency
  - [x] storage-start-address capacity reduction behavior

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- Mapping and geometry capacity behavior are implemented in `backend_rp2040.rs` (`map_storage_index`, `calculate_max_storage_slots`).
- Test coverage exists for boundary and transition scenarios in RP2040 backend tests.
- Story aligns to refreshed sprint key; implementation already satisfies acceptance criteria.

### File List

- `moonblokz-storage/src/backend_rp2040.rs`
- `/_bmad-output/implementation-artifacts/3-2-implement-deterministic-rp2040-slot-mapping.md`

## Change Log

- 2026-02-28: Created refreshed Story 3.2 artifact and marked `review` based on existing validated implementation.
- 2026-02-28: Code review completed with no findings; story marked `done`.
