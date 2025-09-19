// required-features: two-uarts
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::uarte::{UarteRx, UarteTx};
use embassy_nrf::{peripherals, uarte};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD1M;

    let mut tx = UarteTx::new(
        peri!(p, UART0).reborrow(),
        irqs!(UART0),
        peri!(p, PIN_A).reborrow(),
        config.clone(),
    );
    let mut rx = UarteRx::new(
        peri!(p, UART1).reborrow(),
        irqs!(UART1),
        peri!(p, PIN_B).reborrow(),
        config.clone(),
    );

    let data = [
        0x42, 0x43, 0x44, 0x45, 0x66, 0x12, 0x23, 0x34, 0x45, 0x19, 0x91, 0xaa, 0xff, 0xa5, 0x5a, 0x77,
    ];

    let tx_fut = async {
        tx.write(&data).await.unwrap();
    };
    let rx_fut = async {
        let mut buf = [0u8; 16];
        rx.read(&mut buf).await.unwrap();
        assert_eq!(data, buf);
    };
    join(rx_fut, tx_fut).await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}
