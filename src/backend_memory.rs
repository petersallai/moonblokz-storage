/*! In-memory backend module for MoonBlokz storage contract testing/integration. */

use crate::{
    CONTROL_PLANE_COUNT, CONTROL_PLANE_VERSION, ControlPlaneData, INIT_PARAMS_SIZE, StorageError,
    StorageIndex, StorageTrait,
};
use moonblokz_chain_types::{Block, MAX_BLOCK_SIZE};
use moonblokz_crypto::PRIVATE_KEY_SIZE;

const VERSION_OFFSET: usize = 0;
const PRIVATE_KEY_SIZE_OFFSET: usize = VERSION_OFFSET + 1;
const PRIVATE_KEY_OFFSET: usize = PRIVATE_KEY_SIZE_OFFSET + 1;
const OWN_NODE_ID_OFFSET: usize = PRIVATE_KEY_OFFSET + PRIVATE_KEY_SIZE;
const INIT_PARAMS_SIZE_OFFSET: usize = OWN_NODE_ID_OFFSET + 4;
const INIT_PARAMS_OFFSET: usize = INIT_PARAMS_SIZE_OFFSET + 1;
const MAX_BLOCK_SIZE_OFFSET: usize = INIT_PARAMS_OFFSET + INIT_PARAMS_SIZE;
const CHAIN_CONFIG_OFFSET: usize = MAX_BLOCK_SIZE_OFFSET + 2;
const CONTROL_CRC32_OFFSET: usize = CHAIN_CONFIG_OFFSET + MAX_BLOCK_SIZE;
const CONTROL_PLANE_ENTRY_SIZE: usize = CONTROL_CRC32_OFFSET + 4;
const CONTROL_PLANE_RESERVED_BYTES: usize = CONTROL_PLANE_COUNT * CONTROL_PLANE_ENTRY_SIZE;

/// In-memory backend with compile-time byte capacity.
///
/// Capacity rule:
/// - Control-plane uses the first `CONTROL_PLANE_COUNT * CONTROL_PLANE_ENTRY_SIZE` bytes.
/// - Effective block slots are `(STORAGE_SIZE - control_plane_reserved_bytes) / MAX_BLOCK_SIZE`.
/// - Remainder bytes are intentionally unused.
/// - Empty slot marker is `slot[0] == 0` (block version byte is zero).
///
/// Startup read-cycle example:
/// ```
/// use moonblokz_chain_types::{Block, HEADER_SIZE, MAX_BLOCK_SIZE};
/// use moonblokz_storage::{
///     INIT_PARAMS_SIZE, MemoryBackend, StorageError, StorageTrait,
/// };
/// use moonblokz_crypto::PRIVATE_KEY_SIZE;
///
/// let mut backend = MemoryBackend::<{ 4 * MAX_BLOCK_SIZE + 8000 }>::new();
/// let init_result = backend.init([1u8; PRIVATE_KEY_SIZE], 7, [0u8; INIT_PARAMS_SIZE]);
/// assert!(init_result.is_ok());
///
/// let mut bytes = [0u8; HEADER_SIZE];
/// bytes[0] = 1;
/// let block_result = Block::from_bytes(&bytes);
/// assert!(block_result.is_ok());
/// let block = match block_result {
///     Ok(value) => value,
///     Err(_) => return,
/// };
/// assert!(backend.save_block(1, &block).is_ok());
///
/// for storage_index in 0u32..4u32 {
///     match backend.read_block(storage_index) {
///         Ok(_) => { /* populated slot */ }
///         Err(StorageError::BlockAbsent) => { /* empty slot */ }
///         Err(StorageError::InvalidIndex) => { /* out-of-range, stop or skip */ }
///         Err(StorageError::IntegrityFailure) => { /* reserved for integrity stories */ }
///         Err(StorageError::ControlPlaneUninitialized) => { /* init missing */ }
///         Err(StorageError::ChainConfigurationAlreadySet) => { /* not used in read */ }
///         Err(StorageError::ControlPlaneCorrupted) => { /* control-plane issue */ }
///         Err(StorageError::ControlPlaneIncompatible) => { /* control-plane incompatibility */ }
///         Err(StorageError::BackendIo { .. }) => { /* backend error */ }
///     }
/// }
/// ```
pub struct MemoryBackend<const STORAGE_SIZE: usize> {
    storage: [u8; STORAGE_SIZE],
}

