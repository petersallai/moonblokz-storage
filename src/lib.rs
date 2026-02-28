/*! MoonBlokz storage crate.

This crate targets embedded `no_std` environments. Implementation modules and
public API contracts are added incrementally by stories.
*/

#![no_std]

#[cfg(test)]
mod conformance;
pub mod error;
pub mod types;

#[cfg(feature = "backend-memory")]
pub mod backend_memory;
#[cfg(feature = "backend-rp2040")]
pub mod backend_rp2040;

#[cfg(not(any(feature = "backend-memory", feature = "backend-rp2040")))]
compile_error!("Exactly one backend feature must be enabled: backend-memory or backend-rp2040.");

#[cfg(all(feature = "backend-memory", feature = "backend-rp2040"))]
compile_error!("Exactly one backend feature must be enabled: backend-memory or backend-rp2040.");

use moonblokz_chain_types::Block;
use moonblokz_crypto::PRIVATE_KEY_SIZE;

#[cfg(feature = "backend-memory")]
pub use backend_memory::MemoryBackend;
#[cfg(feature = "backend-rp2040")]
pub use backend_rp2040::Rp2040Backend;
#[cfg(all(feature = "backend-memory", not(feature = "backend-rp2040")))]
/// Canonical storage backend alias for the selected `backend-memory` feature.
///
/// Parameters:
/// - `STORAGE_SIZE`: total storage bytes used by the memory backend.
///
/// Example:
/// ```
/// use moonblokz_chain_types::MAX_BLOCK_SIZE;
/// use moonblokz_storage::MoonblokzStorage;
///
/// let _storage = MoonblokzStorage::<{ 4 * MAX_BLOCK_SIZE }>::new();
/// ```
pub type MoonblokzStorage<const STORAGE_SIZE: usize> = MemoryBackend<STORAGE_SIZE>;
#[cfg(all(feature = "backend-rp2040", not(feature = "backend-memory")))]
/// Canonical storage backend alias for the selected `backend-rp2040` feature.
///
/// Parameters:
/// - `STORAGE_SIZE`: total RP2040 flash size in bytes used for geometry calculations.
///
/// Example:
/// ```ignore
/// use moonblokz_storage::MoonblokzStorage;
///
/// let storage = MoonblokzStorage::<{ 2 * 1024 * 1024 }>::new_for_tests(0).unwrap();
/// let _use_storage = storage;
/// ```
pub type MoonblokzStorage<const STORAGE_SIZE: usize> = Rp2040Backend<STORAGE_SIZE>;
pub use error::StorageError;
pub use types::StorageIndex;

/// Initialization parameter byte size.
pub const INIT_PARAMS_SIZE: usize = 100;
/// Number of replicated control-plane entries.
pub const CONTROL_PLANE_COUNT: usize = 3;
/// Storage-library control-plane schema version.
pub const CONTROL_PLANE_VERSION: u8 = 1;

/// Canonical control-plane data returned by `load_control_data`.
pub struct ControlPlaneData {
    /// Persisted private key.
    pub private_key: [u8; PRIVATE_KEY_SIZE],
    /// Persisted own node id.
    pub own_node_id: u32,
    /// Persisted free-form init parameters.
    pub init_params: [u8; INIT_PARAMS_SIZE],
    /// Optional chain-configuration block.
    pub chain_configuration: Option<Block>,
}

/// Synchronous, `no_std` storage API contract for MoonBlokz chain logic.
pub trait StorageTrait {
    /// Initializes backend storage state.
    ///
    /// Parameters:
    /// - `private_key`: node private key bytes.
    /// - `own_node_id`: local node identifier.
    /// - `init_params`: free-form control-plane initialization bytes.
    ///
    /// Example:
    /// ```
    /// use moonblokz_storage::{StorageError, StorageTrait};
    ///
    /// struct DummyStorage;
    ///
    /// impl StorageTrait for DummyStorage {
    ///     fn init(
    ///         &mut self,
    ///         _private_key: [u8; moonblokz_crypto::PRIVATE_KEY_SIZE],
    ///         _own_node_id: u32,
    ///         _init_params: [u8; moonblokz_storage::INIT_PARAMS_SIZE],
    ///     ) -> Result<(), StorageError> { Ok(()) }
    ///     fn save_block(&mut self, _storage_index: u32, _block: &moonblokz_chain_types::Block) -> Result<(), StorageError> {
    ///         Ok(())
    ///     }
    ///     fn read_block(&self, _storage_index: u32) -> Result<moonblokz_chain_types::Block, StorageError> {
    ///         Err(StorageError::BlockAbsent)
    ///     }
    ///     fn set_chain_configuration(&mut self, _block: &moonblokz_chain_types::Block) -> Result<(), StorageError> {
    ///         Ok(())
    ///     }
    ///     fn load_control_data(&mut self) -> Result<moonblokz_storage::ControlPlaneData, StorageError> {
    ///         Err(StorageError::ControlPlaneUninitialized)
    ///     }
    /// }
    ///
    /// let mut storage = DummyStorage;
    /// assert!(storage.init(
    ///     [1u8; moonblokz_crypto::PRIVATE_KEY_SIZE],
    ///     7,
    ///     [0u8; moonblokz_storage::INIT_PARAMS_SIZE]
    /// ).is_ok());
    /// ```
    fn init(
        &mut self,
        private_key: [u8; PRIVATE_KEY_SIZE],
        own_node_id: u32,
        init_params: [u8; INIT_PARAMS_SIZE],
    ) -> Result<(), StorageError>;

