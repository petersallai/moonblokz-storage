/*! Host (`std`) example for MoonBlokz storage lifecycle usage.

This example demonstrates:
- checking whether storage is initialized,
- initializing when required,
- saving a block,
- reading the same block back.
*/

use moonblokz_chain_types::{Block, BlockBuilder, BlockHeader};
use moonblokz_crypto::{Crypto, CryptoTrait, PRIVATE_KEY_SIZE};
use moonblokz_storage::{INIT_PARAMS_SIZE, MemoryBackend, StorageError, StorageTrait};

const STORAGE_SIZE: usize = 64 * 1024;
const EXAMPLE_STORAGE_INDEX: u32 = 0;

fn make_example_block() -> Result<Block, StorageError> {
    let header = BlockHeader {
        version: 1,
        sequence: 1,
        creator: 1001,
        mined_amount: 0,
        payload_type: 1,
        consumed_votes: 0,
        first_voted_node: 0,
        consumed_votes_from_first_voted_node: 0,
        previous_hash: [0u8; 32],
        signature: [1u8; 64],
    };

    let crypto =
        Crypto::new([7u8; PRIVATE_KEY_SIZE]).map_err(|_| StorageError::BackendIo { code: 239 })?;

    BlockBuilder::new()
        .header(header)
        .build_signed(&crypto)
        .map_err(|_| StorageError::BackendIo { code: 241 })
}

fn block_round_tripped(expected: &Block, actual: &Block) -> bool {
    let expected_bytes = expected.serialized_bytes();
    let actual_bytes = actual.serialized_bytes();
    if actual_bytes.len() < expected_bytes.len() {
        return false;
    }
    if &actual_bytes[..expected_bytes.len()] != expected_bytes {
        return false;
    }
    actual_bytes[expected_bytes.len()..]
        .iter()
        .all(|value| *value == 0)
}

fn run_flow(storage: &mut impl StorageTrait) -> Result<bool, StorageError> {
    match storage.load_control_data() {
        Ok(_) => {}
        Err(StorageError::ControlPlaneUninitialized) => {
            storage.init([7u8; PRIVATE_KEY_SIZE], 1001, [9u8; INIT_PARAMS_SIZE])?;
        }
        Err(err) => return Err(err),
    }

    let capacity = storage.capacity();
    if capacity == 0 || EXAMPLE_STORAGE_INDEX >= capacity {
        return Err(StorageError::InvalidIndex);
    }

    let block = make_example_block()?;
    storage.save_block(EXAMPLE_STORAGE_INDEX, &block)?;
    let loaded = storage.read_block(EXAMPLE_STORAGE_INDEX)?;
    Ok(block_round_tripped(&block, &loaded))
}

fn main() {
    let mut storage = MemoryBackend::<STORAGE_SIZE>::new();
    match run_flow(&mut storage) {
        Ok(true) => {
            println!("Storage example succeeded: block save/read flow completed.");
        }
        Ok(false) => {
            println!("Storage example failed: read block differs from saved block.");
        }
        Err(err) => {
            println!("Storage example failed with error.");
            match err {
                StorageError::InvalidIndex => println!("Error: InvalidIndex"),
                StorageError::BlockAbsent => println!("Error: BlockAbsent"),
                StorageError::IntegrityFailure => println!("Error: IntegrityFailure"),
                StorageError::ControlPlaneUninitialized => {
                    println!("Error: ControlPlaneUninitialized")
                }
                StorageError::ChainConfigurationAlreadySet => {
                    println!("Error: ChainConfigurationAlreadySet")
                }
                StorageError::ControlPlaneCorrupted => println!("Error: ControlPlaneCorrupted"),
                StorageError::ControlPlaneIncompatible => {
                    println!("Error: ControlPlaneIncompatible")
                }
                StorageError::InvalidConfiguration => println!("Error: InvalidConfiguration"),
                StorageError::BackendIo { code } => println!("Error: BackendIo(code={})", code),
            }
        }
    }
}
