#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::bind_interrupts;
use hal::peripherals::RTC0;
use hal::rtc::{DateTime, InterruptHandler, Month, Rtc, Weekday};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    RTC => InterruptHandler<RTC0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    defmt::info!("=== RTC Alarm Example ===");

    let mut rtc = Rtc::new(p.RTC0, Irqs, Default::default()).unwrap();

    let now = DateTime {
        year: 2026,
        month: Month::March,
        dow: Weekday::Wednesday,
        day: 11,
        hour: 14,
        minute: 30,
        second: 42,
    };

    defmt::info!("Time set to: 2026-03-11 14:30:42");
    rtc.set_datetime(now).unwrap();

    let mut alarm = now;
    alarm.second += 10;

    defmt::info!("Alarm set for: 2026-03-11 14:30:52 (+10 seconds)");
    rtc.wait_for_alarm(alarm).await.unwrap();

    defmt::info!("Example complete - Test PASSED!");
}
