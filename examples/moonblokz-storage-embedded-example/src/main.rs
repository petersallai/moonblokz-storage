/*! RP2040 embedded example for MoonBlokz storage lifecycle usage.

Behavior:
- check initialization state,
- initialize when missing,
- save/read a block,
- signal result on LED:
  - success: one 0.5 second blink,
  - failure: three blinks.
*/

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_rp::gpio::{Level, Output};
use moonblokz_chain_types::{Block, BlockBuilder, BlockHeader};
use moonblokz_crypto::PRIVATE_KEY_SIZE;
use moonblokz_storage::{INIT_PARAMS_SIZE, Rp2040Backend, StorageError, StorageTrait};
use panic_halt as _;

const RP2040_FLASH_SIZE: usize = 2 * 1024 * 1024;
const DATA_STORAGE_START_ADDRESS: usize = 1536 * 1024;
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
            storage.init([7u8; PRIVATE_KEY_SIZE], 1001, [9u8; INIT_PARAMS_SIZE])?;
        }
        Err(err) => return Err(err),
    }

    let block = make_example_block()?;
    storage.save_block(EXAMPLE_STORAGE_INDEX, &block)?;
    let loaded = storage.read_block(EXAMPLE_STORAGE_INDEX)?;
    let expected = block.header();
    let actual = loaded.header();
    Ok(expected.version == actual.version
        && expected.sequence == actual.sequence
        && expected.creator == actual.creator
        && expected.payload_type == actual.payload_type)
}

fn delay_ms(ms: u32) {
    // RP2040 default system clock is typically 125 MHz.
    // Approximate CPU cycles per millisecond for simple LED timing.
    let cycles_per_ms: u32 = 125_000;
    cortex_m::asm::delay(ms.saturating_mul(cycles_per_ms));
}

#[entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    let result = match Rp2040Backend::<RP2040_FLASH_SIZE>::new(p.FLASH, DATA_STORAGE_START_ADDRESS) {
        Ok(mut storage) => run_flow(&mut storage),
        Err(err) => Err(err),
    };

    match result {
        Ok(true) => {
            led.set_high();
            delay_ms(500);
            led.set_low();
        }
        _ => {
            let mut count = 0u8;
            while count < 3 {
                led.set_high();
                delay_ms(200);
                led.set_low();
                delay_ms(200);
                count += 1;
            }
        }
    }

    loop {
        delay_ms(1000);
    }
}
