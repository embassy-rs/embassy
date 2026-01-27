#![no_std]
#![no_main]

use defmt::*;
#[cfg(feature = "defmt-rtt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::i2c::I2c;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, dma, i2c, peripherals};
use embassy_time::{Duration, Timer};
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs{
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
    DMA1_CHANNEL6 => dma::InterruptHandler<peripherals::DMA1_CH6>;
    DMA1_CHANNEL7 => dma::InterruptHandler<peripherals::DMA1_CH7>;
});

#[embassy_executor::main(executor = "embassy_stm32::Executor", entry = "cortex_m_rt::entry")]
async fn async_main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // enable HSI clock
    config.rcc.hsi = true;
    // enable LSI clock for RTC
    config.rcc.ls = embassy_stm32::rcc::LsConfig::default_lsi();
    config.rcc.msi = Some(embassy_stm32::rcc::MSIRange::RANGE4M);
    config.rcc.sys = embassy_stm32::rcc::Sysclk::MSI;
    // enable ADC with HSI clock
    config.rcc.mux.i2c2sel = embassy_stm32::pac::rcc::vals::I2c2sel::HSI;
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
    let en3v3 = embassy_stm32::gpio::Output::new(
        p.PA9,
        embassy_stm32::gpio::Level::High,
        embassy_stm32::gpio::Speed::High,
    );
    core::mem::forget(en3v3); // keep the output pin enabled

    let mut i2c = I2c::new(p.I2C2, p.PB15, p.PA15, p.DMA1_CH6, p.DMA1_CH7, Irqs, {
        let mut config = i2c::Config::default();
        config.frequency = Hertz::khz(100);
        config.timeout = Duration::from_millis(1000);
        config
    });

    loop {
        let mut buffer = [0; 2];
        // read the temperature register of the onboard lm75
        match i2c.read(0x48, &mut buffer).await {
            Ok(_) => info!("--> {:?}", buffer),
            Err(e) => info!("--> Error: {:?}", e),
        }
        Timer::after_secs(5).await;
    }
}
