#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::dac::Dac;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);
    let dac = defmt::unwrap!(Dac::new(p.DAC0, p.P2_2, embassy_mcxa::dac::Config::default()));

    // Make a saw wave with a period of ~4 secs
    loop {
        dac.write((embassy_time::Instant::now().as_millis() % 4096) as u16);
    }
}
