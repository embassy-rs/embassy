// This example is configured for the nucleo-wl55jc board. Curret monitor should show just a few microamps when the device is in stop2 mode.
#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use defmt::*;
#[cfg(feature = "defmt-rtt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::SharedData;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use panic_probe as _;

#[unsafe(link_section = ".shared_data.0")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[embassy_executor::main(executor = "embassy_stm32::Executor", entry = "cortex_m_rt::entry")]
async fn async_main(_spawner: Spawner) {
    let p = embassy_stm32::init_secondary(&SHARED_DATA);

    #[cfg(feature = "defmt-serial")]
    {
        use embassy_stm32::mode::Blocking;
        use embassy_stm32::usart::Uart;
        use static_cell::StaticCell;
        let config = embassy_stm32::usart::Config::default();
        let uart = Uart::new_blocking(p.LPUART1, p.PA3, p.PA2, config).expect("failed to configure UART!");
        static SERIAL: StaticCell<Uart<'static, Blocking>> = StaticCell::new();
        defmt_serial::defmt_serial(SERIAL.init(uart));
    }

    info!("Hello World!");

    let mut led = Output::new(p.PB11, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(100).await;

        info!("low");
        led.set_low();
        Timer::after_millis(4900).await;
    }
}
