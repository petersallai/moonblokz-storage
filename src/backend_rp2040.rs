/*! RP2040 backend geometry mapping and synchronous flash save/retrieve paths. */

use crate::{
    ControlPlaneData, INIT_PARAMS_SIZE, StorageError, StorageIndex, StorageTrait,
};
use core::cell::RefCell;
use moonblokz_chain_types::{Block, HASH_SIZE, MAX_BLOCK_SIZE, calculate_hash};
use moonblokz_crypto::PRIVATE_KEY_SIZE;

#[cfg(all(not(test), target_arch = "arm"))]
use embassy_rp::Peripheral;
#[cfg(all(not(test), target_arch = "arm"))]
use embassy_rp::flash::{Blocking, Flash};
#[cfg(all(not(test), target_arch = "arm"))]
use embassy_rp::peripherals::FLASH;

/// RP2040 flash page size in bytes.
pub const FLASH_PAGE_SIZE: usize = 4096;
/// Default RP2040 full flash size in bytes.
pub const RP2040_DEFAULT_FLASH_SIZE: usize = 2 * 1024 * 1024;
/// Slot hash metadata offset (after fixed-size block bytes).
const SLOT_HASH_OFFSET: usize = MAX_BLOCK_SIZE;
/// Total bytes used by one persisted slot (`block bytes + hash metadata`).
const SLOT_SIZE_BYTES: usize = MAX_BLOCK_SIZE + HASH_SIZE;

/// Number of block slots per RP2040 flash page.
pub const BLOCKS_PER_PAGE: usize = FLASH_PAGE_SIZE / SLOT_SIZE_BYTES;

/// Number of block slots per page in storage-index type space.
pub const BLOCKS_PER_PAGE_INDEX: StorageIndex = BLOCKS_PER_PAGE as StorageIndex;

// Compile-time geometry guard.
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
/// ```ignore
/// use moonblokz_storage::backend_rp2040::BLOCKS_PER_PAGE;
///
/// let mapping = map_storage_index(BLOCKS_PER_PAGE as u32);
/// assert_eq!(mapping.page_index, 1);
/// assert_eq!(mapping.slot_index, 0);
/// ```
fn map_storage_index(storage_index: StorageIndex) -> Rp2040SlotMapping {
    let page_index = storage_index / BLOCKS_PER_PAGE_INDEX;
    let slot_index = storage_index % BLOCKS_PER_PAGE_INDEX;
    let byte_offset_in_page = slot_index as usize * SLOT_SIZE_BYTES;

    Rp2040SlotMapping {
        page_index,
        slot_index,
        byte_offset_in_page,
    }
}

/// RP2040 backend implementation.
pub struct Rp2040Backend<const RP2040_FLASH_SIZE: usize = RP2040_DEFAULT_FLASH_SIZE> {
    #[cfg(all(not(test), target_arch = "arm"))]
    flash: RefCell<Flash<'static, FLASH, Blocking, RP2040_FLASH_SIZE>>,
    data_storage_start_address: usize,
    max_storage_slots: StorageIndex,
    page_buffer: RefCell<[u8; FLASH_PAGE_SIZE]>,
    #[cfg(any(test, not(target_arch = "arm")))]
    flash_mock: RefCell<MockFlash<RP2040_FLASH_SIZE>>,
}

