#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use hal::bind_interrupts;
use hal::config::Config;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use hal::wwdt::{InterruptHandler, Watchdog};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        WWDT0 => InterruptHandler;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = hal::init(config);

    defmt::info!("Watchdog example");

    let wwdt_config = hal::wwdt::Config {
        timeout: Duration::from_millis(4000),
        warning: None,
    };

    let mut watchdog = Watchdog::new(p.WWDT0, Irqs, wwdt_config).unwrap();
    let mut led = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Fast);

    // Set the LED high for 2 seconds so we know when we're about to start the watchdog
    led.toggle();
    Timer::after_secs(2).await;

    // Set to watchdog to reset if it's not fed within 4 seconds, and start it
    watchdog.start();
    defmt::info!("Started the watchdog timer");

    // Blink once a second for 5 seconds, feed the watchdog timer once a second to avoid a reset
    for _ in 1..=5 {
        led.toggle();
        Timer::after_millis(500).await;
        led.toggle();
        Timer::after_millis(500).await;
        defmt::info!("Feeding watchdog");
        watchdog.feed();
    }

    defmt::info!("Stopped feeding, device will reset in 4 seconds");
    // Blink 10 times per second, not feeding the watchdog.
    // The processor should reset in 4 seconds.
    loop {
        led.toggle();
        Timer::after_millis(100).await;
        led.toggle();
        Timer::after_millis(100).await;
    }
}
