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
    ///
    /// Canonical `code` mapping:
    /// Runtime codes:
    /// - `1`: memory backend save path received oversized block bytes.
    /// - `2`: memory backend read path failed to parse stored slot bytes.
    /// - `210`: RP2040 flash page read failed.
    /// - `211`: RP2040 flash page erase failed.
    /// - `212`: RP2040 flash page write failed.
    /// - `213`: RP2040 save path reached an unreachable backend branch.
    /// - `220`: RP2040 flash page read failed during retrieve path.
    ///
    /// Test-only codes (RP2040 mock flash):
    /// - `230`: mock flash read out of bounds.
    /// - `231`: mock flash erase range invalid/out of bounds.
    /// - `232`: mock flash write out of bounds.
    BackendIo {
        /// Backend-local error code.
        code: u16,
    },
}
