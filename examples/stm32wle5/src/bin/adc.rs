#![no_std]
#![no_main]

use defmt::*;
#[cfg(feature = "defmt-rtt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::low_power;
use embassy_time::Timer;
use panic_probe as _;
use static_cell::StaticCell;

#[embassy_executor::main(executor = "low_power::Executor")]
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

    #[cfg(feature = "defmt-serial")]
    {
        use embassy_stm32::mode::Blocking;
        use embassy_stm32::usart::Uart;
        let config = embassy_stm32::usart::Config::default();
        let uart = Uart::new_blocking(p.LPUART1, p.PC0, p.PC1, config).expect("failed to configure UART!");
        static SERIAL: StaticCell<Uart<'static, Blocking>> = StaticCell::new();
        defmt_serial::defmt_serial(SERIAL.init(uart));
    }

    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1);
    let mut pin = p.PA10;

    let mut vrefint = adc.enable_vrefint();
    let vrefint_sample = adc.blocking_read(&mut vrefint, SampleTime::CYCLES79_5);
    let convert_to_millivolts = |sample| {
        // From https://www.st.com/resource/en/datasheet/stm32g031g8.pdf
        // 6.3.3 Embedded internal reference voltage
        const VREFINT_MV: u32 = 1212; // mV

        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    };

    loop {
        let v = adc.blocking_read(&mut pin, SampleTime::CYCLES79_5);
        info!("--> {} - {} mV", v, convert_to_millivolts(v));
        Timer::after_secs(1).await;
    }
}
