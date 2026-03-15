#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::bind_interrupts;
use hal::peripherals::RTC0;
use hal::rtc::{DateTime, InterruptHandler, Rtc};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    RTC => InterruptHandler<RTC0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    defmt::info!("=== RTC Alarm Example ===");

    let mut rtc = Rtc::new(p.RTC0, Irqs, Default::default());

    let now = DateTime {
        year: 2025,
        month: 10,
        day: 15,
        hour: 14,
        minute: 30,
        second: 0,
    };

    rtc.stop();

    defmt::info!("Time set to: 2025-10-15 14:30:00");
    rtc.set_datetime(now);

    let mut alarm = now;
    alarm.second += 10;

    defmt::info!("Alarm set for: 2025-10-15 14:30:10 (+10 seconds)");
    defmt::info!("RTC started, waiting for alarm...");

    rtc.wait_for_alarm(alarm).await;
    defmt::info!("*** ALARM TRIGGERED! ***");

    defmt::info!("Example complete - Test PASSED!");
}
