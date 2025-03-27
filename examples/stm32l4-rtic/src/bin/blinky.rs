#![no_std]
#![no_main]

use rtic_monotonics::stm32::prelude::*;
use {defmt_rtt as _, panic_probe as _};

stm32_tim2_monotonic!(Mono, 1_000_000);

// setting up the RTIC application with a `software task` using the SPI HW interrupt
#[rtic::app(device = embassy_stm32, peripherals = true, dispatchers = [SPI1])]
mod app {
    use defmt::info;
    use embassy_stm32::gpio::{Level, Output, Speed};

    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        info!("Hello World!");

        let device_config = embassy_stm32::Config::default();
        let stm32_peripherals = embassy_stm32::init(device_config);

        let timer2_frequency = embassy_stm32::rcc::frequency::<embassy_stm32::peripherals::TIM2>();
        info!("Timer2 clock frequency: {} Hz", timer2_frequency);

        // Initialize the systick interrupt & obtain the token to prove that we did
        Mono::start(timer2_frequency.0);

        let led = Output::new(stm32_peripherals.PB14, Level::High, Speed::Low);

        blink::spawn(led).ok();

        (Shared {}, Local {})
    }

    // Using a 'software task' to blink the LED
    #[task(priority = 1)]
    async fn blink(_cx: blink::Context, mut led: Output<'static>) {
        loop {
            led.toggle();
            Mono::delay(100.millis()).await;
        }
    }
}
