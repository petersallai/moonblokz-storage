/*! Host (`std`) example for MoonBlokz storage lifecycle usage.

This example demonstrates:
- checking whether storage is initialized,
- initializing when required,
- saving a block,
- reading the same block back.
*/

use moonblokz_chain_types::{Block, BlockBuilder, BlockHeader};
use moonblokz_crypto::PRIVATE_KEY_SIZE;
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

    let builder = BlockBuilder::new()
        .header(header)
        .payload(&[1u8, 2u8, 3u8, 4u8])
        .map_err(|_| StorageError::BackendIo { code: 240 })?;

    builder.build().map_err(|_| StorageError::BackendIo { code: 241 })
}

fn run_flow(storage: &mut impl StorageTrait) -> Result<bool, StorageError> {
    match storage.load_control_data() {
        Ok(_) => {}
        Err(StorageError::ControlPlaneUninitialized) => {
            storage.init(
                [7u8; PRIVATE_KEY_SIZE],
                1001,
                [9u8; INIT_PARAMS_SIZE],
            )?;
        }
        Err(err) => return Err(err),
    }

    let block = make_example_block()?;
    storage.save_block(EXAMPLE_STORAGE_INDEX, &block)?;
    let loaded = storage.read_block(EXAMPLE_STORAGE_INDEX)?;
    let expected = block.header();
    let actual = loaded.header();
    Ok(
        expected.version == actual.version
            && expected.sequence == actual.sequence
            && expected.creator == actual.creator
            && expected.payload_type == actual.payload_type,
    )
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
