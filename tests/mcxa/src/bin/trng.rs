#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

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

    // Note: this test might fail once every ~2^28 or so.

    // Static
    let mut trng = Trng::new_blocking_128(p.TRNG0.reborrow());
    let rand1 = trng.blocking_next_u32();
    let rand2 = trng.blocking_next_u32();
    assert_ne!(rand1, rand2);
    drop(trng);

    let mut trng = Trng::new_blocking_256(p.TRNG0.reborrow());
    let rand1 = trng.blocking_next_u32();
    let rand2 = trng.blocking_next_u32();
    assert_ne!(rand1, rand2);
    drop(trng);

    let mut trng = Trng::new_blocking_512(p.TRNG0.reborrow());
    let rand1 = trng.blocking_next_u32();
    let rand2 = trng.blocking_next_u32();
    assert_ne!(rand1, rand2);
    drop(trng);

    // Blocking
    let config = trng::Config::default();
    let mut trng = Trng::new_blocking_with_custom_config(p.TRNG0.reborrow(), config);

    let rand1 = trng.blocking_next_u32();
    let rand2 = trng.blocking_next_u32();
    assert_ne!(rand1, rand2);

    let rand1 = trng.blocking_next_u64();
    let rand2 = trng.blocking_next_u64();
    assert_ne!(rand1, rand2);

    let rand1 = trng.next_u32();
    let rand2 = trng.next_u32();
    assert_ne!(rand1, rand2);

    let rand1 = trng.next_u64();
    let rand2 = trng.next_u64();
    assert_ne!(rand1, rand2);
    drop(trng);

    // Async
    let mut trng = Trng::new_with_custom_config(p.TRNG0.reborrow(), Irqs, config);

    let rand1 = trng.async_next_u32().await.unwrap();
    let rand2 = trng.async_next_u32().await.unwrap();
    assert_ne!(rand1, rand2);

    let rand1 = trng.async_next_u64().await.unwrap();
    let rand2 = trng.async_next_u64().await.unwrap();
    assert_ne!(rand1, rand2);

    let mut rand1 = [0u8; 10];
    let mut rand2 = [0u8; 10];
    trng.async_fill_bytes(&mut rand1).await.unwrap();
    trng.async_fill_bytes(&mut rand2).await.unwrap();

    let mut rand1 = [0u32; 8];
    let mut rand2 = [0u32; 8];
    trng.async_next_block(&mut rand1).await.unwrap();
    trng.async_next_block(&mut rand2).await.unwrap();
    assert_ne!(rand1, rand2);
    drop(trng);

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