impl<const RP2040_FLASH_SIZE: usize> Rp2040Backend<RP2040_FLASH_SIZE> {
    /// Creates a new RP2040 backend instance.
    ///
    /// Parameters:
    /// - `flash_peripheral`: RP2040 flash peripheral.
    /// - `data_storage_start_address`: first flash address reserved for block storage.
    ///
    /// Example:
    /// ```ignore
    /// use embassy_rp::init;
    /// use moonblokz_storage::backend_rp2040::Rp2040Backend;
    ///
    /// let peripherals = init(Default::default());
    /// let _backend = Rp2040Backend::<{ 2 * 1024 * 1024 }>::new(peripherals.FLASH, 256 * 1024);
    /// ```
    #[cfg(all(not(test), target_arch = "arm"))]
    pub fn new(
        flash_peripheral: impl Peripheral<P = FLASH> + 'static,
        data_storage_start_address: usize,
    ) -> Self {
        let max_storage_slots = Self::calculate_max_storage_slots(data_storage_start_address);

        Self {
            flash: RefCell::new(Flash::new_blocking(flash_peripheral)),
            data_storage_start_address,
            max_storage_slots,
            page_buffer: RefCell::new([0xFF; FLASH_PAGE_SIZE]),
        }
    }

    /// Creates a host/non-ARM RP2040 backend with an in-memory flash mock.
    ///
    /// Parameters:
    /// - `data_storage_start_address`: first flash address reserved for block storage.
    ///
    /// Example:
    /// ```ignore
    /// use moonblokz_storage::backend_rp2040::Rp2040Backend;
    ///
    /// let _backend = Rp2040Backend::<{ 2 * 4096 }>::new(0);
    /// ```
    #[cfg(not(target_arch = "arm"))]
    pub fn new(data_storage_start_address: usize) -> Self {
        let max_storage_slots = Self::calculate_max_storage_slots(data_storage_start_address);

        Self {
            data_storage_start_address,
            max_storage_slots,
            page_buffer: RefCell::new([0xFF; FLASH_PAGE_SIZE]),
            flash_mock: RefCell::new(MockFlash::new()),
        }
    }

    /// Creates a host-test RP2040 backend with an in-memory flash mock.
    ///
    /// Parameters:
    /// - `data_storage_start_address`: first flash address reserved for block storage.
    ///
    /// Example:
    /// ```ignore
    /// use moonblokz_storage::backend_rp2040::Rp2040Backend;
    ///
    /// let _backend = Rp2040Backend::<{ 2 * 4096 }>::new_for_tests(0);
    /// ```
    #[cfg(test)]
    pub fn new_for_tests(data_storage_start_address: usize) -> Self {
        let max_storage_slots = Self::calculate_max_storage_slots(data_storage_start_address);

        Self {
            data_storage_start_address,
            max_storage_slots,
            page_buffer: RefCell::new([0xFF; FLASH_PAGE_SIZE]),
            flash_mock: RefCell::new(MockFlash::new()),
        }
    }

    fn calculate_max_storage_slots(data_storage_start_address: usize) -> StorageIndex {
        let available_bytes = RP2040_FLASH_SIZE.saturating_sub(data_storage_start_address);
        let usable_pages = available_bytes / FLASH_PAGE_SIZE;
        (usable_pages * BLOCKS_PER_PAGE) as StorageIndex
    }

    fn page_flash_address(&self, mapping: &Rp2040SlotMapping) -> usize {
        self.data_storage_start_address + mapping.page_index as usize * FLASH_PAGE_SIZE
    }

    #[cfg(test)]
    fn slot_flash_address(&self, mapping: &Rp2040SlotMapping) -> usize {
        self.page_flash_address(mapping) + mapping.byte_offset_in_page
    }

    fn write_slot(&self, mapping: &Rp2040SlotMapping, block: &Block) -> Result<(), StorageError> {
        let page_address = self.page_flash_address(mapping);

        #[cfg(any(test, not(target_arch = "arm")))]
        {
            return self.write_slot_with_mock(page_address, mapping, block);
        }

        #[cfg(all(not(test), target_arch = "arm"))]
        {
            return self.write_slot_with_flash(page_address, mapping, block);
        }

        #[allow(unreachable_code)]
        Err(StorageError::BackendIo { code: 213 })
    }

