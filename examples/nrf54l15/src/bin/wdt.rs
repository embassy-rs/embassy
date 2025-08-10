#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::wdt::{Config, HaltConfig, Watchdog};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Hello WDT");

    const TIMEOUT_S: u32 = 5;

    let mut config = Config::default();
    config.timeout_ticks = 32768 * TIMEOUT_S;

    // This is needed for `probe-rs run` to be able to catch the panic message
    // in the WDT interrupt. The core resets 2 ticks after firing the interrupt.
    config.action_during_debug_halt = HaltConfig::PAUSE;

    // The nrf54l15 has two watchdogs. Only one (WDT) is available in non-secure (ns) mode, as the
    // other is reserved for the secure (s) environment. In secure mode, both are available as WDT0
    // and WDT1.
    info!("Watchdog launched with {} s timeout", TIMEOUT_S);
    let (_wdt, [mut handle]) = match Watchdog::try_new(p.WDT1, config) {
        Ok(x) => x,
        Err(_) => {
            info!("Watchdog already active with wrong config, waiting for it to timeout...");
            loop {}
        }
    };

    for wait in 1..=TIMEOUT_S {
        info!("Waiting {} seconds ...", wait);
        Timer::after_secs(wait as u64).await;
        handle.pet();
        info!("Pet watchdog");
    }
}
