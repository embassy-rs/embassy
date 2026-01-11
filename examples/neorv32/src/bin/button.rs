#![no_std]
#![no_main]

#[cfg(feature = "sim")]
compile_error!("Button example not available in simulation.");

use embassy_neorv32::gpio::{self, Gpio};
use embassy_neorv32::uart::UartTx;
use embassy_neorv32::{bind_interrupts, peripherals};
use embassy_neorv32_examples::*;

bind_interrupts!(struct Irqs {
    GPIO => gpio::InterruptHandler<peripherals::GPIO>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_neorv32::init();

    let mut uart = UartTx::new_blocking(p.UART0, UART_BAUD, UART_IS_SIM, false).expect("UART must be supported");

    let gpio = Gpio::new_async(p.GPIO, Irqs).expect("GPIO must be supported");
    let mut input = gpio.new_input(p.PORT0);

    uart.blocking_write(b"Starting button example...\n");
    loop {
        input.wait_for_falling_edge().await;
        uart.blocking_write(b"Button press detected\n");
    }
}
