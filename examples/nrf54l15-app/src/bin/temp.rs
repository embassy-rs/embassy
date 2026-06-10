#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::temp::Temp;
use embassy_nrf::{bind_interrupts, temp};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TEMP => temp::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut temp = Temp::new(p.TEMP, Irqs);

    loop {
        let value = temp.read().await;
        info!("temperature: {}℃", value.to_num::<u16>());
        Timer::after_secs(1).await;
    }
}
