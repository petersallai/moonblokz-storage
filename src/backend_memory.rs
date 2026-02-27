/*! In-memory backend module for MoonBlokz storage contract testing/integration. */

use crate::{StorageError, StorageIndex, StorageTrait};
use moonblokz_chain_types::{Block, MAX_BLOCK_SIZE};

/// In-memory backend with compile-time byte capacity.
///
/// Capacity rule:
/// - Effective slots are `STORAGE_SIZE / MAX_BLOCK_SIZE`.
/// - Remainder bytes are intentionally unused.
/// - Empty slot marker is `slot[0] == 0` (block version byte is zero).
///
/// Startup read-cycle example:
/// ```
/// use moonblokz_chain_types::{Block, HEADER_SIZE, MAX_BLOCK_SIZE};
/// use moonblokz_storage::{MemoryBackend, StorageError, StorageTrait};
///
/// let mut backend = MemoryBackend::<{ 3 * MAX_BLOCK_SIZE }>::new();
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
///         Err(StorageError::BackendIo { .. }) => { /* backend error */ }
///     }
/// }
/// ```
pub struct MemoryBackend<const STORAGE_SIZE: usize> {
    storage: [u8; STORAGE_SIZE],
}

impl<const STORAGE_SIZE: usize> MemoryBackend<STORAGE_SIZE> {
    const MAX_STORAGE_SLOTS: StorageIndex = (STORAGE_SIZE / MAX_BLOCK_SIZE) as StorageIndex;

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
    /// let _backend = MemoryBackend::<{ 8 * MAX_BLOCK_SIZE }>::new();
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

        let slot_start = storage_index as usize * MAX_BLOCK_SIZE;
        let slot_end = slot_start + MAX_BLOCK_SIZE;
        Ok((slot_start, slot_end))
    }
}

impl<const STORAGE_SIZE: usize> StorageTrait for MemoryBackend<STORAGE_SIZE> {
    fn init(&mut self) -> Result<(), StorageError> {
        self.storage.fill(0);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use moonblokz_chain_types::HEADER_SIZE;

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

    #[test]
    fn compile_time_block_storage_size_is_enforced() {
        let mut backend = MemoryBackend::<{ 2 * MAX_BLOCK_SIZE }>::new();
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
        let backend = MemoryBackend::<{ 2 * MAX_BLOCK_SIZE }>::new();
        assert!(matches!(
            backend.read_block(0),
            Err(StorageError::BlockAbsent)
        ));
    }

    #[test]
    fn read_reports_invalid_index_for_out_of_range_slot() {
        let backend = MemoryBackend::<{ 2 * MAX_BLOCK_SIZE }>::new();
        assert!(matches!(
            backend.read_block(2),
            Err(StorageError::InvalidIndex)
        ));
    }

    #[test]
    fn save_and_read_round_trip() {
        let mut backend = MemoryBackend::<{ 2 * MAX_BLOCK_SIZE }>::new();
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
    fn init_resets_state_to_empty() {
        let mut backend = MemoryBackend::<{ 2 * MAX_BLOCK_SIZE }>::new();
        let block = block_from_len_and_marker(HEADER_SIZE, 2);

        assert!(backend.save_block(0, &block).is_ok());
        assert!(backend.init().is_ok());
        assert!(matches!(
            backend.read_block(0),
            Err(StorageError::BlockAbsent)
        ));
    }

    #[test]
    fn startup_read_cycle_over_empty_backend_is_deterministic() {
        let backend = MemoryBackend::<{ 3 * MAX_BLOCK_SIZE }>::new();

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
        let mut backend = MemoryBackend::<{ 2 * MAX_BLOCK_SIZE }>::new();
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
        let mut backend = MemoryBackend::<{ 3 * MAX_BLOCK_SIZE }>::new();
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
        let mut backend = MemoryBackend::<{ 4 * MAX_BLOCK_SIZE }>::new();
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
        let mut backend = MemoryBackend::<{ 4 * MAX_BLOCK_SIZE }>::new();
        let block_a = block_from_len_and_marker(HEADER_SIZE, 8);
        let block_b = block_from_len_and_marker(HEADER_SIZE + 3, 9);

        // Startup-style initial query on empty storage.
        assert!(matches!(
            backend.read_block(0),
            Err(StorageError::BlockAbsent)
        ));
        assert!(matches!(
            backend.read_block(1),
            Err(StorageError::BlockAbsent)
        ));

        // Ingest-style writes.
        assert!(backend.save_block(0, &block_a).is_ok());
        assert!(backend.save_block(2, &block_b).is_ok());

        // Query-style reads.
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

        // Negative-path query outcomes remain typed and deterministic.
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
