#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_stm32::peripherals;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use static_cell::StaticCell;

#[cortex_m_rt::entry]
fn main() -> ! {
    embassy_stm32::low_power::Executor::take().run(|spawner| {
        spawner.spawn(async_main(spawner)).unwrap();
    });
}

#[embassy_executor::task]
async fn async_main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // when enabled the power-consumption is much higher during stop, but debugging and RTT is working
    config.enable_debug_during_sleep = true;
    // Enable LSE and configure RTC to use LSE
    config.rcc.ls = embassy_stm32::rcc::LsConfig::default_lse();
    config.rcc.ls.rtc = embassy_stm32::rcc::RtcClockSource::LSE;
    let p = embassy_stm32::init(config);

    static RTC: StaticCell<Rtc> = StaticCell::new();
    static HSEM: StaticCell<embassy_stm32::hsem::HardwareSemaphore<peripherals::HSEM>> = StaticCell::new();
    let rtc = RTC.init(Rtc::new(p.RTC, RtcConfig::default()));
    let hsem = HSEM.init(embassy_stm32::hsem::HardwareSemaphore::new(p.HSEM));

    embassy_stm32::low_power::stop_with_rtc_and_hsem(rtc, hsem);

    info!("Hello World!");

    let mut led = Output::new(p.PB0, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(1000).await;

        info!("low");
        led.set_low();
        Timer::after_millis(1000).await;
    }
}
