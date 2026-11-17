#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_mcxa as hal;
use embassy_mcxa::bind_interrupts;
use embassy_time::Timer;
use hal::peripherals::RTC0;
use hal::rtc::{ClkSel, DateTime, InterruptHandler, Month, Rtc, Weekday};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    RTC0 => InterruptHandler<RTC0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("=== RTC Alarm Example ===");
    let mut config = hal::config::Config::default();
    // VBAT POR selects OSC32K; system resets can retain either selection.
    // Keep default FRO16K and the crystal available before adopting the RTC.
    // OSC32K defaults use the SDK 26.09 startup profile, then LP operation;
    // supply, crystal/PCB loading, drive, and startup require qualification.
    config.clock_cfg.osc32k = Some(Default::default());
    let p = hal::init(config);
    defmt::info!("Board clocks initialized; adopting RTC");

    let rtc_config = hal::rtc::Config::default().with_clksel(ClkSel::Clk32768);
    let (mut rtc, time_provider) = Rtc::new(p.RTC0, Irqs, rtc_config).unwrap();

    let now = DateTime {
        year: 2026,
        month: Month::March,
        dow: Weekday::Wednesday,
        day: 11,
        hour: 14,
        minute: 30,
        second: 10,
    };

    defmt::info!("Time set to: 2026-03-11 14:30:10");
    rtc.set_datetime(now).unwrap();

    defmt::info!("Wait for 15 seconds");
    Timer::after_secs(15).await;

    let mut alarm = now;
    alarm.second += 20;

    defmt::info!("Alarm set for: 2026-03-11 14:30:30 (+20 seconds from initial time)");
    let (result, ()) = join(rtc.wait_for_alarm(alarm), async {
        Timer::after_secs(1).await;
        let at = time_provider.now().unwrap();
        defmt::info!(
            "Clock remains readable while waiting: {=u8}:{=u8}:{=u8}",
            at.hour,
            at.minute,
            at.second
        );
    })
    .await;
    result.unwrap();

    defmt::info!("Example complete - Test PASSED!");
}
