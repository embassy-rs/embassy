#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

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

    rtc.set_datetime(now);

    let mut alarm = now;
    alarm.second += 1;

    rtc.wait_for_alarm(alarm).await;
    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
