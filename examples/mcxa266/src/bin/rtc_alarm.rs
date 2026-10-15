#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::bind_interrupts;
use embassy_time::Timer;
use hal::peripherals::RTC0;
use hal::rtc::{DateTime, InterruptHandler, Rtc};
use panic_probe as _;

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

    defmt::info!("Time set to: 2025-10-15 14:30:00");
    rtc.set_datetime(now);

    defmt::info!("Wait for 15 seconds");
    Timer::after_secs(15).await;

    let mut alarm = now;
    alarm.second += 20;

    // SR[TAF] is set when TSR equals TAR *and then increments* (RM 31.5.1.7),
    // so the alarm lands as the clock ticks to 14:30:21, about 6 s from here.
    defmt::info!("Alarm set for: 2025-10-15 14:30:20, waiting...");

    rtc.wait_for_alarm(alarm).await;
    let at = rtc.get_datetime();
    defmt::info!(
        "*** ALARM TRIGGERED at {=u8}:{=u8}:{=u8} ***",
        at.hour,
        at.minute,
        at.second
    );

    defmt::info!("Example complete - Test PASSED!");
}