struct ControlPlaneRecord {
    private_key: [u8; PRIVATE_KEY_SIZE],
    own_node_id: u32,
    init_params: [u8; INIT_PARAMS_SIZE],
    chain_configuration: Option<[u8; MAX_BLOCK_SIZE]>,
}

impl<const STORAGE_SIZE: usize> MemoryBackend<STORAGE_SIZE> {
    const MAX_STORAGE_SLOTS: StorageIndex = if STORAGE_SIZE > CONTROL_PLANE_RESERVED_BYTES {
        ((STORAGE_SIZE - CONTROL_PLANE_RESERVED_BYTES) / MAX_BLOCK_SIZE) as StorageIndex
    } else {
        0
    };

    /// Creates a new memory backend.
    ///
    /// Parameters:
    /// - none.
    ///
    /// Example:
    /// ```
    /// use moonblokz_chain_types::MAX_BLOCK_SIZE;
    /// use moonblokz_storage::backend_memory::MemoryBackend;
    ///
    /// let _backend = MemoryBackend::<{ 8 * MAX_BLOCK_SIZE + 8000 }>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            storage: [0u8; STORAGE_SIZE],
        }
    }

    fn slot_range(storage_index: StorageIndex) -> Result<(usize, usize), StorageError> {
        if storage_index >= Self::MAX_STORAGE_SLOTS {
            return Err(StorageError::InvalidIndex);
        }

        let slot_start = CONTROL_PLANE_RESERVED_BYTES + storage_index as usize * MAX_BLOCK_SIZE;
        let slot_end = slot_start + MAX_BLOCK_SIZE;
        Ok((slot_start, slot_end))
    }

    fn control_plane_entry_offset(replica_index: usize) -> usize {
        replica_index * CONTROL_PLANE_ENTRY_SIZE
    }

    fn read_control_plane_entry(&self, replica_index: usize) -> [u8; CONTROL_PLANE_ENTRY_SIZE] {
        let start = Self::control_plane_entry_offset(replica_index);
        let end = start + CONTROL_PLANE_ENTRY_SIZE;
        let mut out = [0u8; CONTROL_PLANE_ENTRY_SIZE];
        out.copy_from_slice(&self.storage[start..end]);
        out
    }

    fn write_control_plane_entry(
        &mut self,
        replica_index: usize,
        entry: &[u8; CONTROL_PLANE_ENTRY_SIZE],
    ) {
        let start = Self::control_plane_entry_offset(replica_index);
        let end = start + CONTROL_PLANE_ENTRY_SIZE;
        self.storage[start..end].copy_from_slice(entry);
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

    fn serialize_record(record: &ControlPlaneRecord) -> [u8; CONTROL_PLANE_ENTRY_SIZE] {
        let mut out = [0u8; CONTROL_PLANE_ENTRY_SIZE];
        out[VERSION_OFFSET] = CONTROL_PLANE_VERSION;
        out[PRIVATE_KEY_SIZE_OFFSET] = PRIVATE_KEY_SIZE as u8;
        out[PRIVATE_KEY_OFFSET..PRIVATE_KEY_OFFSET + PRIVATE_KEY_SIZE]
            .copy_from_slice(&record.private_key);
        out[OWN_NODE_ID_OFFSET..OWN_NODE_ID_OFFSET + 4]
            .copy_from_slice(&record.own_node_id.to_le_bytes());
        out[INIT_PARAMS_SIZE_OFFSET] = INIT_PARAMS_SIZE as u8;
        out[INIT_PARAMS_OFFSET..INIT_PARAMS_OFFSET + INIT_PARAMS_SIZE]
            .copy_from_slice(&record.init_params);

        let max_block_size_u16 = MAX_BLOCK_SIZE as u16;
        out[MAX_BLOCK_SIZE_OFFSET..MAX_BLOCK_SIZE_OFFSET + 2]
            .copy_from_slice(&max_block_size_u16.to_le_bytes());

        if let Some(chain_configuration) = &record.chain_configuration {
            out[CHAIN_CONFIG_OFFSET..CHAIN_CONFIG_OFFSET + MAX_BLOCK_SIZE]
                .copy_from_slice(chain_configuration);
        }

        let crc = Self::crc32(&out[..CONTROL_CRC32_OFFSET]);
        out[CONTROL_CRC32_OFFSET..CONTROL_CRC32_OFFSET + 4].copy_from_slice(&crc.to_le_bytes());
        out
    }

    fn deserialize_record(
        bytes: &[u8; CONTROL_PLANE_ENTRY_SIZE],
    ) -> Result<ControlPlaneRecord, StorageError> {
        if bytes.iter().all(|value| *value == 0) {
            return Err(StorageError::ControlPlaneUninitialized);
        }

        let mut stored_crc_bytes = [0u8; 4];
        stored_crc_bytes.copy_from_slice(&bytes[CONTROL_CRC32_OFFSET..CONTROL_CRC32_OFFSET + 4]);
        let stored_crc = u32::from_le_bytes(stored_crc_bytes);
        let computed_crc = Self::crc32(&bytes[..CONTROL_CRC32_OFFSET]);
        if stored_crc != computed_crc {
            return Err(StorageError::ControlPlaneCorrupted);
        }

        if bytes[VERSION_OFFSET] != CONTROL_PLANE_VERSION {
            return Err(StorageError::ControlPlaneIncompatible);
        }

        if bytes[PRIVATE_KEY_SIZE_OFFSET] as usize != PRIVATE_KEY_SIZE {
            return Err(StorageError::ControlPlaneIncompatible);
        }

        if bytes[INIT_PARAMS_SIZE_OFFSET] as usize != INIT_PARAMS_SIZE {
            return Err(StorageError::ControlPlaneIncompatible);
        }

        let mut max_block_size_bytes = [0u8; 2];
        max_block_size_bytes
            .copy_from_slice(&bytes[MAX_BLOCK_SIZE_OFFSET..MAX_BLOCK_SIZE_OFFSET + 2]);
        let persisted_max_block_size = u16::from_le_bytes(max_block_size_bytes) as usize;
        if persisted_max_block_size != MAX_BLOCK_SIZE {
            return Err(StorageError::ControlPlaneIncompatible);
        }

        let mut private_key = [0u8; PRIVATE_KEY_SIZE];
        private_key.copy_from_slice(&bytes[PRIVATE_KEY_OFFSET..PRIVATE_KEY_OFFSET + PRIVATE_KEY_SIZE]);

        let mut own_node_id_bytes = [0u8; 4];
        own_node_id_bytes.copy_from_slice(&bytes[OWN_NODE_ID_OFFSET..OWN_NODE_ID_OFFSET + 4]);
        let own_node_id = u32::from_le_bytes(own_node_id_bytes);

        let mut init_params = [0u8; INIT_PARAMS_SIZE];
        init_params.copy_from_slice(&bytes[INIT_PARAMS_OFFSET..INIT_PARAMS_OFFSET + INIT_PARAMS_SIZE]);

        let chain_configuration = if bytes[CHAIN_CONFIG_OFFSET] == 0 {
            None
        } else {
            let mut value = [0u8; MAX_BLOCK_SIZE];
            value.copy_from_slice(&bytes[CHAIN_CONFIG_OFFSET..CHAIN_CONFIG_OFFSET + MAX_BLOCK_SIZE]);
            Some(value)
        };

        Ok(ControlPlaneRecord {
            private_key,
            own_node_id,
            init_params,
            chain_configuration,
        })
    }

    fn load_primary_record_and_repair(&mut self) -> Result<ControlPlaneRecord, StorageError> {
        let mut first_valid_index: Option<usize> = None;
        let mut first_valid_record: Option<ControlPlaneRecord> = None;
        let mut invalid_indexes = [usize::MAX; CONTROL_PLANE_COUNT];
        let mut invalid_len = 0usize;
        let mut saw_non_zero = false;
        let mut saw_incompatible = false;
        let mut saw_corrupted = false;

        let mut index = 0usize;
        while index < CONTROL_PLANE_COUNT {
            let entry = self.read_control_plane_entry(index);
            if entry.iter().any(|value| *value != 0) {
                saw_non_zero = true;
            }

            match Self::deserialize_record(&entry) {
                Ok(record) => {
                    if first_valid_index.is_none() {
                        first_valid_index = Some(index);
                        first_valid_record = Some(record);
                    }
                }
                Err(err) => {
                    if matches!(err, StorageError::ControlPlaneIncompatible) {
                        saw_incompatible = true;
                    }
                    if matches!(err, StorageError::ControlPlaneCorrupted) {
                        saw_corrupted = true;
                    }
                    invalid_indexes[invalid_len] = index;
                    invalid_len += 1;
                }
            }
            index += 1;
        }

        let record = match first_valid_record {
            Some(value) => value,
            None => {
                if !saw_non_zero {
                    return Err(StorageError::ControlPlaneUninitialized);
                }
                if saw_incompatible {
                    return Err(StorageError::ControlPlaneIncompatible);
                }
                if saw_corrupted {
                    return Err(StorageError::ControlPlaneCorrupted);
                }
                return Err(StorageError::ControlPlaneCorrupted);
            }
        };

        let encoded = Self::serialize_record(&record);
        let mut repair_index = 0usize;
        while repair_index < invalid_len {
            let target = invalid_indexes[repair_index];
            if Some(target) != first_valid_index {
                self.write_control_plane_entry(target, &encoded);
            }
            repair_index += 1;
        }

        Ok(record)
    }

    fn write_record_to_all_replicas(&mut self, record: &ControlPlaneRecord) {
        let encoded = Self::serialize_record(record);
        let mut index = 0usize;
        while index < CONTROL_PLANE_COUNT {
            self.write_control_plane_entry(index, &encoded);
            index += 1;
        }
    }
}

