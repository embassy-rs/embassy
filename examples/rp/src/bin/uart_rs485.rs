//! This example shows how to use UART (Universal asynchronous receiver-transmitter) in the RP2040 chip.
//!
//! No specific hardware is specified in this example. Only output on pin 0 and pin 2, and input on pin 1 is tested.
//! The Raspberry Pi Debug Probe (https://www.raspberrypi.com/products/debug-probe/) could be used
//! with its UART port.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config};
use embedded_io_async::{Read, Write};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static TX_BUF: StaticCell<[u8; 32]> = StaticCell::new();
static RX_BUF: StaticCell<[u8; 32]> = StaticCell::new();

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let config = Config::default();
    let mut uart = BufferedUart::new(
        p.UART0,
        p.PIN_0,
        p.PIN_1,
        Irqs,
        TX_BUF.init([0; 32]),
        RX_BUF.init([0; 32]),
        config,
    );
    let mut data_enable = Output::new(p.PIN_2, Level::Low);

    loop {
        // When using a UART with an RS485 bus, often a transceiver is used which has a DE/nRE pin.
        // Setting this DE/nRE pin high to allow transmitting data, could cause the RX pin to go low depending on the chip and hardware used.
        // When done transmitting data and setting the DE/nRE pin low, the RX pin would go up again which will trigger a break condition in the UART receiver.
        // This can be prevented by temporarily disabling the receiver of the UART, roughly as follows:
        uart.disable_rx();
        data_enable.set_high();

        uart.write_all("hello there!\r\n".as_bytes()).await.unwrap();

        data_enable.set_low();
        uart.enable_rx();

        let mut buf = [0; 16];
        uart.read(&mut buf).await.unwrap();

        info!("Rx: {:?}", buf);

        cortex_m::asm::delay(1_000_000);
    }
}
