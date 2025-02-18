//! This example shows TRNG usage

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::TRNG;
use embassy_rp::trng::Trng;
use embassy_time::Timer;
use rand::RngCore;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TRNG_IRQ => embassy_rp::trng::InterruptHandler<TRNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    // Initialize the TRNG with default configuration
    let mut trng = Trng::new(peripherals.TRNG, Irqs, embassy_rp::trng::Config::default());
    // A buffer to collect random bytes in.
    let mut randomness = [0u8; 58];

    let mut led = Output::new(peripherals.PIN_25, Level::Low);

    loop {
        trng.fill_bytes(&mut randomness).await;
        info!("Random bytes async {}", &randomness);
        trng.blocking_fill_bytes(&mut randomness);
        info!("Random bytes blocking {}", &randomness);
        let random_u32 = trng.next_u32();
        let random_u64 = trng.next_u64();
        info!("Random u32 {} u64 {}", random_u32, random_u64);
        // Random number of blinks between 0 and 31
        let blinks = random_u32 % 32;
        for _ in 0..blinks {
            led.set_high();
            Timer::after_millis(20).await;
            led.set_low();
            Timer::after_millis(20).await;
        }
        Timer::after_millis(1000).await;
    }
}
