#![no_std]
#![no_main]

use core::fmt::Write;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, dma, peripherals, usart};
use heapless::String;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART4 => usart::InterruptHandler<peripherals::UART4>;
    DMA1_CHANNEL3 => dma::InterruptHandler<peripherals::DMA1_CH3>;
    DMA1_CHANNEL4 => dma::InterruptHandler<peripherals::DMA1_CH4>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let config = Config::default();
    let mut usart = Uart::new(p.UART4, p.PA1, p.PA0, p.DMA1_CH3, p.DMA1_CH4, Irqs, config).unwrap();

    for n in 0u32.. {
        let mut s: String<128> = String::new();
        core::write!(&mut s, "Hello DMA World {}!\r\n", n).unwrap();

        info!("Writing...");
        usart.write(s.as_bytes()).await.ok();

        info!("wrote DMA");
    }
}
