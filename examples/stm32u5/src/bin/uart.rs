#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{self, Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");
    let config = Config::default();
    let mut uart = Uart::new(p.USART1, p.PA10, p.PA9, Irqs, p.GPDMA1_CH0, p.GPDMA1_CH1, config).unwrap();
    let mut buffer = [0u8; 32];
    loop {
        if let Ok(len) = uart.read_until_idle(&mut buffer).await {
            info!("{}", &buffer[0..len]);
            uart.write(&buffer[0..len]).await.unwrap()
        }
    }
}
