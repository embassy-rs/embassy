#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nxp::gpio::{Input, Level, Output, Pull};
use {defmt_rtt as _, embassy_imxrt1062_evk_examples as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nxp::init(Default::default());
    info!("Hello world!");

    // User LED (D8)
    let led = p.GPIO_AD_B0_08;
    // User button (SW5)
    let button = p.WAKEUP;
    let mut button = Input::new(button, Pull::Up100K);
    let mut led = Output::new(led, Level::Low);
    led.set_high();

    loop {
        button.wait_for_falling_edge().await;

        info!("Toggled");
        led.toggle();

        // Software debounce.
        button.wait_for_rising_edge().await;

        // Stabilization.
        for _ in 0..100_000 {
            cortex_m::asm::nop();
        }
    }
}
