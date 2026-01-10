#![no_std]
#![no_main]

use defmt::*;
#[cfg(feature = "defmt-rtt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::Pull;
use embassy_stm32::{bind_interrupts, interrupt};
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(
    pub struct Irqs{
        EXTI0 => exti::InterruptHandler<interrupt::typelevel::EXTI0>;
});

#[embassy_executor::main(executor = "embassy_stm32::Executor", entry = "cortex_m_rt::entry")]
async fn async_main(_spawner: Spawner) {
    // delay to allow probe-rs to connect for flashing
    cortex_m::asm::delay(1_000_000);
    let mut config = embassy_stm32::Config::default();
    config.rcc.msi = Some(embassy_stm32::rcc::MSIRange::RANGE4M);
    config.rcc.sys = embassy_stm32::rcc::Sysclk::MSI;
    #[cfg(feature = "defmt-serial")]
    {
        // enable HSI clock
        config.rcc.hsi = true;
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

    let mut button = ExtiInput::new(p.PA0, p.EXTI0, Pull::Up, Irqs);

    info!("Press the USER button...");

    loop {
        button.wait_for_falling_edge().await;
        info!("Pressed!");
        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}
