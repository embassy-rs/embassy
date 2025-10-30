#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::flexcomm::uart::{Blocking, Uart, UartRx, UartTx};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn usart4_task(mut uart: UartRx<'static, Blocking>) {
    info!("RX Task");

    loop {
        let mut buf = [0; 8];

        Timer::after_millis(10).await;

        uart.blocking_read(&mut buf).unwrap();

        let s = core::str::from_utf8(&buf).unwrap();

        info!("Received '{}'", s);
    }
}

#[embassy_executor::task]
async fn usart2_task(mut uart: UartTx<'static, Blocking>) {
    info!("TX Task");

    loop {
        let buf = "Testing\0".as_bytes();

        uart.blocking_write(buf).unwrap();

        Timer::after_millis(10).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("UART test start");

    let usart4 = Uart::new_blocking(p.FLEXCOMM4, p.PIO0_29, p.PIO0_30, Default::default()).unwrap();

    let (_, usart4) = usart4.split();
    spawner.spawn(usart4_task(usart4).unwrap());

    let usart2 = UartTx::new_blocking(p.FLEXCOMM2, p.PIO0_15, Default::default()).unwrap();
    spawner.spawn(usart2_task(usart2).unwrap());
}