impl<const STORAGE_SIZE: usize> StorageTrait for MemoryBackend<STORAGE_SIZE> {
    fn init(
        &mut self,
        private_key: [u8; PRIVATE_KEY_SIZE],
        own_node_id: u32,
        init_params: [u8; INIT_PARAMS_SIZE],
    ) -> Result<(), StorageError> {
        self.storage.fill(0);

        let record = ControlPlaneRecord {
            private_key,
            own_node_id,
            init_params,
            chain_configuration: None,
        };
        self.write_record_to_all_replicas(&record);
        Ok(())
    }

    fn save_block(
        &mut self,
        storage_index: StorageIndex,
        block: &Block,
    ) -> Result<(), StorageError> {
        let (slot_start, slot_end) = Self::slot_range(storage_index)?;

        let block_bytes = block.as_bytes();
        if block_bytes.len() > MAX_BLOCK_SIZE {
            return Err(StorageError::BackendIo { code: 1 });
        }

        self.storage[slot_start..slot_end].fill(0);
        let write_end = slot_start + block_bytes.len();
        self.storage[slot_start..write_end].copy_from_slice(block_bytes);
        Ok(())
    }

    fn read_block(&self, storage_index: StorageIndex) -> Result<Block, StorageError> {
        let (slot_start, slot_end) = Self::slot_range(storage_index)?;
        let slot = &self.storage[slot_start..slot_end];
        if slot[0] == 0 {
            return Err(StorageError::BlockAbsent);
        }

        Block::from_bytes(slot).map_err(|_| StorageError::BackendIo { code: 2 })
    }

