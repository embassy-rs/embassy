#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_time::{Duration, Instant};
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::hash::*;
use sha2::{Digest, Sha256};

const TEST_STRING_1: &[u8] = b"hello world";

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    let hw_start_time = Instant::now();

    // Compute a digest in hardware.
    let mut hw_hasher = Hash::new(p.HASH);
    let mut context = hw_hasher.start(Algorithm::SHA256, DataType::Width8);
    hw_hasher.update(&mut context, TEST_STRING_1);
    let mut buffer: [u8; 32] = [0; 32];
    let hw_digest = hw_hasher.finish(context, &mut buffer);

    let hw_end_time = Instant::now();
    let hw_execution_time = hw_end_time - hw_start_time;

    let sw_start_time = Instant::now();

    // Compute a digest in software.
    let mut sw_hasher = Sha256::new();
    sw_hasher.update(TEST_STRING_1);
    let sw_digest = sw_hasher.finalize();

    let sw_end_time = Instant::now();
    let sw_execution_time = sw_end_time - sw_start_time;

    info!("Hardware Digest: {:?}", hw_digest);
    info!("Software Digest: {:?}", sw_digest[..]);
    info!("Hardware Execution Time: {:?}", hw_execution_time);
    info!("Software Execution Time: {:?}", sw_execution_time);
    assert_eq!(*hw_digest, sw_digest[..]);

    loop {}
}
