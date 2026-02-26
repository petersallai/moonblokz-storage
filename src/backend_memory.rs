/*! In-memory backend module for MoonBlokz storage contract testing/integration. */

use crate::{StorageError, StorageIndex, StorageTrait};
use moonblokz_chain_types::Block;

/// In-memory backend with compile-time block capacity.
///
/// Startup read-cycle example:
/// ```
/// use moonblokz_chain_types::{Block, HEADER_SIZE};
/// use moonblokz_storage::{MemoryBackend, StorageError, StorageTrait};
///
/// let mut backend = MemoryBackend::<3>::new();
/// let block_result = Block::from_bytes(&[0u8; HEADER_SIZE]);
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
pub struct MemoryBackend<const BLOCK_STORAGE_SIZE: usize> {
    blocks: [Option<Block>; BLOCK_STORAGE_SIZE],
}

impl<const BLOCK_STORAGE_SIZE: usize> MemoryBackend<BLOCK_STORAGE_SIZE> {
    /// Creates a new memory backend.
    ///
    /// Parameters:
    /// - none.
    ///
    /// Example:
    /// ```
    /// use moonblokz_storage::backend_memory::MemoryBackend;
    ///
    /// let _backend = MemoryBackend::<8>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            blocks: core::array::from_fn(|_| None),
        }
    }
}

impl<const BLOCK_STORAGE_SIZE: usize> StorageTrait for MemoryBackend<BLOCK_STORAGE_SIZE> {
    fn init(&mut self) -> Result<(), StorageError> {
        self.blocks = core::array::from_fn(|_| None);
        Ok(())
    }

    fn save_block(
        &mut self,
        storage_index: StorageIndex,
        block: &Block,
    ) -> Result<(), StorageError> {
        let slot_index = storage_index as usize;
        if slot_index >= BLOCK_STORAGE_SIZE {
            return Err(StorageError::InvalidIndex);
        }

        let copy_result = Block::from_bytes(block.as_bytes());
        match copy_result {
            Ok(value) => {
                self.blocks[slot_index] = Some(value);
                Ok(())
            }
            Err(_) => Err(StorageError::BackendIo { code: 1 }),
        }
    }

    fn read_block(&self, storage_index: StorageIndex) -> Result<Block, StorageError> {
        let slot_index = storage_index as usize;
        if slot_index >= BLOCK_STORAGE_SIZE {
            return Err(StorageError::InvalidIndex);
        }

        match &self.blocks[slot_index] {
            Some(block) => {
                let copy_result = Block::from_bytes(block.as_bytes());
                match copy_result {
                    Ok(value) => Ok(value),
                    Err(_) => Err(StorageError::BackendIo { code: 2 }),
                }
            }
            None => Err(StorageError::BlockAbsent),
        }
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

    #[test]
    fn compile_time_block_storage_size_is_enforced() {
        let mut backend = MemoryBackend::<2>::new();
        let block = block_from_len_and_marker(HEADER_SIZE, 0);

        assert!(backend.save_block(0, &block).is_ok());
        assert!(backend.save_block(1, &block).is_ok());
        assert!(matches!(
            backend.save_block(2, &block),
            Err(StorageError::InvalidIndex)
        ));
    }

    #[test]
    fn read_reports_absent_for_valid_empty_slot() {
        let backend = MemoryBackend::<2>::new();
        assert!(matches!(backend.read_block(0), Err(StorageError::BlockAbsent)));
    }

    #[test]
    fn read_reports_invalid_index_for_out_of_range_slot() {
        let backend = MemoryBackend::<2>::new();
        assert!(matches!(backend.read_block(2), Err(StorageError::InvalidIndex)));
    }

    #[test]
    fn save_and_read_round_trip() {
        let mut backend = MemoryBackend::<2>::new();
        let block = block_from_len_and_marker(HEADER_SIZE, 1);

        assert!(backend.save_block(0, &block).is_ok());
        let read_result = backend.read_block(0);
        assert!(read_result.is_ok());
        let read_block = match read_result {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(read_block.len(), HEADER_SIZE);
    }

    #[test]
    fn init_resets_state_to_empty() {
        let mut backend = MemoryBackend::<2>::new();
        let block = block_from_len_and_marker(HEADER_SIZE, 2);

        assert!(backend.save_block(0, &block).is_ok());
        assert!(backend.init().is_ok());
        assert!(matches!(backend.read_block(0), Err(StorageError::BlockAbsent)));
    }

    #[test]
    fn startup_read_cycle_over_empty_backend_is_deterministic() {
        let backend = MemoryBackend::<3>::new();

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
        let mut backend = MemoryBackend::<2>::new();
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
        assert_eq!(read_block.as_bytes(), second.as_bytes());
    }

    #[test]
    fn multi_index_save_and_retrieve_returns_expected_blocks() {
        let mut backend = MemoryBackend::<3>::new();
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

        assert_eq!(read_a.as_bytes(), block_a.as_bytes());
        assert_eq!(read_b.as_bytes(), block_b.as_bytes());
    }

    #[test]
    fn startup_read_cycle_with_mixed_slots_returns_typed_outcomes() {
        let mut backend = MemoryBackend::<4>::new();
        let block = block_from_len_and_marker(HEADER_SIZE + 1, 7);

        assert!(backend.save_block(1, &block).is_ok());
        assert!(backend.save_block(3, &block).is_ok());

        assert!(matches!(backend.read_block(0), Err(StorageError::BlockAbsent)));
        assert!(matches!(backend.read_block(1), Ok(_)));
        assert!(matches!(backend.read_block(2), Err(StorageError::BlockAbsent)));
        assert!(matches!(backend.read_block(3), Ok(_)));
        assert!(matches!(
            backend.read_block(4),
            Err(StorageError::InvalidIndex)
        ));
    }
}
