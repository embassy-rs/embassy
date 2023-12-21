#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut usart = Uart::new(p.USART1, p.PB7, p.PB6, Irqs, p.DMA1_CH2, p.DMA1_CH3, Config::default()).unwrap();

    usart.write(b"Hello Embassy World!\r\n").await.unwrap();
    info!("wrote Hello, starting echo");

    let mut buf = [0; 1];
    loop {
        usart.read(&mut buf[..]).await.unwrap();
        usart.write(&buf[..]).await.unwrap();
    }
}