    /// Persists a block at a specific `storage_index`.
    ///
    /// Parameters:
    /// - `storage_index`: destination slot index.
    /// - `block`: canonical block reference to persist.
    ///
    /// Example:
    /// ```
    /// use moonblokz_storage::{StorageError, StorageTrait};
    ///
    /// struct DummyStorage;
    ///
    /// impl StorageTrait for DummyStorage {
    ///     fn init(
    ///         &mut self,
    ///         _private_key: [u8; moonblokz_crypto::PRIVATE_KEY_SIZE],
    ///         _own_node_id: u32,
    ///         _init_params: [u8; moonblokz_storage::INIT_PARAMS_SIZE],
    ///     ) -> Result<(), StorageError> { Ok(()) }
    ///     fn save_block(&mut self, storage_index: u32, _block: &moonblokz_chain_types::Block) -> Result<(), StorageError> {
    ///         if storage_index != 0 {
    ///             return Err(StorageError::InvalidIndex);
    ///         }
    ///         Ok(())
    ///     }
    ///     fn read_block(&self, _storage_index: u32) -> Result<moonblokz_chain_types::Block, StorageError> {
    ///         Err(StorageError::BlockAbsent)
    ///     }
    ///     fn set_chain_configuration(&mut self, _block: &moonblokz_chain_types::Block) -> Result<(), StorageError> {
    ///         Ok(())
    ///     }
    ///     fn load_control_data(&mut self) -> Result<moonblokz_storage::ControlPlaneData, StorageError> {
    ///         Err(StorageError::ControlPlaneUninitialized)
    ///     }
    /// }
    ///
    /// let mut storage = DummyStorage;
    /// let mut bytes = [0u8; moonblokz_chain_types::HEADER_SIZE];
    /// bytes[0] = 1;
    /// let block_result = moonblokz_chain_types::Block::from_bytes(&bytes);
    /// assert!(block_result.is_ok());
    /// let block = match block_result {
    ///     Ok(value) => value,
    ///     Err(_) => return,
    /// };
    /// assert!(storage.save_block(0, &block).is_ok());
    /// ```
    fn save_block(
        &mut self,
        storage_index: StorageIndex,
        block: &Block,
    ) -> Result<(), StorageError>;

    /// Reads and returns a block from a specific `storage_index`.
    ///
    /// Parameters:
    /// - `storage_index`: slot index to read.
    ///
    /// Example:
    /// ```
    /// use moonblokz_storage::{StorageError, StorageTrait};
    ///
    /// struct DummyStorage;
    ///
    /// impl StorageTrait for DummyStorage {
    ///     fn init(
    ///         &mut self,
    ///         _private_key: [u8; moonblokz_crypto::PRIVATE_KEY_SIZE],
    ///         _own_node_id: u32,
    ///         _init_params: [u8; moonblokz_storage::INIT_PARAMS_SIZE],
    ///     ) -> Result<(), StorageError> { Ok(()) }
    ///     fn save_block(&mut self, _storage_index: u32, _block: &moonblokz_chain_types::Block) -> Result<(), StorageError> {
    ///         Ok(())
    ///     }
    ///     fn read_block(&self, _storage_index: u32) -> Result<moonblokz_chain_types::Block, StorageError> {
    ///         Err(StorageError::BlockAbsent)
    ///     }
    ///     fn set_chain_configuration(&mut self, _block: &moonblokz_chain_types::Block) -> Result<(), StorageError> {
    ///         Ok(())
    ///     }
    ///     fn load_control_data(&mut self) -> Result<moonblokz_storage::ControlPlaneData, StorageError> {
    ///         Err(StorageError::ControlPlaneUninitialized)
    ///     }
    /// }
    ///
    /// let storage = DummyStorage;
    /// let result = storage.read_block(0);
    /// assert!(matches!(result, Err(StorageError::BlockAbsent)));
    /// ```
    fn read_block(&self, storage_index: StorageIndex) -> Result<Block, StorageError>;

    /// Persists the chain-configuration block once after initialization.
    ///
    /// Parameters:
    /// - `block`: chain-configuration block to persist.
    fn set_chain_configuration(&mut self, block: &Block) -> Result<(), StorageError>;

    /// Loads control-plane data and performs best-effort replica repair.
    ///
    /// Parameters:
    /// - none.
    fn load_control_data(&mut self) -> Result<ControlPlaneData, StorageError>;
}