    #[cfg(all(not(test), target_arch = "arm"))]
    fn write_slot_with_flash(
        &self,
        page_address: usize,
        mapping: &Rp2040SlotMapping,
        block: &Block,
    ) -> Result<(), StorageError> {
        let mut flash = self.flash.borrow_mut();
        let mut page_buffer = self.page_buffer.borrow_mut();
        flash
            .blocking_read(page_address as u32, &mut page_buffer[..])
            .map_err(|_| StorageError::BackendIo { code: 210 })?;

        Self::encode_block_to_slot(&mut page_buffer[..], mapping, block)?;

        flash
            .blocking_erase(page_address as u32, (page_address + FLASH_PAGE_SIZE) as u32)
            .map_err(|_| StorageError::BackendIo { code: 211 })?;

        flash
            .blocking_write(page_address as u32, &page_buffer[..])
            .map_err(|_| StorageError::BackendIo { code: 212 })?;

        Ok(())
    }

    #[cfg(any(test, not(target_arch = "arm")))]
    fn write_slot_with_mock(
        &self,
        page_address: usize,
        mapping: &Rp2040SlotMapping,
        block: &Block,
    ) -> Result<(), StorageError> {
        let mut flash_mock = self.flash_mock.borrow_mut();
        let mut page_buffer = self.page_buffer.borrow_mut();
        flash_mock
            .read(page_address as u32, &mut page_buffer[..])
            .map_err(|code| StorageError::BackendIo { code })?;

        Self::encode_block_to_slot(&mut page_buffer[..], mapping, block)?;

        flash_mock
            .erase(page_address as u32, (page_address + FLASH_PAGE_SIZE) as u32)
            .map_err(|code| StorageError::BackendIo { code })?;

        flash_mock
            .write(page_address as u32, &page_buffer[..])
            .map_err(|code| StorageError::BackendIo { code })?;

        Ok(())
    }

    fn encode_block_to_slot(
        page_buffer: &mut [u8],
        mapping: &Rp2040SlotMapping,
        block: &Block,
    ) -> Result<(), StorageError> {
        let slot_start = mapping.byte_offset_in_page;
        let slot_end = slot_start + SLOT_SIZE_BYTES;
        page_buffer[slot_start..slot_end].fill(0);

        let block_bytes = block.serialized_bytes();
        if block_bytes.len() > MAX_BLOCK_SIZE {
            return Err(StorageError::BackendIo { code: 213 });
        }
        let data_end = slot_start + block_bytes.len();
        page_buffer[slot_start..data_end].copy_from_slice(block_bytes);

        let hash_start = slot_start + SLOT_HASH_OFFSET;
        let hash_end = hash_start + HASH_SIZE;
        let computed_hash = calculate_hash(&page_buffer[slot_start..slot_start + MAX_BLOCK_SIZE]);
        page_buffer[hash_start..hash_end].copy_from_slice(&computed_hash);

        Ok(())
    }

    fn read_slot(&self, mapping: &Rp2040SlotMapping) -> Result<Block, StorageError> {
        let page_address = self.page_flash_address(mapping);

        #[cfg(any(test, not(target_arch = "arm")))]
        {
            return self.read_slot_with_mock(page_address, mapping);
        }

        #[cfg(all(not(test), target_arch = "arm"))]
        {
            return self.read_slot_with_flash(page_address, mapping);
        }

        #[allow(unreachable_code)]
        Err(StorageError::BackendIo { code: 220 })
    }

    #[cfg(any(test, not(target_arch = "arm")))]
    fn read_slot_with_mock(
        &self,
        page_address: usize,
        mapping: &Rp2040SlotMapping,
    ) -> Result<Block, StorageError> {
        let flash_mock = self.flash_mock.borrow();
        let mut page_buffer = self.page_buffer.borrow_mut();
        flash_mock
            .read(page_address as u32, &mut page_buffer[..])
            .map_err(|code| StorageError::BackendIo { code })?;

        let slot_start = mapping.byte_offset_in_page;
        let slot_end = slot_start + SLOT_SIZE_BYTES;
        Self::decode_slot_block(&page_buffer[slot_start..slot_end])
    }

