#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::temp::Temp;
use embassy_nrf::{bind_interrupts, temp};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    TEMP => temp::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut temp = Temp::new(p.TEMP, Irqs);

    loop {
        let value = temp.read_celsius_x4().await;
        info!("temperature: {}℃", value as f32 / 4.0);
        Timer::after_secs(1).await;
    }
}
