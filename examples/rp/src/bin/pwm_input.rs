//! This example shows how to use the PWM module to measure the frequency of an input signal.
//! 
//! Due to the u16 limitation of the `top` register value, the maximum frequency that can
//! be reliably measured is 65.535 KHz. To measure higher frequencies you will either need
//! to count the number of overflows (wraps) of the counter and calculate the frequency
//! based on that, or use a different method to measure the frequency, such as using the
//! `PwmCounter` type. See the `pwm_counter` example for more information.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::pwm::prelude::*;
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize peripherals
    let peripherals = embassy_rp::init(Default::default());

    let slice0 = Pwm::builder()
        .edge_sensitive(EdgeSensitivity::Rising)
        .with_input_defaults()
        .apply(peripherals.PWM_SLICE7, peripherals.PIN_15)
        .unwrap();

    slice0.enable();

    let sampling_ms: u16 = 1_000;
    let mut ticker = Ticker::every(Duration::from_millis(sampling_ms.into()));
    let mut counter = 0u64;

    loop {
        loop {
            counter += 1;
            if counter % 2 == 0 {
                info!("Input frequency: {} Hz", slice0.counter() / (sampling_ms / 1_000));
            }
            slice0.set_counter(0);
            ticker.next().await;
        }
    }

    // let p = embassy_rp::init(Default::default());

    // let cfg: Config = Default::default();
    // let pwm = Pwm::new_input(p.PWM_SLICE2, p.PIN_5, Pull::None, InputMode::RisingEdge, cfg);

    // let mut ticker = Ticker::every(Duration::from_secs(1));
    // loop {
    //     info!("Input frequency: {} Hz", pwm.counter());
    //     pwm.set_counter(0);
    //     ticker.next().await;
    // }
}