    #[cfg(all(not(test), target_arch = "arm"))]
    fn read_slot_with_flash(
        &self,
        page_address: usize,
        mapping: &Rp2040SlotMapping,
    ) -> Result<Block, StorageError> {
        let mut flash = self.flash.borrow_mut();
        let mut page_buffer = self.page_buffer.borrow_mut();
        flash
            .blocking_read(page_address as u32, &mut page_buffer[..])
            .map_err(|_| StorageError::BackendIo { code: 220 })?;

        let slot_start = mapping.byte_offset_in_page;
        let slot_end = slot_start + SLOT_SIZE_BYTES;
        Self::decode_slot_block(&page_buffer[slot_start..slot_end])
    }

    fn decode_slot_block(slot_bytes: &[u8]) -> Result<Block, StorageError> {
        if slot_bytes.iter().all(|byte| *byte == 0xFF) {
            return Err(StorageError::BlockAbsent);
        }

        let mut stored_hash = [0u8; HASH_SIZE];
        stored_hash.copy_from_slice(&slot_bytes[SLOT_HASH_OFFSET..SLOT_HASH_OFFSET + HASH_SIZE]);

        let mut hash_buffer = [0u8; MAX_BLOCK_SIZE];
        hash_buffer.copy_from_slice(&slot_bytes[..MAX_BLOCK_SIZE]);
        let computed_hash = calculate_hash(&hash_buffer);
        if computed_hash != stored_hash {
            return Err(StorageError::IntegrityFailure);
        }

        Block::from_bytes(&slot_bytes[..MAX_BLOCK_SIZE]).map_err(|_| StorageError::IntegrityFailure)
    }

    #[cfg(test)]
    fn with_corrupted_mock_slot_byte(&self, storage_index: StorageIndex, byte_index: usize) {
        let mapping = map_storage_index(storage_index);
        let slot_start = self.slot_flash_address(&mapping);
        self.flash_mock.borrow_mut().data[slot_start + byte_index] ^= 0xFF;
    }

    #[cfg(test)]
    fn write_mock_slot_raw(&self, storage_index: StorageIndex, slot_bytes: &[u8; SLOT_SIZE_BYTES]) {
        let mapping = map_storage_index(storage_index);
        let slot_start = self.slot_flash_address(&mapping);
        let slot_end = slot_start + SLOT_SIZE_BYTES;
        self.flash_mock.borrow_mut().data[slot_start..slot_end].copy_from_slice(slot_bytes);
    }
}

impl<const RP2040_FLASH_SIZE: usize> StorageTrait for Rp2040Backend<RP2040_FLASH_SIZE> {
    fn init(
        &mut self,
        _private_key: [u8; PRIVATE_KEY_SIZE],
        _own_node_id: u32,
        _init_params: [u8; INIT_PARAMS_SIZE],
    ) -> Result<(), StorageError> {
        Ok(())
    }

    fn save_block(
        &mut self,
        storage_index: StorageIndex,
        block: &Block,
    ) -> Result<(), StorageError> {
        if storage_index >= self.max_storage_slots {
            return Err(StorageError::InvalidIndex);
        }

        let mapping = map_storage_index(storage_index);
        self.write_slot(&mapping, block)
    }

    fn read_block(&self, storage_index: StorageIndex) -> Result<Block, StorageError> {
        if storage_index >= self.max_storage_slots {
            return Err(StorageError::InvalidIndex);
        }

        let mapping = map_storage_index(storage_index);
        self.read_slot(&mapping)
    }

    fn set_chain_configuration(&mut self, _block: &Block) -> Result<(), StorageError> {
        Err(StorageError::BackendIo { code: 213 })
    }

    fn load_control_data(&mut self) -> Result<ControlPlaneData, StorageError> {
        Err(StorageError::BackendIo { code: 213 })
    }
}

