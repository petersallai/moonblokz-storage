/*! RP2040 backend module stub for MoonBlokz storage contract wiring. */

use crate::{StorageError, StorageIndex, StorageTrait};
use moonblokz_chain_types::Block;

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
