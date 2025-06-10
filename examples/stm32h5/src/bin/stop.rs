// Notice:
// the MCU might need an extra reset to make the code actually running

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AnyPin, Level, Output, Speed};
use embassy_stm32::low_power::Executor;
use embassy_stm32::rcc::{HSIPrescaler, LsConfig};
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_stm32::{Config, Peri};
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    Executor::take().run(|spawner| {
        unwrap!(spawner.spawn(async_main(spawner)));
    })
}

#[embassy_executor::task]
async fn async_main(spawner: Spawner) {
    defmt::info!("Program Start");

    let mut config = Config::default();

    // System Clock seems need to be equal or lower than 16 MHz
    config.rcc.hsi = Some(HSIPrescaler::DIV4);

    config.rcc.ls = LsConfig::default_lsi();
    // when enabled the power-consumption is much higher during stop, but debugging and RTT is working
    // if you wan't to measure the power-consumption, or for production: uncomment this line
    // config.enable_debug_during_sleep = false;
    let p = embassy_stm32::init(config);

    // give the RTC to the executor...
    let rtc = Rtc::new(p.RTC, RtcConfig::default());
    static RTC: StaticCell<Rtc> = StaticCell::new();
    let rtc = RTC.init(rtc);
    embassy_stm32::low_power::stop_with_rtc(rtc);

    unwrap!(spawner.spawn(blinky(p.PB4.into())));
    unwrap!(spawner.spawn(timeout()));
}

#[embassy_executor::task]
async fn blinky(led: Peri<'static, AnyPin>) {
    let mut led = Output::new(led, Level::Low, Speed::Low);
    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}

// when enable_debug_during_sleep is false, it is more difficult to reprogram the MCU
// therefore we block the MCU after 30s to be able to reprogram it easily
#[embassy_executor::task]
async fn timeout() -> ! {
    Timer::after_secs(30).await;
    #[allow(clippy::empty_loop)]
    loop {}
}
