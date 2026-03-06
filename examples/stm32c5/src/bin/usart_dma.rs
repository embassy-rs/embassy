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
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    LPDMA1_CH0 => dma::InterruptHandler<peripherals::LPDMA1_CH0>;
    LPDMA1_CH1 => dma::InterruptHandler<peripherals::LPDMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let config = Config::default();
    let mut usart = Uart::new(p.USART2, p.PD6, p.PD5, p.LPDMA1_CH0, p.LPDMA1_CH1, Irqs, config).unwrap();
    info!("Usart DMA example");

    for n in 0u32.. {
        let mut tx: String<128> = String::new();

        core::write!(&mut tx, "Hello DMA World {}!\r\n", n).unwrap();

        usart.write(tx.as_bytes()).await.ok();

        info!("wrote DMA");
    }
}
