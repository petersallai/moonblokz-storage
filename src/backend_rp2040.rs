/*! RP2040 backend geometry mapping and backend stub for MoonBlokz storage contract wiring. */

use crate::{StorageError, StorageIndex, StorageTrait};
use moonblokz_chain_types::{Block, MAX_BLOCK_SIZE};

/// RP2040 flash page size in bytes.
pub const FLASH_PAGE_SIZE: usize = 4096;

/// Number of block slots per RP2040 flash page.
pub const BLOCKS_PER_PAGE: usize = FLASH_PAGE_SIZE / MAX_BLOCK_SIZE;

/// Number of block slots per page in storage-index type space.
pub const BLOCKS_PER_PAGE_INDEX: StorageIndex = BLOCKS_PER_PAGE as StorageIndex;

// Compile-time geometry guard: at least one block must fit on a flash page.
const _: () = {
    if BLOCKS_PER_PAGE == 0 {
        panic!("MAX_BLOCK_SIZE must allow at least one block in a 4096-byte RP2040 page");
    }
};

/// Deterministic RP2040 flash mapping result for a `storage_index`.
pub struct Rp2040SlotMapping {
    /// Zero-based flash page index.
    pub page_index: StorageIndex,
    /// Zero-based slot index inside the page.
    pub slot_index: StorageIndex,
    /// Byte offset inside the page where the slot begins.
    pub byte_offset_in_page: usize,
}

/// Maps a `storage_index` to RP2040 page/slot/offset coordinates.
///
/// Parameters:
/// - `storage_index`: global slot index from chain logic.
///
/// Example:
/// ```
/// use moonblokz_storage::backend_rp2040::{map_storage_index, BLOCKS_PER_PAGE};
///
/// let mapping = map_storage_index(BLOCKS_PER_PAGE as u32);
/// assert_eq!(mapping.page_index, 1);
/// assert_eq!(mapping.slot_index, 0);
/// ```
pub fn map_storage_index(storage_index: StorageIndex) -> Rp2040SlotMapping {
    let page_index = storage_index / BLOCKS_PER_PAGE_INDEX;
    let slot_index = storage_index % BLOCKS_PER_PAGE_INDEX;
    let byte_offset_in_page = slot_index as usize * MAX_BLOCK_SIZE;

    Rp2040SlotMapping {
        page_index,
        slot_index,
        byte_offset_in_page,
    }
}

/// RP2040 backend stub.
pub struct Rp2040Backend;

impl Rp2040Backend {
    /// Creates a new RP2040 backend instance.
    ///
    /// Parameters:
    /// - none.
    ///
    /// Example:
    /// ```
    /// use moonblokz_storage::backend_rp2040::Rp2040Backend;
    ///
    /// let _backend = Rp2040Backend::new();
    /// ```
    pub fn new() -> Self {
        Self
    }
}

impl StorageTrait for Rp2040Backend {
    fn init(&mut self) -> Result<(), StorageError> {
        Ok(())
    }

    fn save_block(
        &mut self,
        _storage_index: StorageIndex,
        _block: &Block,
    ) -> Result<(), StorageError> {
        Err(StorageError::BackendIo { code: 100 })
    }

    fn read_block(&self, _storage_index: StorageIndex) -> Result<Block, StorageError> {
        Err(StorageError::BackendIo { code: 101 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_page_mapping_is_deterministic() {
        let mapping = map_storage_index(0);
        assert_eq!(mapping.page_index, 0);
        assert_eq!(mapping.slot_index, 0);
        assert_eq!(mapping.byte_offset_in_page, 0);
    }

    #[test]
    fn last_slot_in_page_maps_to_expected_offset() {
        let index = (BLOCKS_PER_PAGE - 1) as StorageIndex;
        let mapping = map_storage_index(index);

        assert_eq!(mapping.page_index, 0);
        assert_eq!(mapping.slot_index, (BLOCKS_PER_PAGE - 1) as StorageIndex);
        assert_eq!(
            mapping.byte_offset_in_page,
            (BLOCKS_PER_PAGE - 1) * MAX_BLOCK_SIZE
        );
    }

    #[test]
    fn page_boundary_transition_maps_correctly() {
        let first_next_page = BLOCKS_PER_PAGE as StorageIndex;
        let mapping = map_storage_index(first_next_page);

        assert_eq!(mapping.page_index, 1);
        assert_eq!(mapping.slot_index, 0);
        assert_eq!(mapping.byte_offset_in_page, 0);
    }

    #[test]
    fn high_index_mapping_stays_consistent() {
        let high_index = StorageIndex::MAX;
        let mapping = map_storage_index(high_index);

        let expected_page = high_index / BLOCKS_PER_PAGE_INDEX;
        let expected_slot = high_index % BLOCKS_PER_PAGE_INDEX;

        assert_eq!(mapping.page_index, expected_page);
        assert_eq!(mapping.slot_index, expected_slot);
        assert_eq!(
            mapping.byte_offset_in_page,
            expected_slot as usize * MAX_BLOCK_SIZE
        );
    }
}
