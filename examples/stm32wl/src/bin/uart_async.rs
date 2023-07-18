#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config, InterruptHandler, Uart};
use embassy_stm32::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    USART1 => InterruptHandler<peripherals::USART1>;
    LPUART1 => InterruptHandler<peripherals::LPUART1>;
});

/*
Pass Incoming data from LPUART1 to USART1
Example is written for the LoRa-E5 mini v1.0,
but can be surely changed for your needs.
*/
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.mux = embassy_stm32::rcc::ClockSrc::HSE32;
    let p = embassy_stm32::init(config);

    defmt::info!("Starting system");

    let mut config1 = Config::default();
    config1.baudrate = 9600;

    let mut config2 = Config::default();
    config2.baudrate = 9600;

    //RX/TX connected to USB/UART Bridge on LoRa-E5 mini v1.0
    let mut usart1 = Uart::new(p.USART1, p.PB7, p.PB6, Irqs, p.DMA1_CH3, p.DMA1_CH4, config1);

    //RX1/TX1 (LPUART) on LoRa-E5 mini v1.0
    let mut usart2 = Uart::new(p.LPUART1, p.PC0, p.PC1, Irqs, p.DMA1_CH5, p.DMA1_CH6, config2);

    unwrap!(usart1.write(b"Hello Embassy World!\r\n").await);
    unwrap!(usart2.write(b"Hello Embassy World!\r\n").await);

    let mut buf = [0u8; 300];
    loop {
        let result = usart2.read_until_idle(&mut buf).await;
        match result {
            Ok(size) => {
                match usart1.write(&buf[0..size]).await {
                    Ok(()) => {
                        //Write suc.
                    }
                    Err(..) => {
                        //Wasn't able to write
                    }
                }
            }
            Err(_err) => {
                //Ignore eg. framing errors
            }
        }
    }
}
