#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_mcxa as hal;
use embassy_time::{Duration, Timer, with_timeout};
use hal::rtc::{ClkSel, DateTime, InterruptHandler, Month, Rtc, Weekday};
use panic_probe as _;

hal::bind_interrupts!(struct Irqs {
    RTC0 => InterruptHandler<hal::peripherals::RTC0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("Starting MCXA287 RTC example");
    let mut config = hal::config::Config::default();
    // Keep both FRO16K and OSC32K available during initialization. RTC retains
    // its clock selection across system resets, and a stopped selected clock
    // prevents access even to the register that would change that selection.
    // SDK 26.09 reference startup: 12 pF per pin, gain 01, VBAT > 3.0 V,
    // followed by LP operation. Crystal/PCB loading and drive need qualification.
    config.clock_cfg.osc32k = Some(Default::default());
    let p = hal::init(config);
    defmt::info!("Board clocks initialized; adopting RTC");

    let rtc_config = hal::rtc::Config::default().with_clksel(ClkSel::Clk32768);
    let (mut rtc, time_provider) = Rtc::new(p.RTC0, Irqs, rtc_config).unwrap();
    let now = DateTime {
        year: 2026,
        month: Month::November,
        dow: Weekday::Friday,
        day: 13,
        hour: 12,
        minute: 34,
        second: 57,
    };
    rtc.set_datetime(now).unwrap();
    defmt::info!("RTC time set: {:?}", time_provider.now().unwrap());

    let mut alarm = now;
    alarm.minute = 35;
    alarm.second = 0;
    defmt::info!("Waiting for alarm across the minute rollover: {:?}", alarm);

    let (result, ()) = join(with_timeout(Duration::from_secs(5), rtc.wait_for_alarm(alarm)), async {
        Timer::after_secs(1).await;
        defmt::info!(
            "Clock remains readable while waiting: {:?}",
            time_provider.now().unwrap()
        );
    })
    .await;
    result.expect("RTC alarm timed out").unwrap();

    let at = time_provider.now().unwrap();
    assert_eq!(at.hour, 12);
    assert_eq!(at.minute, 35);
    assert!(at.second <= 1);
    defmt::info!("Alarm fired: {:?}", at);
    defmt::info!("RTC example complete - Test PASSED!");
    if cortex_m::peripheral::DCB::is_debugger_attached() {
        cortex_m::asm::bkpt();
    }
}
