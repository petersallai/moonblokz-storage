/*! In-memory backend module for MoonBlokz storage contract testing/integration. */

use crate::{StorageError, StorageIndex, StorageTrait};
use moonblokz_chain_types::Block;

/// In-memory backend with compile-time block capacity.
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

    #[test]
    fn compile_time_block_storage_size_is_enforced() {
        let mut backend = MemoryBackend::<2>::new();
        let block_result = Block::from_bytes(&[0u8; HEADER_SIZE]);
        assert!(block_result.is_ok());
        let block = match block_result {
            Ok(value) => value,
            Err(_) => return,
        };

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
    fn save_and_read_round_trip() {
        let mut backend = MemoryBackend::<2>::new();
        let block_result = Block::from_bytes(&[0u8; HEADER_SIZE]);
        assert!(block_result.is_ok());
        let block = match block_result {
            Ok(value) => value,
            Err(_) => return,
        };

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
        let block_result = Block::from_bytes(&[0u8; HEADER_SIZE]);
        assert!(block_result.is_ok());
        let block = match block_result {
            Ok(value) => value,
            Err(_) => return,
        };

        assert!(backend.save_block(0, &block).is_ok());
        assert!(backend.init().is_ok());
        assert!(matches!(backend.read_block(0), Err(StorageError::BlockAbsent)));
    }
}
