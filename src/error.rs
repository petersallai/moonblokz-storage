/*! Core error model for MoonBlokz storage public API contracts. */

/// Public storage error categories used by chain-level logic.
pub enum StorageError {
    /// `storage_index` is outside valid storage bounds.
    InvalidIndex,
    /// Requested storage slot has no persisted block.
    BlockAbsent,
    /// Retrieved data failed integrity verification.
    IntegrityFailure,
    /// Backend-level I/O failure while executing a storage operation.
    BackendIo {
        /// Backend-local error code.
        code: u16,
    },
}
