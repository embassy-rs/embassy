#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::{bind_interrupts, peripherals, spim};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SERIAL20 => spim::InterruptHandler<peripherals::TWISPI20>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Running!");

    let mut config = spim::Config::default();
    config.prescaler = 16;

    let mut spim = spim::Spim::new(p.TWISPI20, Irqs, p.P2_01, p.P2_02, p.P2_04, config);

    let mut ncs = Output::new(p.P2_05, Level::High, OutputDrive::Standard);

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
