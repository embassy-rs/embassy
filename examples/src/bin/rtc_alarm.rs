#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa as hal;
use hal::rtc::{RtcDateTime, RtcInterruptEnable};
use hal::InterruptExt;

type MyRtc = hal::rtc::Rtc<'static, hal::rtc::Rtc0>;

use embassy_mcxa::bind_interrupts;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RTC => hal::rtc::RtcHandler;
});

#[used]
#[no_mangle]
static KEEP_RTC: unsafe extern "C" fn() = RTC;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    defmt::info!("=== RTC Alarm Example ===");

    let rtc_config = hal::rtc::get_default_config();

    let rtc = MyRtc::new(p.RTC0, rtc_config);

    let now = RtcDateTime {
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

    rtc.set_alarm(alarm);
    defmt::info!("Alarm set for: 2025-10-15 14:30:10 (+10 seconds)");

    rtc.set_interrupt(RtcInterruptEnable::RTC_ALARM_INTERRUPT_ENABLE);

    unsafe {
        hal::interrupt::RTC.enable();
    }

    unsafe {
        cortex_m::interrupt::enable();
    }

    rtc.start();

    defmt::info!("RTC started, waiting for alarm...");

    loop {
        if rtc.is_alarm_triggered() {
            defmt::info!("*** ALARM TRIGGERED! ***");
            break;
        }
    }

    defmt::info!("Example complete - Test PASSED!");
}
