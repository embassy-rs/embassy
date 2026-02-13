#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_mcxa::bind_interrupts;
use hal::rtc::{InterruptHandler, Rtc, RtcDateTime};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    RTC => InterruptHandler<hal::rtc::Rtc0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    let rtc_config = hal::rtc::get_default_config();

    let mut rtc = Rtc::new(p.RTC0, Irqs, rtc_config);

    let now = RtcDateTime {
        year: 2025,
        month: 10,
        day: 15,
        hour: 14,
        minute: 30,
        second: 0,
    };

    rtc.stop();

    rtc.set_datetime(now);

    let mut alarm = now;
    alarm.second += 1;

    rtc.wait_for_alarm(alarm).await;
    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