    fn set_chain_configuration(&mut self, block: &Block) -> Result<(), StorageError> {
        let mut record = self.load_primary_record_and_repair()?;
        if record.chain_configuration.is_some() {
            return Err(StorageError::ChainConfigurationAlreadySet);
        }

        let mut encoded_block = [0u8; MAX_BLOCK_SIZE];
        let block_bytes = block.as_bytes();
        if block_bytes.len() > MAX_BLOCK_SIZE {
            return Err(StorageError::BackendIo { code: 1 });
        }
        encoded_block[..block_bytes.len()].copy_from_slice(block_bytes);
        record.chain_configuration = Some(encoded_block);

        self.write_record_to_all_replicas(&record);
        Ok(())
    }

    fn load_control_data(&mut self) -> Result<ControlPlaneData, StorageError> {
        let record = self.load_primary_record_and_repair()?;

        let chain_configuration = match record.chain_configuration {
            Some(bytes) => {
                Some(Block::from_bytes(&bytes).map_err(|_| StorageError::ControlPlaneCorrupted)?)
            }
            None => None,
        };

        Ok(ControlPlaneData {
            version: CONTROL_PLANE_VERSION,
            private_key: record.private_key,
            own_node_id: record.own_node_id,
            init_params: record.init_params,
            chain_configuration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moonblokz_chain_types::HEADER_SIZE;

    const TEST_PRIVATE_KEY: [u8; PRIVATE_KEY_SIZE] = [7u8; PRIVATE_KEY_SIZE];
    const TEST_NODE_ID: u32 = 42;
    const TEST_INIT_PARAMS: [u8; INIT_PARAMS_SIZE] = [9u8; INIT_PARAMS_SIZE];
    const TEST_STORAGE_SIZE_2_SLOTS: usize = CONTROL_PLANE_RESERVED_BYTES + (2 * MAX_BLOCK_SIZE);
    const TEST_STORAGE_SIZE_3_SLOTS: usize = CONTROL_PLANE_RESERVED_BYTES + (3 * MAX_BLOCK_SIZE);
    const TEST_STORAGE_SIZE_4_SLOTS: usize = CONTROL_PLANE_RESERVED_BYTES + (4 * MAX_BLOCK_SIZE);

    fn block_from_len_and_marker(len: usize, marker: u8) -> Block {
        let mut bytes = [0u8; HEADER_SIZE + 8];
        bytes[0] = marker;
        let parse_result = Block::from_bytes(&bytes[..len]);
        assert!(parse_result.is_ok());
        match parse_result {
            Ok(value) => value,
            Err(_) => unreachable!(),
        }
    }

    fn expected_slot_bytes(block: &Block) -> [u8; MAX_BLOCK_SIZE] {
        let mut out = [0u8; MAX_BLOCK_SIZE];
        let bytes = block.as_bytes();
        out[..bytes.len()].copy_from_slice(bytes);
        out
    }

    fn initialized_backend<const STORAGE_SIZE: usize>() -> MemoryBackend<STORAGE_SIZE> {
        let mut backend = MemoryBackend::<STORAGE_SIZE>::new();
        assert!(backend
            .init(TEST_PRIVATE_KEY, TEST_NODE_ID, TEST_INIT_PARAMS)
            .is_ok());
        backend
    }

    #[test]
    fn load_control_data_reports_uninitialized_before_init() {
        let mut backend = MemoryBackend::<TEST_STORAGE_SIZE_2_SLOTS>::new();
        assert!(matches!(
            backend.load_control_data(),
            Err(StorageError::ControlPlaneUninitialized)
        ));
    }

    #[test]
    fn load_control_data_reports_incompatible_when_all_replicas_incompatible() {
        let mut backend = initialized_backend::<TEST_STORAGE_SIZE_2_SLOTS>();
        let mut replica = backend.read_control_plane_entry(0);
        replica[VERSION_OFFSET] = CONTROL_PLANE_VERSION.wrapping_add(1);
        let crc = MemoryBackend::<TEST_STORAGE_SIZE_2_SLOTS>::crc32(&replica[..CONTROL_CRC32_OFFSET]);
        replica[CONTROL_CRC32_OFFSET..CONTROL_CRC32_OFFSET + 4].copy_from_slice(&crc.to_le_bytes());
        backend.write_control_plane_entry(0, &replica);
        backend.write_control_plane_entry(1, &replica);
        backend.write_control_plane_entry(2, &replica);

        assert!(matches!(
            backend.load_control_data(),
            Err(StorageError::ControlPlaneIncompatible)
        ));
    }

    #[test]
    fn init_stores_control_plane_and_clears_block_slots() {
        let mut backend = MemoryBackend::<TEST_STORAGE_SIZE_2_SLOTS>::new();
        let block = block_from_len_and_marker(HEADER_SIZE, 1);
        assert!(backend.save_block(0, &block).is_ok());

        assert!(backend
            .init(TEST_PRIVATE_KEY, TEST_NODE_ID, TEST_INIT_PARAMS)
            .is_ok());

        assert!(matches!(backend.read_block(0), Err(StorageError::BlockAbsent)));
        let loaded = backend.load_control_data();
        assert!(loaded.is_ok());
        let loaded = match loaded {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(loaded.version, CONTROL_PLANE_VERSION);
        assert_eq!(loaded.private_key, TEST_PRIVATE_KEY);
        assert_eq!(loaded.own_node_id, TEST_NODE_ID);
        assert_eq!(loaded.init_params, TEST_INIT_PARAMS);
        assert!(loaded.chain_configuration.is_none());
    }

    #[test]
    fn set_chain_configuration_is_set_once() {
        let mut backend = initialized_backend::<TEST_STORAGE_SIZE_2_SLOTS>();
        let config = block_from_len_and_marker(HEADER_SIZE, 3);

        assert!(backend.set_chain_configuration(&config).is_ok());
        assert!(matches!(
            backend.set_chain_configuration(&config),
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
    fn load_repairs_corrupted_control_plane_replica() {
        let mut backend = initialized_backend::<TEST_STORAGE_SIZE_2_SLOTS>();
        let mut replica = backend.read_control_plane_entry(1);
        replica[VERSION_OFFSET] ^= 0xFF;
        backend.write_control_plane_entry(1, &replica);

        let loaded = backend.load_control_data();
        assert!(loaded.is_ok());
        let repaired = MemoryBackend::<TEST_STORAGE_SIZE_2_SLOTS>::deserialize_record(
            &backend.read_control_plane_entry(1),
        );
        assert!(repaired.is_ok());
    }

    #[test]
    fn compile_time_block_storage_size_is_enforced() {
        let mut backend = initialized_backend::<TEST_STORAGE_SIZE_2_SLOTS>();
        let block = block_from_len_and_marker(HEADER_SIZE, 1);

        assert!(backend.save_block(0, &block).is_ok());
        assert!(backend.save_block(1, &block).is_ok());
        assert!(matches!(
            backend.save_block(2, &block),
            Err(StorageError::InvalidIndex)
        ));
    }

    #[test]
    fn read_reports_absent_for_valid_empty_slot() {
        let backend = initialized_backend::<TEST_STORAGE_SIZE_2_SLOTS>();
        assert!(matches!(
            backend.read_block(0),
            Err(StorageError::BlockAbsent)
        ));
    }

    #[test]
    fn read_reports_invalid_index_for_out_of_range_slot() {
        let backend = initialized_backend::<TEST_STORAGE_SIZE_2_SLOTS>();
        assert!(matches!(
            backend.read_block(2),
            Err(StorageError::InvalidIndex)
        ));
    }

    #[test]
    fn save_and_read_round_trip() {
        let mut backend = initialized_backend::<TEST_STORAGE_SIZE_2_SLOTS>();
        let block = block_from_len_and_marker(HEADER_SIZE, 1);

        assert!(backend.save_block(0, &block).is_ok());
        let read_result = backend.read_block(0);
        assert!(read_result.is_ok());
        let read_block = match read_result {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(read_block.len(), MAX_BLOCK_SIZE);
        assert_eq!(read_block.as_bytes(), &expected_slot_bytes(&block));
    }

    #[test]
    fn startup_read_cycle_over_empty_backend_is_deterministic() {
        let backend = initialized_backend::<TEST_STORAGE_SIZE_3_SLOTS>();

        for storage_index in 0u32..3u32 {
            let result = backend.read_block(storage_index);
            assert!(matches!(result, Err(StorageError::BlockAbsent)));
        }

        assert!(matches!(
            backend.read_block(3),
            Err(StorageError::InvalidIndex)
        ));
    }

    #[test]
    fn overwrite_same_index_is_deterministic() {
        let mut backend = initialized_backend::<TEST_STORAGE_SIZE_2_SLOTS>();
        let first = block_from_len_and_marker(HEADER_SIZE, 3);
        let second = block_from_len_and_marker(HEADER_SIZE + 1, 4);

        assert!(backend.save_block(0, &first).is_ok());
        assert!(backend.save_block(0, &second).is_ok());

        let read_result = backend.read_block(0);
        assert!(read_result.is_ok());
        let read_block = match read_result {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(read_block.as_bytes(), &expected_slot_bytes(&second));
    }

    #[test]
    fn multi_index_save_and_retrieve_returns_expected_blocks() {
        let mut backend = initialized_backend::<TEST_STORAGE_SIZE_3_SLOTS>();
        let block_a = block_from_len_and_marker(HEADER_SIZE, 5);
        let block_b = block_from_len_and_marker(HEADER_SIZE + 2, 6);

        assert!(backend.save_block(0, &block_a).is_ok());
        assert!(backend.save_block(1, &block_b).is_ok());

        let read_a = backend.read_block(0);
        let read_b = backend.read_block(1);
        assert!(read_a.is_ok());
        assert!(read_b.is_ok());

        let read_a = match read_a {
            Ok(value) => value,
            Err(_) => return,
        };
        let read_b = match read_b {
            Ok(value) => value,
            Err(_) => return,
        };

        assert_eq!(read_a.as_bytes(), &expected_slot_bytes(&block_a));
        assert_eq!(read_b.as_bytes(), &expected_slot_bytes(&block_b));
    }

    #[test]
    fn startup_read_cycle_with_mixed_slots_returns_typed_outcomes() {
        let mut backend = initialized_backend::<TEST_STORAGE_SIZE_4_SLOTS>();
        let block = block_from_len_and_marker(HEADER_SIZE + 1, 7);

        assert!(backend.save_block(1, &block).is_ok());
        assert!(backend.save_block(3, &block).is_ok());

        assert!(matches!(
            backend.read_block(0),
            Err(StorageError::BlockAbsent)
        ));
        assert!(matches!(backend.read_block(1), Ok(_)));
        assert!(matches!(
            backend.read_block(2),
            Err(StorageError::BlockAbsent)
        ));
        assert!(matches!(backend.read_block(3), Ok(_)));
        assert!(matches!(
            backend.read_block(4),
            Err(StorageError::InvalidIndex)
        ));
    }

    #[test]
    fn ingest_query_integration_flow_covers_positive_and_negative_paths() {
        let mut backend = initialized_backend::<TEST_STORAGE_SIZE_4_SLOTS>();
        let block_a = block_from_len_and_marker(HEADER_SIZE, 8);
        let block_b = block_from_len_and_marker(HEADER_SIZE + 3, 9);

        assert!(matches!(
            backend.read_block(0),
            Err(StorageError::BlockAbsent)
        ));
        assert!(matches!(
            backend.read_block(1),
            Err(StorageError::BlockAbsent)
        ));

        assert!(backend.save_block(0, &block_a).is_ok());
        assert!(backend.save_block(2, &block_b).is_ok());

        let read_a = backend.read_block(0);
        let read_b = backend.read_block(2);
        assert!(read_a.is_ok());
        assert!(read_b.is_ok());
        let read_a = match read_a {
            Ok(value) => value,
            Err(_) => return,
        };
        let read_b = match read_b {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(read_a.as_bytes(), &expected_slot_bytes(&block_a));
        assert_eq!(read_b.as_bytes(), &expected_slot_bytes(&block_b));

        assert!(matches!(
            backend.read_block(1),
            Err(StorageError::BlockAbsent)
        ));
        assert!(matches!(
            backend.save_block(4, &block_a),
            Err(StorageError::InvalidIndex)
        ));
        assert!(matches!(
            backend.read_block(4),
            Err(StorageError::InvalidIndex)
        ));
    }
}
