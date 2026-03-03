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
        timeout: Duration::from_millis(1050),
        warning: Some(Duration::from_micros(4000)),
    };

    let mut watchdog = Watchdog::new(p.WWDT0, Irqs, wwdt_config).unwrap();
    let mut led = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Fast);

    // Set the LED high for 2 seconds so we know when we're about to start the watchdog
    led.toggle();
    Timer::after_secs(2).await;

    // Set to watchdog to generate interrupt if it's not fed within 1.05 seconds, and start it.
    // The warning interrupt will trigger 4ms before the timeout.
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

    defmt::info!("Stopped feeding, watchdog interrupt will be triggered in 1 second");
    // Blink 10 times per second, not feeding the watchdog.
    // Watchdog timer will trigger after 1.0 second as warning is set to 50ms.
    loop {
        led.toggle();
        Timer::after_millis(100).await;
        led.toggle();
        Timer::after_millis(100).await;
    }
}
