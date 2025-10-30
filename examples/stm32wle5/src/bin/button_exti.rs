#![no_std]
#![no_main]

use defmt::*;
#[cfg(feature = "defmt-rtt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Pull;
use embassy_stm32::low_power::Executor;
use embassy_stm32::rtc::{Rtc, RtcConfig};
use panic_probe as _;
use static_cell::StaticCell;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("main: Starting!");
    Executor::take().run(|spawner| {
        spawner.spawn(unwrap!(async_main(spawner)));
    });
}

#[embassy_executor::task]
async fn async_main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // enable HSI clock
    config.rcc.hsi = true;
    // enable LSI clock for RTC
    config.rcc.ls = embassy_stm32::rcc::LsConfig::default_lsi();
    config.rcc.msi = Some(embassy_stm32::rcc::MSIRange::RANGE4M);
    config.rcc.sys = embassy_stm32::rcc::Sysclk::MSI;
    // enable ADC with HSI clock
    config.rcc.mux.adcsel = embassy_stm32::pac::rcc::vals::Adcsel::HSI;
    #[cfg(feature = "defmt-serial")]
    {
        // disable debug during sleep to reduce power consumption since we are
        // using defmt-serial on LPUART1.
        config.enable_debug_during_sleep = false;
        // if we are using defmt-serial on LPUART1, we need to use HSI for the clock
        // so that its registers are preserved during STOP modes.
        config.rcc.mux.lpuart1sel = embassy_stm32::pac::rcc::vals::Lpuart1sel::HSI;
    }
    // Initialize STM32WL peripherals (use default config like wio-e5-async example)
    let p = embassy_stm32::init(config);

    // start with all GPIOs as analog to reduce power consumption
    for r in [
        embassy_stm32::pac::GPIOA,
        embassy_stm32::pac::GPIOB,
        embassy_stm32::pac::GPIOC,
        embassy_stm32::pac::GPIOH,
    ] {
        r.moder().modify(|w| {
            for i in 0..16 {
                // don't reset these if probe-rs should stay connected!
                #[cfg(feature = "defmt-rtt")]
                if config.enable_debug_during_sleep && r == embassy_stm32::pac::GPIOA && [13, 14].contains(&i) {
                    continue;
                }
                w.set_moder(i, embassy_stm32::pac::gpio::vals::Moder::ANALOG);
            }
        });
    }
    #[cfg(feature = "defmt-serial")]
    {
        use embassy_stm32::mode::Blocking;
        use embassy_stm32::usart::Uart;
        let config = embassy_stm32::usart::Config::default();
        let uart = Uart::new_blocking(p.LPUART1, p.PC0, p.PC1, config).expect("failed to configure UART!");
        static SERIAL: StaticCell<Uart<'static, Blocking>> = StaticCell::new();
        defmt_serial::defmt_serial(SERIAL.init(uart));
    }

    // give the RTC to the low_power executor...
    let rtc_config = RtcConfig::default();
    let rtc = Rtc::new(p.RTC, rtc_config);
    static RTC: StaticCell<Rtc> = StaticCell::new();
    let rtc = RTC.init(rtc);
    embassy_stm32::low_power::stop_with_rtc(rtc);

    info!("Hello World!");

    let mut button = ExtiInput::new(p.PA0, p.EXTI0, Pull::Up);

    info!("Press the USER button...");

    loop {
        button.wait_for_falling_edge().await;
        info!("Pressed!");
        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}
