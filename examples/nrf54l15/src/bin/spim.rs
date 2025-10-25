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

    todo!()
}
