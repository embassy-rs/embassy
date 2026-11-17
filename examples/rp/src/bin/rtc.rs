//! This example shows how to use RTC (Real Time Clock) in the RP2040 chip.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::rtc::Rtc;
use embassy_time::Timer;
use jiff::civil::date;
use panic_probe as _;

// Bind the RTC interrupt to the handler
bind_interrupts!(struct Irqs {
    RTC_IRQ => embassy_rp::rtc::InterruptHandler;
});

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Wait for 20s");

    let mut rtc = Rtc::new(p.RTC, Irqs);

    if !rtc.is_running() {
        info!("Start RTC");
        rtc.set(date(2000, 1, 1).at(0, 0, 0, 0)).unwrap();
    }

    Timer::after_millis(20000).await;

    if let Ok(dt) = rtc.read() {
        info!(
            "Now: {}-{:02}-{:02} {}:{:02}:{:02}",
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second(),
        );
    }

    info!("Reboot.");
    Timer::after_millis(200).await;
    cortex_m::peripheral::SCB::sys_reset();
}
