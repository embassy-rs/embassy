#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::rng::Rng;
use embassy_time::Timer;
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());
    let mut rng = Rng::new(p.RNG);

    loop {
        let value = rng.read_u32().await;
        info!("random value: {=u32}", value);
        Timer::after_millis(500).await;
    }
}
