#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{self, Input, Pull};
use embassy_nrf::wdt::{Config, HaltConfig, Watchdog};
use embassy_nrf::{bind_interrupts, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    GPIOTE => gpio::InterruptHandler<peripherals::P0_11>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Hello World!");

    let mut config = Config::default();
    config.timeout_ticks = 32768 * 3; // 3 seconds

    // This is needed for `probe-rs run` to be able to catch the panic message
    // in the WDT interrupt. The core resets 2 ticks after firing the interrupt.
    config.action_during_debug_halt = HaltConfig::Pause;

    let (_wdt, [mut handle]) = match Watchdog::try_new(p.WDT, config) {
        Ok(x) => x,
        Err(_) => {
            info!("Watchdog already active with wrong config, waiting for it to timeout...");
            loop {}
        }
    };

    let mut button = Input::new_async(p.P0_11, Irqs, Pull::Up);

    info!("Watchdog started, press button 1 to feed it or I'll reset in 3 seconds!");

    loop {
        button.wait_for_high().await;
        button.wait_for_low().await;
        info!("Button pressed, feeding watchdog!");
        handle.feed();
    }
}
