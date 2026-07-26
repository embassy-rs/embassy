#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::config::{Config, HfclkSource, LfclkSource, LfxoCapacitance};
use embassy_nrf::pac;
use {defmt_rtt as _, panic_probe as _};

fn print_xosc32mcaps() {
    let value = pac::OSCILLATORS.xosc32mcaps().read();
    info!("XOSC32MCAPS.ENABLE = {}", value.enable());
    info!("XOSC32MCAPS.CAPVALUE = {}", value.capvalue());
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Before init:");
    print_xosc32mcaps();

    let mut config = Config::default();
    config.hfclk_source = HfclkSource::Internal;
    config.lfclk_source = LfclkSource::ExternalXtal;
    config.internal_capacitors.hfxo = None; // keep the value from the FICR
    config.internal_capacitors.lfxo = Some(LfxoCapacitance::_7pF);
    let _p = embassy_nrf::init(config);

    info!("After init:");
    print_xosc32mcaps();
}
