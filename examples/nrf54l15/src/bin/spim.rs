#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::{bind_interrupts, peripherals, spim};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SERIAL00 => spim::InterruptHandler<peripherals::SERIAL00>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("running!");

    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M16;

    let mut spim = spim::Spim::new(p.SERIAL00, Irqs, p.P0_01, p.P0_00, p.P0_02, config);

    // Example on how to talk to an ADXL343

    let mut ncs = Output::new(p.P0_03, Level::High, OutputDrive::Standard);

    // Example on how to talk to an ENC28J60 chip

    // softreset
    cortex_m::asm::delay(10);
    ncs.set_low();
    cortex_m::asm::delay(5);
    let tx = [0xFF];
    unwrap!(spim.transfer(&mut [], &tx).await);
    cortex_m::asm::delay(10);
    ncs.set_high();

    cortex_m::asm::delay(100000);

    let mut rx = [0; 2];

    // read ESTAT
    cortex_m::asm::delay(5000);
    ncs.set_low();
    cortex_m::asm::delay(5000);
    let tx = [0b000_11101, 0];
    unwrap!(spim.transfer(&mut rx, &tx).await);
    cortex_m::asm::delay(5000);
    ncs.set_high();
    info!("estat: {=[?]}", rx);

    // Switch to bank 3
    cortex_m::asm::delay(10);
    ncs.set_low();
    cortex_m::asm::delay(5);
    let tx = [0b100_11111, 0b11];
    unwrap!(spim.transfer(&mut rx, &tx).await);
    cortex_m::asm::delay(10);
    ncs.set_high();

    // read EREVID
    cortex_m::asm::delay(10);
    ncs.set_low();
    cortex_m::asm::delay(5);
    let tx = [0b000_10010, 0];
    unwrap!(spim.transfer(&mut rx, &tx).await);
    cortex_m::asm::delay(10);
    ncs.set_high();

    info!("erevid: {=[?]}", rx);
}
