#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::pdm::{self, Config, Pdm};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PDM => pdm::InterruptHandler<peripherals::PDM>;
});

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let config = Config::default();
    let mut pdm = Pdm::new(p.PDM, Irqs, p.P0_01, p.P0_00, config);

    loop {
        pdm.start().await;

        // wait some time till the microphon settled
        Timer::after(Duration::from_millis(1000)).await;

        const SAMPLES: usize = 2048;
        let mut buf = [0i16; SAMPLES];
        pdm.sample(&mut buf).await.unwrap();

        info!("samples: {:?}", &buf);

        pdm.stop().await;
        Timer::after(Duration::from_millis(100)).await;
    }
}
