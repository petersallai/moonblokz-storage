/*! RP2040 backend geometry mapping and synchronous flash save/retrieve paths. */

use crate::{
    CONTROL_PLANE_COUNT, CONTROL_PLANE_VERSION, ControlPlaneData, INIT_PARAMS_SIZE, StorageError,
    StorageIndex, StorageTrait,
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
/// Reserved control-plane bytes (one full page per replica).
const CONTROL_PLANE_RESERVED_BYTES: usize = CONTROL_PLANE_COUNT * FLASH_PAGE_SIZE;

const CP_VERSION_OFFSET: usize = 0;
const CP_PRIVATE_KEY_SIZE_OFFSET: usize = CP_VERSION_OFFSET + 1;
const CP_PRIVATE_KEY_OFFSET: usize = CP_PRIVATE_KEY_SIZE_OFFSET + 1;
const CP_OWN_NODE_ID_OFFSET: usize = CP_PRIVATE_KEY_OFFSET + PRIVATE_KEY_SIZE;
const CP_INIT_PARAMS_SIZE_OFFSET: usize = CP_OWN_NODE_ID_OFFSET + 4;
const CP_INIT_PARAMS_OFFSET: usize = CP_INIT_PARAMS_SIZE_OFFSET + 1;
const CP_MAX_BLOCK_SIZE_OFFSET: usize = CP_INIT_PARAMS_OFFSET + INIT_PARAMS_SIZE;
const CP_CHAIN_CONFIG_OFFSET: usize = CP_MAX_BLOCK_SIZE_OFFSET + 2;
const CP_CRC32_OFFSET: usize = CP_CHAIN_CONFIG_OFFSET + MAX_BLOCK_SIZE;
const CONTROL_PLANE_ENTRY_SIZE: usize = CP_CRC32_OFFSET + 4;

/// Number of block slots per RP2040 flash page.
pub const BLOCKS_PER_PAGE: usize = FLASH_PAGE_SIZE / SLOT_SIZE_BYTES;

/// Number of block slots per page in storage-index type space.
pub const BLOCKS_PER_PAGE_INDEX: StorageIndex = BLOCKS_PER_PAGE as StorageIndex;

// Compile-time geometry guard.
const _: () = {
    if BLOCKS_PER_PAGE == 0 {
        panic!("MAX_BLOCK_SIZE must allow at least one block in a 4096-byte RP2040 page");
    }
    if CONTROL_PLANE_ENTRY_SIZE > FLASH_PAGE_SIZE {
        panic!("control-plane entry must fit in one RP2040 flash page");
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
    fn validate_page_aligned_start_address(
        data_storage_start_address: usize,
    ) -> Result<(), StorageError> {
        if data_storage_start_address % FLASH_PAGE_SIZE != 0 {
            return Err(StorageError::InvalidConfiguration);
        }
        Ok(())
    }

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
    ) -> Result<Self, StorageError> {
        Self::validate_page_aligned_start_address(data_storage_start_address)?;
        let max_storage_slots = Self::calculate_max_storage_slots(data_storage_start_address);

        Ok(Self {
            flash: RefCell::new(Flash::new_blocking(flash_peripheral)),
            data_storage_start_address,
            max_storage_slots,
            page_buffer: RefCell::new([0xFF; FLASH_PAGE_SIZE]),
        })
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
    pub fn new(data_storage_start_address: usize) -> Result<Self, StorageError> {
        Self::validate_page_aligned_start_address(data_storage_start_address)?;
        let max_storage_slots = Self::calculate_max_storage_slots(data_storage_start_address);

        Ok(Self {
            data_storage_start_address,
            max_storage_slots,
            page_buffer: RefCell::new([0xFF; FLASH_PAGE_SIZE]),
            flash_mock: RefCell::new(MockFlash::new()),
        })
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
    /// let _backend = Rp2040Backend::<{ 2 * 4096 }>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
    /// ```
    #[cfg(test)]
    pub fn new_for_tests(data_storage_start_address: usize) -> Result<Self, StorageError> {
        Self::validate_page_aligned_start_address(data_storage_start_address)?;
        let max_storage_slots = Self::calculate_max_storage_slots(data_storage_start_address);

        Ok(Self {
            data_storage_start_address,
            max_storage_slots,
            page_buffer: RefCell::new([0xFF; FLASH_PAGE_SIZE]),
            flash_mock: RefCell::new(MockFlash::new()),
        })
    }

    fn calculate_max_storage_slots(data_storage_start_address: usize) -> StorageIndex {
        let available_bytes = RP2040_FLASH_SIZE.saturating_sub(data_storage_start_address);
        let block_storage_bytes = available_bytes.saturating_sub(CONTROL_PLANE_RESERVED_BYTES);
        let usable_pages = block_storage_bytes / FLASH_PAGE_SIZE;
        (usable_pages * BLOCKS_PER_PAGE) as StorageIndex
    }

    fn page_flash_address(&self, mapping: &Rp2040SlotMapping) -> usize {
        self.data_storage_start_address
            + CONTROL_PLANE_RESERVED_BYTES
            + mapping.page_index as usize * FLASH_PAGE_SIZE
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

    fn control_plane_page_address(&self, replica_index: usize) -> usize {
        self.data_storage_start_address + replica_index * FLASH_PAGE_SIZE
    }

    fn read_page(&self, page_address: usize, out: &mut [u8; FLASH_PAGE_SIZE]) -> Result<(), StorageError> {
        #[cfg(any(test, not(target_arch = "arm")))]
        {
            let flash_mock = self.flash_mock.borrow();
            return flash_mock
                .read(page_address as u32, out)
                .map_err(|code| StorageError::BackendIo { code });
        }

        #[cfg(all(not(test), target_arch = "arm"))]
        {
            let mut flash = self.flash.borrow_mut();
            return flash
                .blocking_read(page_address as u32, out)
                .map_err(|_| StorageError::BackendIo { code: 210 });
        }

        #[allow(unreachable_code)]
        Err(StorageError::BackendIo { code: 210 })
    }

    fn erase_page(&self, page_address: usize) -> Result<(), StorageError> {
        let page_end = page_address + FLASH_PAGE_SIZE;
        #[cfg(any(test, not(target_arch = "arm")))]
        {
            let mut flash_mock = self.flash_mock.borrow_mut();
            return flash_mock
                .erase(page_address as u32, page_end as u32)
                .map_err(|code| StorageError::BackendIo { code });
        }

        #[cfg(all(not(test), target_arch = "arm"))]
        {
            let mut flash = self.flash.borrow_mut();
            return flash
                .blocking_erase(page_address as u32, page_end as u32)
                .map_err(|_| StorageError::BackendIo { code: 211 });
        }

        #[allow(unreachable_code)]
        Err(StorageError::BackendIo { code: 211 })
    }

    fn write_page(&self, page_address: usize, page: &[u8; FLASH_PAGE_SIZE]) -> Result<(), StorageError> {
        #[cfg(any(test, not(target_arch = "arm")))]
        {
            let mut flash_mock = self.flash_mock.borrow_mut();
            return flash_mock
                .write(page_address as u32, page)
                .map_err(|code| StorageError::BackendIo { code });
        }

        #[cfg(all(not(test), target_arch = "arm"))]
        {
            let mut flash = self.flash.borrow_mut();
            return flash
                .blocking_write(page_address as u32, page)
                .map_err(|_| StorageError::BackendIo { code: 212 });
        }

        #[allow(unreachable_code)]
        Err(StorageError::BackendIo { code: 212 })
    }

    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        let mut i = 0usize;
        while i < bytes.len() {
            crc ^= bytes[i] as u32;
            let mut bit = 0usize;
            while bit < 8 {
                if (crc & 1) != 0 {
                    crc = (crc >> 1) ^ 0xEDB8_8320;
                } else {
                    crc >>= 1;
                }
                bit += 1;
            }
            i += 1;
        }
        !crc
    }

    fn serialize_control_record(record: &ControlPlaneData) -> [u8; CONTROL_PLANE_ENTRY_SIZE] {
        let mut out = [0u8; CONTROL_PLANE_ENTRY_SIZE];
        out[CP_VERSION_OFFSET] = CONTROL_PLANE_VERSION;
        out[CP_PRIVATE_KEY_SIZE_OFFSET] = PRIVATE_KEY_SIZE as u8;
        out[CP_PRIVATE_KEY_OFFSET..CP_PRIVATE_KEY_OFFSET + PRIVATE_KEY_SIZE]
            .copy_from_slice(&record.private_key);
        out[CP_OWN_NODE_ID_OFFSET..CP_OWN_NODE_ID_OFFSET + 4]
            .copy_from_slice(&record.own_node_id.to_le_bytes());
        out[CP_INIT_PARAMS_SIZE_OFFSET] = INIT_PARAMS_SIZE as u8;
        out[CP_INIT_PARAMS_OFFSET..CP_INIT_PARAMS_OFFSET + INIT_PARAMS_SIZE]
            .copy_from_slice(&record.init_params);

        let max_block_size = MAX_BLOCK_SIZE as u16;
        out[CP_MAX_BLOCK_SIZE_OFFSET..CP_MAX_BLOCK_SIZE_OFFSET + 2]
            .copy_from_slice(&max_block_size.to_le_bytes());

        if let Some(chain_configuration) = &record.chain_configuration {
            let bytes = chain_configuration.as_bytes();
            out[CP_CHAIN_CONFIG_OFFSET..CP_CHAIN_CONFIG_OFFSET + bytes.len()].copy_from_slice(bytes);
        }

        let crc = Self::crc32(&out[..CP_CRC32_OFFSET]);
        out[CP_CRC32_OFFSET..CP_CRC32_OFFSET + 4].copy_from_slice(&crc.to_le_bytes());
        out
    }

    fn deserialize_control_record(
        bytes: &[u8; CONTROL_PLANE_ENTRY_SIZE],
    ) -> Result<ControlPlaneData, StorageError> {
        let all_zero = bytes.iter().all(|v| *v == 0);
        let all_ff = bytes.iter().all(|v| *v == 0xFF);
        if all_zero || all_ff {
            return Err(StorageError::ControlPlaneUninitialized);
        }

        let mut crc_bytes = [0u8; 4];
        crc_bytes.copy_from_slice(&bytes[CP_CRC32_OFFSET..CP_CRC32_OFFSET + 4]);
        let stored_crc = u32::from_le_bytes(crc_bytes);
        let computed_crc = Self::crc32(&bytes[..CP_CRC32_OFFSET]);
        if stored_crc != computed_crc {
            return Err(StorageError::ControlPlaneCorrupted);
        }

        if bytes[CP_VERSION_OFFSET] != CONTROL_PLANE_VERSION
            || bytes[CP_PRIVATE_KEY_SIZE_OFFSET] as usize != PRIVATE_KEY_SIZE
            || bytes[CP_INIT_PARAMS_SIZE_OFFSET] as usize != INIT_PARAMS_SIZE
        {
            return Err(StorageError::ControlPlaneIncompatible);
        }

        let mut max_block_size_bytes = [0u8; 2];
        max_block_size_bytes
            .copy_from_slice(&bytes[CP_MAX_BLOCK_SIZE_OFFSET..CP_MAX_BLOCK_SIZE_OFFSET + 2]);
        let persisted_max_block_size = u16::from_le_bytes(max_block_size_bytes) as usize;
        if persisted_max_block_size != MAX_BLOCK_SIZE {
            return Err(StorageError::ControlPlaneIncompatible);
        }

        let mut private_key = [0u8; PRIVATE_KEY_SIZE];
        private_key.copy_from_slice(&bytes[CP_PRIVATE_KEY_OFFSET..CP_PRIVATE_KEY_OFFSET + PRIVATE_KEY_SIZE]);

        let mut own_node_id_bytes = [0u8; 4];
        own_node_id_bytes.copy_from_slice(&bytes[CP_OWN_NODE_ID_OFFSET..CP_OWN_NODE_ID_OFFSET + 4]);
        let own_node_id = u32::from_le_bytes(own_node_id_bytes);

        let mut init_params = [0u8; INIT_PARAMS_SIZE];
        init_params.copy_from_slice(&bytes[CP_INIT_PARAMS_OFFSET..CP_INIT_PARAMS_OFFSET + INIT_PARAMS_SIZE]);

        let chain_configuration = if bytes[CP_CHAIN_CONFIG_OFFSET] == 0 {
            None
        } else {
            let mut cfg = [0u8; MAX_BLOCK_SIZE];
            cfg.copy_from_slice(&bytes[CP_CHAIN_CONFIG_OFFSET..CP_CHAIN_CONFIG_OFFSET + MAX_BLOCK_SIZE]);
            Some(Block::from_bytes(&cfg).map_err(|_| StorageError::ControlPlaneCorrupted)?)
        };

        Ok(ControlPlaneData {
            private_key,
            own_node_id,
            init_params,
            chain_configuration,
        })
    }

    fn read_control_record_from_replica(&self, replica_index: usize) -> Result<ControlPlaneData, StorageError> {
        let page_address = self.control_plane_page_address(replica_index);
        let mut page = [0u8; FLASH_PAGE_SIZE];
        self.read_page(page_address, &mut page)?;

        let mut entry = [0u8; CONTROL_PLANE_ENTRY_SIZE];
        entry.copy_from_slice(&page[..CONTROL_PLANE_ENTRY_SIZE]);
        Self::deserialize_control_record(&entry)
    }

    fn write_control_record_to_replica(
        &self,
        replica_index: usize,
        record: &ControlPlaneData,
    ) -> Result<(), StorageError> {
        let page_address = self.control_plane_page_address(replica_index);
        let mut page = [0u8; FLASH_PAGE_SIZE];
        page.fill(0);
        let encoded = Self::serialize_control_record(record);
        page[..CONTROL_PLANE_ENTRY_SIZE].copy_from_slice(&encoded);
        self.erase_page(page_address)?;
        self.write_page(page_address, &page)
    }

    fn load_primary_control_record_and_repair(&self) -> Result<ControlPlaneData, StorageError> {
        let mut first_valid_record: Option<ControlPlaneData> = None;
        let mut first_valid_index: Option<usize> = None;
        let mut invalid = [usize::MAX; CONTROL_PLANE_COUNT];
        let mut invalid_len = 0usize;
        let mut saw_non_uninitialized = false;
        let mut saw_incompatible = false;

        let mut i = 0usize;
        while i < CONTROL_PLANE_COUNT {
            match self.read_control_record_from_replica(i) {
                Ok(record) => {
                    if first_valid_record.is_none() {
                        first_valid_index = Some(i);
                        first_valid_record = Some(record);
                    }
                }
                Err(StorageError::ControlPlaneUninitialized) => {
                    invalid[invalid_len] = i;
                    invalid_len += 1;
                }
                Err(StorageError::ControlPlaneIncompatible) => {
                    saw_non_uninitialized = true;
                    saw_incompatible = true;
                    invalid[invalid_len] = i;
                    invalid_len += 1;
                }
                Err(StorageError::ControlPlaneCorrupted) => {
                    saw_non_uninitialized = true;
                    invalid[invalid_len] = i;
                    invalid_len += 1;
                }
                Err(err) => return Err(err),
            }
            i += 1;
        }

        let record = match first_valid_record {
            Some(value) => value,
            None => {
                if saw_incompatible {
                    return Err(StorageError::ControlPlaneIncompatible);
                }
                if saw_non_uninitialized {
                    return Err(StorageError::ControlPlaneCorrupted);
                }
                return Err(StorageError::ControlPlaneUninitialized);
            }
        };

        let mut j = 0usize;
        while j < invalid_len {
            let target = invalid[j];
            if Some(target) != first_valid_index {
                self.write_control_record_to_replica(target, &record)?;
            }
            j += 1;
        }

        Ok(record)
    }
}

impl<const RP2040_FLASH_SIZE: usize> StorageTrait for Rp2040Backend<RP2040_FLASH_SIZE> {
    fn init(
        &mut self,
        private_key: [u8; PRIVATE_KEY_SIZE],
        own_node_id: u32,
        init_params: [u8; INIT_PARAMS_SIZE],
    ) -> Result<(), StorageError> {
        Self::validate_page_aligned_start_address(self.data_storage_start_address)?;
        let first_page = self.data_storage_start_address / FLASH_PAGE_SIZE;
        let page_count = (RP2040_FLASH_SIZE.saturating_sub(self.data_storage_start_address)) / FLASH_PAGE_SIZE;
        let mut page = 0usize;
        while page < page_count {
            let page_address = (first_page + page) * FLASH_PAGE_SIZE;
            self.erase_page(page_address)?;
            page += 1;
        }

        let record = ControlPlaneData {
            private_key,
            own_node_id,
            init_params,
            chain_configuration: None,
        };

        let mut replica_index = 0usize;
        while replica_index < CONTROL_PLANE_COUNT {
            self.write_control_record_to_replica(replica_index, &record)?;
            replica_index += 1;
        }

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

    fn set_chain_configuration(&mut self, block: &Block) -> Result<(), StorageError> {
        let mut record = self.load_primary_control_record_and_repair()?;
        if record.chain_configuration.is_some() {
            return Err(StorageError::ChainConfigurationAlreadySet);
        }

        record.chain_configuration = Some(
            Block::from_bytes(block.as_bytes()).map_err(|_| StorageError::BackendIo { code: 213 })?,
        );

        let mut replica_index = 0usize;
        while replica_index < CONTROL_PLANE_COUNT {
            self.write_control_record_to_replica(replica_index, &record)?;
            replica_index += 1;
        }

        Ok(())
    }

    fn load_control_data(&mut self) -> Result<ControlPlaneData, StorageError> {
        let record = self.load_primary_control_record_and_repair()?;
        Ok(record)
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
    use crate::CONTROL_PLANE_COUNT;

    const TEST_FLASH_ONE_BLOCK_PAGE: usize = (CONTROL_PLANE_COUNT + 1) * FLASH_PAGE_SIZE;
    const TEST_FLASH_TWO_BLOCK_PAGES: usize = (CONTROL_PLANE_COUNT + 2) * FLASH_PAGE_SIZE;
    const TEST_FLASH_THREE_BLOCK_PAGES: usize = (CONTROL_PLANE_COUNT + 3) * FLASH_PAGE_SIZE;
    const TEST_FLASH_FOUR_BLOCK_PAGES: usize = (CONTROL_PLANE_COUNT + 4) * FLASH_PAGE_SIZE;
    const TEST_FLASH_EIGHT_BLOCK_PAGES: usize = (CONTROL_PLANE_COUNT + 8) * FLASH_PAGE_SIZE;

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
        let backend = Rp2040Backend::<TEST_FLASH_THREE_BLOCK_PAGES>::new_for_tests(FLASH_PAGE_SIZE).unwrap_or_else(|_| unreachable!());
        assert_eq!(
            backend.max_storage_slots,
            (2 * BLOCKS_PER_PAGE) as StorageIndex
        );
    }

    #[test]
    fn new_for_tests_returns_error_on_misaligned_start_address() {
        let backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(1);
        assert!(matches!(backend, Err(StorageError::InvalidConfiguration)));
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
        let mut backend = Rp2040Backend::<TEST_FLASH_TWO_BLOCK_PAGES>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
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
        let mut backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        let block = block_from_marker(8);

        let save_result = backend.save_block(BLOCKS_PER_PAGE as StorageIndex, &block);
        assert!(matches!(save_result, Err(StorageError::InvalidIndex)));
    }

    #[test]
    fn save_block_succeeds_at_last_valid_index() {
        let mut backend = Rp2040Backend::<TEST_FLASH_EIGHT_BLOCK_PAGES>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        let block = block_from_marker(9);
        let last_valid_index = backend.max_storage_slots - 1;

        let save_result = backend.save_block(last_valid_index, &block);
        assert!(save_result.is_ok());
    }

    #[test]
    fn storage_start_address_reduces_capacity() {
        let backend = Rp2040Backend::<TEST_FLASH_THREE_BLOCK_PAGES>::new_for_tests(2 * FLASH_PAGE_SIZE).unwrap_or_else(|_| unreachable!());
        assert_eq!(backend.max_storage_slots, BLOCKS_PER_PAGE as StorageIndex);
        assert_eq!(backend.data_storage_start_address, 2 * FLASH_PAGE_SIZE);
    }

    #[test]
    fn slot_flash_address_uses_storage_start_address() {
        let backend = Rp2040Backend::<TEST_FLASH_FOUR_BLOCK_PAGES>::new_for_tests(FLASH_PAGE_SIZE).unwrap_or_else(|_| unreachable!());
        let mapping = map_storage_index(BLOCKS_PER_PAGE as StorageIndex);
        let address = backend.slot_flash_address(&mapping);

        assert_eq!(address, (CONTROL_PLANE_COUNT + 2) * FLASH_PAGE_SIZE);
    }

    #[test]
    fn read_block_reports_absent_for_empty_slot() {
        let backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        let read_result = backend.read_block(0);
        assert!(matches!(read_result, Err(StorageError::BlockAbsent)));
    }

    #[test]
    fn read_block_rejects_invalid_index() {
        let backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        let read_result = backend.read_block(BLOCKS_PER_PAGE as StorageIndex);
        assert!(matches!(read_result, Err(StorageError::InvalidIndex)));
    }

    #[test]
    fn read_block_detects_hash_mismatch() {
        let mut backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
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
        let mut backend = Rp2040Backend::<TEST_FLASH_EIGHT_BLOCK_PAGES>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        let block = block_from_marker(12);
        let last_valid_index = backend.max_storage_slots - 1;
        assert!(backend.save_block(last_valid_index, &block).is_ok());

        let read_result = backend.read_block(last_valid_index);
        assert!(read_result.is_ok());
    }

    #[test]
    fn read_block_detects_partially_written_slot_data() {
        let backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        let mut raw_slot = [0xFFu8; SLOT_SIZE_BYTES];
        raw_slot[0] = 1;
        raw_slot[1] = 2;
        backend.write_mock_slot_raw(0, &raw_slot);

        let read_result = backend.read_block(0);
        assert!(matches!(read_result, Err(StorageError::IntegrityFailure)));
    }

    #[test]
    fn read_block_detects_malformed_slot_with_matching_hash() {
        let backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        let mut raw_slot = [0u8; SLOT_SIZE_BYTES];
        let computed_hash = calculate_hash(&raw_slot[..MAX_BLOCK_SIZE]);
        raw_slot[SLOT_HASH_OFFSET..SLOT_HASH_OFFSET + HASH_SIZE].copy_from_slice(&computed_hash);
        backend.write_mock_slot_raw(0, &raw_slot);

        let read_result = backend.read_block(0);
        assert!(matches!(read_result, Err(StorageError::IntegrityFailure)));
    }

    #[test]
    fn startup_read_cycle_reports_typed_outcomes_for_mixed_slot_states() {
        let mut backend = Rp2040Backend::<TEST_FLASH_TWO_BLOCK_PAGES>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
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
        let mut backend = Rp2040Backend::<TEST_FLASH_THREE_BLOCK_PAGES>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
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
        let mut backend = Rp2040Backend::<TEST_FLASH_TWO_BLOCK_PAGES>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
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

    #[test]
    fn control_plane_load_reports_uninitialized_before_init() {
        let mut backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        assert!(matches!(
            backend.load_control_data(),
            Err(StorageError::ControlPlaneUninitialized)
        ));
    }

    #[test]
    fn control_plane_init_and_load_round_trip() {
        let mut backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        let private_key = [7u8; PRIVATE_KEY_SIZE];
        let init_params = [9u8; INIT_PARAMS_SIZE];
        assert!(backend.init(private_key, 42, init_params).is_ok());

        let loaded = backend.load_control_data();
        assert!(loaded.is_ok());
        let loaded = match loaded {
            Ok(value) => value,
            Err(_) => return,
        };

        assert_eq!(loaded.private_key, private_key);
        assert_eq!(loaded.own_node_id, 42);
        assert_eq!(loaded.init_params, init_params);
        assert!(loaded.chain_configuration.is_none());
    }

    #[test]
    fn control_plane_set_chain_configuration_is_set_once() {
        let mut backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        assert!(backend
            .init([1u8; PRIVATE_KEY_SIZE], 1, [2u8; INIT_PARAMS_SIZE])
            .is_ok());
        let cfg_block = block_from_marker(55);

        assert!(backend.set_chain_configuration(&cfg_block).is_ok());
        assert!(matches!(
            backend.set_chain_configuration(&cfg_block),
            Err(StorageError::ChainConfigurationAlreadySet)
        ));

        let loaded = backend.load_control_data();
        assert!(loaded.is_ok());
        let loaded = match loaded {
            Ok(value) => value,
            Err(_) => return,
        };
        assert!(loaded.chain_configuration.is_some());
    }

    #[test]
    fn control_plane_load_repairs_corrupted_replica() {
        let mut backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        assert!(backend
            .init([1u8; PRIVATE_KEY_SIZE], 2, [3u8; INIT_PARAMS_SIZE])
            .is_ok());

        // Corrupt first replica header byte.
        let replica0_addr = backend.control_plane_page_address(0);
        backend.flash_mock.borrow_mut().data[replica0_addr] ^= 0xFF;

        assert!(backend.load_control_data().is_ok());

        // Re-read replica page and check CRC-valid deserialization.
        let mut repaired_page = [0u8; FLASH_PAGE_SIZE];
        assert!(backend.read_page(replica0_addr, &mut repaired_page).is_ok());
        let mut repaired_entry = [0u8; CONTROL_PLANE_ENTRY_SIZE];
        repaired_entry.copy_from_slice(&repaired_page[..CONTROL_PLANE_ENTRY_SIZE]);
        let repaired = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::deserialize_control_record(&repaired_entry);
        assert!(repaired.is_ok());
    }

    #[test]
    fn init_returns_error_on_misaligned_start_address() {
        let mut backend = Rp2040Backend::<TEST_FLASH_ONE_BLOCK_PAGE>::new_for_tests(0).unwrap_or_else(|_| unreachable!());
        backend.data_storage_start_address = 1;
        let init_result = backend.init([1u8; PRIVATE_KEY_SIZE], 1, [0u8; INIT_PARAMS_SIZE]);
        assert!(matches!(init_result, Err(StorageError::InvalidConfiguration)));
    }
}
