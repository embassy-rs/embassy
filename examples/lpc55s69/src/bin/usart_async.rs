#![no_std]
#![no_main]

use core::str::from_utf8_mut;

use defmt::*;
use embassy_executor::Spawner;
use embassy_nxp::bind_interrupts;
use embassy_nxp::gpio::{Level, Output};
use embassy_nxp::peripherals::USART2;
use embassy_nxp::usart::{Config, InterruptHandler, Usart};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

bind_interrupts!(struct Irqs {
        FLEXCOMM2 => InterruptHandler<USART2>;
    }
);

#[embassy_executor::task]
async fn blinky_task(mut led: Output<'static>) {
    loop {
        info!("[TASK] led off!");
        led.set_high();
        Timer::after_millis(500).await;

        info!("[TASK] led on!");
        led.set_low();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());
    let mut usart = Usart::new(
        p.USART2,
        p.PIO0_27,
        p.PIO1_24,
        Irqs,
        p.DMA_CH11,
        p.DMA_CH10,
        Config::default(),
    );
    let led = Output::new(p.PIO1_6, Level::Low);
    spawner.spawn(blinky_task(led).unwrap());
    info!("[MAIN] Entering main loop");
    loop {
        let tx_buf = b"Hello, Ferris!";
        let mut rx_buf = [0u8; 14];
        info!("[MAIN] Write a message");
        usart.write(tx_buf).await.unwrap();
        Timer::after_millis(500).await;

        info!("[MAIN] Read a message");
        match usart.read(&mut rx_buf).await {
            Ok(_) => match from_utf8_mut(&mut rx_buf) {
                Ok(str) => {
                    info!("[MAIN] The message is: {}", str);
                }
                Err(_) => {
                    error!("[MAIN] Error in converting to UTF8");
                }
            },
            Err(e) => warn!("[MAIN] Error: {}", e),
        }

        Timer::after_millis(500).await;
    }
}
