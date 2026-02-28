/*! Backend conformance tests that validate shared storage trait semantics. */

#[cfg(test)]
mod tests {
    use crate::{MoonblokzStorage, StorageError, StorageTrait};
    use moonblokz_chain_types::{Block, MAX_BLOCK_SIZE};
    use moonblokz_crypto::PRIVATE_KEY_SIZE;

    #[cfg(feature = "backend-memory")]
    const TEST_STORAGE_SIZE: usize = 8 * MAX_BLOCK_SIZE;
    #[cfg(feature = "backend-rp2040")]
    const TEST_STORAGE_SIZE: usize = (crate::CONTROL_PLANE_COUNT + 2) * 4096;
    #[cfg(feature = "backend-rp2040")]
    const TEST_INVALID_INDEX: u32 = ((TEST_STORAGE_SIZE / crate::backend_rp2040::FLASH_PAGE_SIZE)
        * crate::backend_rp2040::BLOCKS_PER_PAGE) as u32;
    #[cfg(feature = "backend-memory")]
    const TEST_CONTROL_PLANE_ENTRY_SIZE: usize =
        1 + 1 + PRIVATE_KEY_SIZE + 4 + 1 + crate::INIT_PARAMS_SIZE + 2 + MAX_BLOCK_SIZE + 4;
    #[cfg(feature = "backend-memory")]
    const TEST_CONTROL_PLANE_RESERVED_BYTES: usize =
        crate::CONTROL_PLANE_COUNT * TEST_CONTROL_PLANE_ENTRY_SIZE;
    #[cfg(feature = "backend-memory")]
    const TEST_INVALID_INDEX: u32 = if TEST_STORAGE_SIZE > TEST_CONTROL_PLANE_RESERVED_BYTES {
        ((TEST_STORAGE_SIZE - TEST_CONTROL_PLANE_RESERVED_BYTES) / MAX_BLOCK_SIZE) as u32
    } else {
        0
    };

    #[cfg(feature = "backend-memory")]
    fn new_backend() -> MoonblokzStorage<TEST_STORAGE_SIZE> {
        MoonblokzStorage::<TEST_STORAGE_SIZE>::new()
    }

    #[cfg(feature = "backend-rp2040")]
    fn new_backend() -> MoonblokzStorage<TEST_STORAGE_SIZE> {
        MoonblokzStorage::<TEST_STORAGE_SIZE>::new_for_tests(0).unwrap_or_else(|_| unreachable!())
    }

    fn block_from_marker(marker: u8) -> Block {
        let mut bytes = [0u8; MAX_BLOCK_SIZE];
        bytes[0] = 1;
        bytes[1] = marker;
        let block_result = Block::from_bytes(&bytes);
        assert!(block_result.is_ok());
        match block_result {
            Ok(value) => value,
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn conformance_save_read_round_trip_returns_exact_saved_block() {
        let mut storage = new_backend();
        assert!(storage
            .init(
                [1u8; PRIVATE_KEY_SIZE],
                1,
                [0u8; crate::INIT_PARAMS_SIZE],
            )
            .is_ok());
        let block = block_from_marker(41);
        assert!(storage.save_block(0, &block).is_ok());

        let read_result = storage.read_block(0);
        assert!(read_result.is_ok());
        let read_block = match read_result {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(read_block.as_bytes(), block.as_bytes());
    }

    #[test]
    fn conformance_empty_slot_reports_block_absent() {
        let mut storage = new_backend();
        assert!(storage
            .init(
                [1u8; PRIVATE_KEY_SIZE],
                1,
                [0u8; crate::INIT_PARAMS_SIZE],
            )
            .is_ok());
        assert!(matches!(
            storage.read_block(0),
            Err(StorageError::BlockAbsent)
        ));
    }

    #[test]
    fn conformance_invalid_index_reports_invalid_index_for_read_and_save() {
        let mut storage = new_backend();
        assert!(storage
            .init(
                [1u8; PRIVATE_KEY_SIZE],
                1,
                [0u8; crate::INIT_PARAMS_SIZE],
            )
            .is_ok());
        let block = block_from_marker(42);

        assert!(matches!(
            storage.read_block(TEST_INVALID_INDEX),
            Err(StorageError::InvalidIndex)
        ));
        assert!(matches!(
            storage.save_block(TEST_INVALID_INDEX, &block),
            Err(StorageError::InvalidIndex)
        ));
    }

    #[test]
    fn conformance_startup_scan_returns_typed_outcomes_for_mixed_slots() {
        let mut storage = new_backend();
        assert!(storage
            .init(
                [1u8; PRIVATE_KEY_SIZE],
                1,
                [0u8; crate::INIT_PARAMS_SIZE],
            )
            .is_ok());
        let block_a = block_from_marker(43);
        let block_b = block_from_marker(44);
        assert!(storage.save_block(1, &block_a).is_ok());
        assert!(storage.save_block(3, &block_b).is_ok());

        assert!(matches!(
            storage.read_block(0),
            Err(StorageError::BlockAbsent)
        ));
        assert!(matches!(storage.read_block(1), Ok(_)));
        assert!(matches!(
            storage.read_block(2),
            Err(StorageError::BlockAbsent)
        ));
        assert!(matches!(storage.read_block(3), Ok(_)));
    }
}
