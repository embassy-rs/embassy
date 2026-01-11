#![no_std]
#![no_main]

#[cfg(feature = "sim")]
compile_error!("Blinky example not available in simulation.");

use embassy_neorv32::gpio::Gpio;
use embassy_neorv32::uart::UartTx;
use embassy_neorv32_examples::*;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_neorv32::init();

    let mut uart = UartTx::new_blocking(p.UART0, UART_BAUD, UART_IS_SIM, false).expect("UART must be supported");

    let gpio = Gpio::new_blocking(p.GPIO).expect("GPIO must be supported");
    let mut output = gpio.new_output(p.PORT0);

    uart.blocking_write(b"Starting blinky example...\n");
    loop {
        output.toggle();
        Timer::after_millis(100).await;
    }
}
