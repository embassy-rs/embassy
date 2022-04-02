#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy::executor::Spawner;
use embassy_rp::{uart, Peripherals};

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let config = uart::Config::default();
    let mut uart = uart::Uart::new(p.UART0, p.PIN_0, p.PIN_1, p.PIN_2, p.PIN_3, config);
    uart.send("Hello World!\r\n".as_bytes());

    loop {
        uart.send("hello there!\r\n".as_bytes());
        cortex_m::asm::delay(1_000_000);
    }
}
