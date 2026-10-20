#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use hal::lpuart::{Config, Lpuart};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());
    let mut uart = Lpuart::new_blocking(
        p.LPUART0,
        p.P0_3,
        p.P0_2,
        Config {
            baudrate_bps: 115_200,
            ..Default::default()
        },
    )
    .unwrap();

    uart.write_str_blocking("FRDM-MCXA156 embassy serial ready\r\n");

    loop {
        let byte = uart.read_byte_blocking();
        uart.blocking_write(&[byte]).unwrap();
    }
}