#[cfg(any(test, not(target_arch = "arm")))]
struct MockFlash<const SIZE: usize> {
    data: [u8; SIZE],
}

#[cfg(any(test, not(target_arch = "arm")))]
impl<const SIZE: usize> MockFlash<SIZE> {
    fn new() -> Self {
        Self { data: [0xFF; SIZE] }
    }

    fn read(&self, from: u32, out: &mut [u8]) -> Result<(), u16> {
        let from_index = from as usize;
        let to_index = from_index + out.len();
        if to_index > SIZE {
            return Err(230);
        }
        out.copy_from_slice(&self.data[from_index..to_index]);
        Ok(())
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), u16> {
        let from_index = from as usize;
        let to_index = to as usize;
        if from_index > to_index || to_index > SIZE {
            return Err(231);
        }
        self.data[from_index..to_index].fill(0xFF);
        Ok(())
    }

    fn write(&mut self, from: u32, bytes: &[u8]) -> Result<(), u16> {
        let from_index = from as usize;
        let to_index = from_index + bytes.len();
        if to_index > SIZE {
            return Err(232);
        }
        self.data[from_index..to_index].copy_from_slice(bytes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_from_marker(marker: u8) -> Block {
        let mut bytes = [0u8; MAX_BLOCK_SIZE];
        bytes[0] = marker;

        let parse_result = Block::from_bytes(&bytes);
        assert!(parse_result.is_ok());
        match parse_result {
            Ok(value) => value,
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn new_calculates_max_slots_from_storage_geometry() {
        let backend = Rp2040Backend::<{ 3 * FLASH_PAGE_SIZE }>::new_for_tests(FLASH_PAGE_SIZE);
        assert_eq!(
            backend.max_storage_slots,
            (2 * BLOCKS_PER_PAGE) as StorageIndex
        );
    }

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
            (BLOCKS_PER_PAGE - 1) * SLOT_SIZE_BYTES
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
            expected_slot as usize * SLOT_SIZE_BYTES
        );
    }

    #[test]
    fn save_block_succeeds_for_valid_index() {
        let mut backend = Rp2040Backend::<{ 2 * FLASH_PAGE_SIZE }>::new_for_tests(0);
        let block = block_from_marker(7);

        let save_result = backend.save_block(1, &block);
        assert!(save_result.is_ok());

        let read_result = backend.read_block(1);
        assert!(read_result.is_ok());
        let read_block = match read_result {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(read_block.as_bytes(), block.as_bytes());
    }

    #[test]
    fn save_block_rejects_invalid_index() {
        let mut backend = Rp2040Backend::<FLASH_PAGE_SIZE>::new_for_tests(0);
        let block = block_from_marker(8);

        let save_result = backend.save_block(BLOCKS_PER_PAGE as StorageIndex, &block);
        assert!(matches!(save_result, Err(StorageError::InvalidIndex)));
    }

    #[test]
    fn save_block_succeeds_at_last_valid_index() {
        let mut backend = Rp2040Backend::<{ 8 * FLASH_PAGE_SIZE }>::new_for_tests(0);
        let block = block_from_marker(9);
        let last_valid_index = backend.max_storage_slots - 1;

        let save_result = backend.save_block(last_valid_index, &block);
        assert!(save_result.is_ok());
    }

    #[test]
    fn storage_start_address_reduces_capacity() {
        let backend = Rp2040Backend::<{ 3 * FLASH_PAGE_SIZE }>::new_for_tests(2 * FLASH_PAGE_SIZE);
        assert_eq!(backend.max_storage_slots, BLOCKS_PER_PAGE as StorageIndex);
        assert_eq!(backend.data_storage_start_address, 2 * FLASH_PAGE_SIZE);
    }

    #[test]
    fn slot_flash_address_uses_storage_start_address() {
        let backend = Rp2040Backend::<{ 4 * FLASH_PAGE_SIZE }>::new_for_tests(FLASH_PAGE_SIZE);
        let mapping = map_storage_index(BLOCKS_PER_PAGE as StorageIndex);
        let address = backend.slot_flash_address(&mapping);

        assert_eq!(address, 2 * FLASH_PAGE_SIZE);
    }

    #[test]
    fn read_block_reports_absent_for_empty_slot() {
        let backend = Rp2040Backend::<{ 2 * FLASH_PAGE_SIZE }>::new_for_tests(0);
        let read_result = backend.read_block(0);
        assert!(matches!(read_result, Err(StorageError::BlockAbsent)));
    }

    #[test]
    fn read_block_rejects_invalid_index() {
        let backend = Rp2040Backend::<FLASH_PAGE_SIZE>::new_for_tests(0);
        let read_result = backend.read_block(BLOCKS_PER_PAGE as StorageIndex);
        assert!(matches!(read_result, Err(StorageError::InvalidIndex)));
    }

    #[test]
    fn read_block_detects_hash_mismatch() {
        let mut backend = Rp2040Backend::<{ 2 * FLASH_PAGE_SIZE }>::new_for_tests(0);
        let block = block_from_marker(11);
        assert!(backend.save_block(0, &block).is_ok());

        // Corrupt first byte of stored hash in slot.
        let hash_byte_index = SLOT_HASH_OFFSET;
        backend.with_corrupted_mock_slot_byte(0, hash_byte_index);

        let read_result = backend.read_block(0);
        assert!(matches!(read_result, Err(StorageError::IntegrityFailure)));
    }

    #[test]
    fn read_block_succeeds_at_last_valid_index() {
        let mut backend = Rp2040Backend::<{ 8 * FLASH_PAGE_SIZE }>::new_for_tests(0);
        let block = block_from_marker(12);
        let last_valid_index = backend.max_storage_slots - 1;
        assert!(backend.save_block(last_valid_index, &block).is_ok());

        let read_result = backend.read_block(last_valid_index);
        assert!(read_result.is_ok());
    }

    #[test]
    fn read_block_detects_partially_written_slot_data() {
        let backend = Rp2040Backend::<{ 2 * FLASH_PAGE_SIZE }>::new_for_tests(0);
        let mut raw_slot = [0xFFu8; SLOT_SIZE_BYTES];
        raw_slot[0] = 1;
        raw_slot[1] = 2;
        backend.write_mock_slot_raw(0, &raw_slot);

        let read_result = backend.read_block(0);
        assert!(matches!(read_result, Err(StorageError::IntegrityFailure)));
    }

    #[test]
    fn read_block_detects_malformed_slot_with_matching_hash() {
        let backend = Rp2040Backend::<{ 2 * FLASH_PAGE_SIZE }>::new_for_tests(0);
        let mut raw_slot = [0u8; SLOT_SIZE_BYTES];
        let computed_hash = calculate_hash(&raw_slot[..MAX_BLOCK_SIZE]);
        raw_slot[SLOT_HASH_OFFSET..SLOT_HASH_OFFSET + HASH_SIZE].copy_from_slice(&computed_hash);
        backend.write_mock_slot_raw(0, &raw_slot);

        let read_result = backend.read_block(0);
        assert!(matches!(read_result, Err(StorageError::IntegrityFailure)));
    }

    #[test]
    fn startup_read_cycle_reports_typed_outcomes_for_mixed_slot_states() {
        let mut backend = Rp2040Backend::<{ 2 * FLASH_PAGE_SIZE }>::new_for_tests(0);
        let block = block_from_marker(13);
        assert!(backend.save_block(0, &block).is_ok());

        let mut partial_slot = [0xFFu8; SLOT_SIZE_BYTES];
        partial_slot[0] = 1;
        backend.write_mock_slot_raw(1, &partial_slot);

        let slot0 = backend.read_block(0);
        let slot1 = backend.read_block(1);
        let slot2 = backend.read_block(2);

        assert!(matches!(slot0, Ok(_)));
        assert!(matches!(slot1, Err(StorageError::IntegrityFailure)));
        assert!(matches!(slot2, Err(StorageError::BlockAbsent)));
    }

    #[test]
    fn integration_startup_ingest_query_flow_with_valid_dataset() {
        let mut backend = Rp2040Backend::<{ 4 * FLASH_PAGE_SIZE }>::new_for_tests(0);
        let block_a = block_from_marker(21);
        let block_b = block_from_marker(22);
        let block_c = block_from_marker(23);

        // Ingest flow: accepted blocks are saved at deterministic indices.
        assert!(backend.save_block(0, &block_a).is_ok());
        assert!(backend.save_block(2, &block_b).is_ok());
        assert!(backend.save_block(5, &block_c).is_ok());

        // Startup flow: chain logic reads a contiguous range and gets typed outcomes.
        let startup_scan = [
            backend.read_block(0),
            backend.read_block(1),
            backend.read_block(2),
            backend.read_block(3),
            backend.read_block(4),
            backend.read_block(5),
        ];

        assert!(matches!(startup_scan[0], Ok(_)));
        assert!(matches!(startup_scan[1], Err(StorageError::BlockAbsent)));
        assert!(matches!(startup_scan[2], Ok(_)));
        assert!(matches!(startup_scan[3], Err(StorageError::BlockAbsent)));
        assert!(matches!(startup_scan[4], Err(StorageError::BlockAbsent)));
        assert!(matches!(startup_scan[5], Ok(_)));

        // Query flow: retrieval returns exact stored blocks for populated indices.
        let read_a = backend.read_block(0);
        let read_b = backend.read_block(2);
        let read_c = backend.read_block(5);
        assert!(read_a.is_ok());
        assert!(read_b.is_ok());
        assert!(read_c.is_ok());
        let read_a = match read_a {
            Ok(value) => value,
            Err(_) => return,
        };
        let read_b = match read_b {
            Ok(value) => value,
            Err(_) => return,
        };
        let read_c = match read_c {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(read_a.as_bytes(), block_a.as_bytes());
        assert_eq!(read_b.as_bytes(), block_b.as_bytes());
        assert_eq!(read_c.as_bytes(), block_c.as_bytes());
    }

    #[test]
    fn integration_startup_and_query_flow_reports_integrity_on_corrupted_dataset() {
        let mut backend = Rp2040Backend::<{ 4 * FLASH_PAGE_SIZE }>::new_for_tests(0);
        let block_ok = block_from_marker(31);
        let block_corrupted = block_from_marker(32);

        // Ingest flow.
        assert!(backend.save_block(0, &block_ok).is_ok());
        assert!(backend.save_block(1, &block_corrupted).is_ok());

        // Corrupt one persisted slot and inject one partial slot.
        backend.with_corrupted_mock_slot_byte(1, SLOT_HASH_OFFSET);
        let mut partial_slot = [0xFFu8; SLOT_SIZE_BYTES];
        partial_slot[0] = 1;
        backend.write_mock_slot_raw(2, &partial_slot);

        // Startup flow over mixed health states.
        assert!(matches!(backend.read_block(0), Ok(_)));
        assert!(matches!(
            backend.read_block(1),
            Err(StorageError::IntegrityFailure)
        ));
        assert!(matches!(
            backend.read_block(2),
            Err(StorageError::IntegrityFailure)
        ));
        assert!(matches!(
            backend.read_block(3),
            Err(StorageError::BlockAbsent)
        ));

        // Query flow: contract consistency for invalid index remains intact.
        let invalid_index = backend.max_storage_slots;
        assert!(matches!(
            backend.read_block(invalid_index),
            Err(StorageError::InvalidIndex)
        ));
    }
}
