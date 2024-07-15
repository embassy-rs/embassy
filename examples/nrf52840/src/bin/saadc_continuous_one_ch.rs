#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::saadc::{CallbackResult, ChannelConfig, Config, Saadc};
use embassy_nrf::{bind_interrupts, saadc};
use {defmt_rtt as _, panic_probe as _};

// Demonstrates continuous sampling with one channel and the internal timer

bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let config = Config::default();
    let channel_1_config = ChannelConfig::single_ended(&mut p.P0_02);
    let mut saadc = Saadc::new(p.SAADC, Irqs, config, [channel_1_config]);

    saadc.calibrate().await;

    let mut bufs = [[[0; 1]; 10]; 2];

    let mut c = 0;
    let mut a: i32 = 0;

    saadc
        .run_timer_sampler(&mut bufs, 2000, move |buf| {
            // NOTE: It is important that the time spent within this callback
            // does not exceed the time taken to acquire the 500 samples we
            // have in this example, which would be 10us + 2us per
            // sample * 500 = 6ms. You need to measure the time taken here
            // and set the sample buffer size accordingly. Exceeding this
            // time can lead to the peripheral re-writing the other buffer.
            for b in buf {
                a += b[0] as i32;
            }
            c += buf.len();
            if c > 1000 {
                a = a / c as i32;
                info!("channel 1: {=i32}", a);
                c = 0;
                a = 0;
            }
            CallbackResult::Continue
        })
        .await;
}
