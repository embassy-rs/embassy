// required-features: easydma
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_nrf::spim::Spim;
use embassy_nrf::{peripherals, spim};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M1;
    let mut spim = Spim::new(
        peri!(p, SPIM0).reborrow(),
        irqs!(SPIM0),
        peri!(p, PIN_X).reborrow(),
        peri!(p, PIN_A).reborrow(), // MISO
        peri!(p, PIN_B).reborrow(), // MOSI
        config.clone(),
    );
    let data = [
        0x42, 0x43, 0x44, 0x45, 0x66, 0x12, 0x23, 0x34, 0x45, 0x19, 0x91, 0xaa, 0xff, 0xa5, 0x5a, 0x77,
    ];
    let mut buf = [0u8; 16];

    buf.fill(0);
    spim.blocking_transfer(&mut buf, &data).unwrap();
    assert_eq!(data, buf);

    buf.fill(0);
    spim.transfer(&mut buf, &data).await.unwrap();
    assert_eq!(data, buf);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
