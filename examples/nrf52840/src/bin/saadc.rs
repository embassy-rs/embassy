#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::saadc::{ChannelConfig, Config, Saadc};
use embassy_nrf::{bind_interrupts, saadc};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let config = Config::default();
    let channel_config = ChannelConfig::single_ended(&mut p.P0_02);
    let mut saadc = Saadc::new(p.SAADC, Irqs, config, [channel_config]);

    loop {
        let mut buf = [0; 1];
        saadc.sample(&mut buf).await;
        info!("sample: {=i16}", &buf[0]);
        Timer::after(Duration::from_millis(100)).await;
    }
}
