#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::dac::{DacChannel, ValueArray};
use embassy_stm32::dma;
use embassy_stm32::peripherals::GPDMA1_CH0;
use embassy_stm32::rcc::{LsConfig, mux};
use embassy_stm32::{Config, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL0 => dma::InterruptHandler<GPDMA1_CH0>;
});

const RAMP_WAVE: [u16; 41] = [
    0, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200, 1300, 1400, 1500, 1600, 1700,
    1800, 1900, 2000, 2100, 2200, 2300, 2400, 2500, 2600, 2700, 2800, 2900, 3000, 3100, 3200, 3300,
    3400, 3500, 3600, 3700, 3800, 3900, 4000,
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Device has started");
    let mut config = Config::default();

    config.rcc.ls = LsConfig::default_lsi(); // turns on internal LSI(needed for DAC sync)
    config.rcc.mux.dac1sel = mux::Dacsel::LSI; // changing the mux to point to our clock(LSI)
    let p = embassy_stm32::init(config);

    info!("Board connected!");

    let mut dac = DacChannel::new(p.DAC1, p.GPDMA1_CH0, Irqs, p.PA4);

    dac.(ValueArray::Bit12Right(&RAMP_WAVE), true).await;

    loop {
        Timer::after_millis(5000).await;
    }
}
