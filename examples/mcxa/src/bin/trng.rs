#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::bind_interrupts;
use hal::config::Config;
use hal::trng::{self, InterruptHandler, Trng};
use rand_core::RngCore;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        TRNG0 => InterruptHandler;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let mut p = hal::init(config);

    defmt::info!("TRNG example");

    let mut trng = Trng::new_blocking_128(p.TRNG0.reborrow());
    let rand = trng.blocking_next_u32();
    defmt::info!("128-bit {}", rand);

    drop(trng);

    let mut trng = Trng::new_blocking_256(p.TRNG0.reborrow());
    let rand = trng.blocking_next_u32();
    defmt::info!("256-bit {}", rand);

    drop(trng);

    let mut trng = Trng::new_blocking_512(p.TRNG0.reborrow());
    let rand = trng.blocking_next_u32();
    defmt::info!("512-bit {}", rand);

    drop(trng);

    let config = trng::Config::default();
    let mut trng = Trng::new_blocking_with_custom_config(p.TRNG0.reborrow(), config);

    defmt::info!("========== BLOCKING ==========");

    defmt::info!("Generate 10 u32");
    for _ in 0..10 {
        let rand = trng.blocking_next_u32();
        defmt::info!("{}", rand);
    }

    defmt::info!("Generate 10 u64");
    for _ in 0..10 {
        let rand = trng.blocking_next_u64();
        defmt::info!("{}", rand);
    }

    let mut buf = [0_u8; 256];

    defmt::info!("Generate 10 256-byte buffers");
    for _ in 0..10 {
        trng.blocking_fill_bytes(&mut buf);
        defmt::info!("{:02x}", buf);
    }

    defmt::info!("RngCore");

    for _ in 0..10 {
        defmt::info!("u32: {}", trng.next_u32());
        defmt::info!("u64: {}", trng.next_u64());
    }

    drop(trng);

    defmt::info!("========== ASYNC ==========");

    let mut trng = Trng::new_with_custom_config(p.TRNG0.reborrow(), Irqs, config);

    defmt::info!("Generate 10 u32");
    for _ in 0..10 {
        let rand = trng.async_next_u32().await.unwrap();
        defmt::info!("{}", rand);
    }

    defmt::info!("Generate 10 u64");
    for _ in 0..10 {
        let rand = trng.async_next_u64().await.unwrap();
        defmt::info!("{}", rand);
    }

    let mut buf = [0_u8; 256];

    defmt::info!("Generate 10 256-byte buffers");
    for _ in 0..10 {
        trng.async_fill_bytes(&mut buf).await.unwrap();
        defmt::info!("{:02x}", buf);
    }

    defmt::info!("RngCore");

    for _ in 0..10 {
        defmt::info!("u32: {}", trng.next_u32());
        defmt::info!("u64: {}", trng.next_u64());
    }
}
