//! UART read until example for RP2040

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::uart;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let config = uart::Config::default();
    let mut uart = uart::Uart::new_blocking(
        p.UART0, p.PIN_0, // TX
        p.PIN_1, // RX
        config,
    );
    let mut buf = [0u8; 100];
    loop {
        match uart.blocking_read_until(&mut buf, b'\r', 5_000_000) {
            Ok(n) => {
                defmt::info!("Received {} bytes", n);
                uart.blocking_write(&buf[..n]).unwrap();
            }
            Err(embassy_rp::uart::Error::Timeout(n)) => {
                defmt::warn!("Timeout after {} bytes", n);
                uart.blocking_write(&buf[..n]).unwrap();
            }
            Err(embassy_rp::uart::Error::BufferOverflow) => {
                defmt::warn!("Buffer overflow");
            }
            Err(e) => {
                defmt::error!("UART error: {:?}", e);
            }
        }
    }
}
