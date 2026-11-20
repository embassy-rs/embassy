//! This example shows how async gpio can be used with a RP235x.
//!
//! The LED on the RP Pico W board is connected differently. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::peripherals::PIN_16;
use embassy_rp::{bind_interrupts, gpio};
use embassy_time::Timer;
use gpio::{Input, Level, Output, Pull};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    IO_IRQ_BANK0 => gpio::InterruptHandler<PIN_16>;
});

/// It requires an external signal to be manually triggered on PIN 16. For
/// example, this could be accomplished using an external power source with a
/// button so that it is possible to toggle the signal from low to high.
///
/// This example will begin with turning on the LED on the board and wait for a
/// high signal on PIN 16. Once the high event/signal occurs the program will
/// continue and turn off the LED, and then wait for 2 seconds before completing
/// the loop and starting over again.
#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);
    let mut async_input = Input::new_async(p.PIN_16, Irqs, Pull::None);

    loop {
        info!("wait_for_high. Turn on LED");
        led.set_high();

        async_input.wait_for_high().await;

        info!("done wait_for_high. Turn off LED");
        led.set_low();

        Timer::after_secs(2).await;
    }
}
