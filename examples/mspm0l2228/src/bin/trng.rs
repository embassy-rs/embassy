#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::Config;
use embassy_mspm0::gpio::{Level, Output};
use embassy_mspm0::trng::{CryptoDecimRate, Trng};
use embassy_time::Timer;
use rand_core::TryRngCore;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_mspm0::init(Config::default());

    let mut trng = expect!(
        Trng::new_secure(p.TRNG, CryptoDecimRate::Decim6),
        "Failed to initialize RNG"
    );
    // Alternatively, use the default crypto-secure decimation rate (Decim4).
    // let mut trng = expect!(Trng::new(p.TRNG), "Failed to initialize RNG");

    // A buffer to collect random bytes in.
    let mut randomness = [0u8; 16];

    let mut led1 = Output::new(p.PA0, Level::Low);
    led1.set_inversion(true);

    loop {
        unwrap!(trng.try_fill_bytes(&mut randomness[..8]));
        unwrap!(trng.async_read_bytes(&mut randomness[8..]).await);
        info!("Random bytes {}", &randomness);
        let random_u32 = unwrap!(trng.try_next_u32());
        let random_u64 = unwrap!(trng.try_next_u64());
        info!("Random u32 {} u64 {}", random_u32, random_u64);
        // Random number of blinks between 0 and 31
        let blinks = random_u32 % 32;
        for _ in 0..blinks * 2 {
            led1.toggle();
            Timer::after_millis(20).await;
        }
        Timer::after_millis(1000).await;
    }
}
