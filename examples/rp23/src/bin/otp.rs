//! This example shows reading the OTP constants on the RP235x.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::otp;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _ = embassy_rp::init(Default::default());
    //
    // add some delay to give an attached debug probe time to parse the
    // defmt RTT header. Reading that header might touch flash memory, which
    // interferes with flash write operations.
    // https://github.com/knurling-rs/defmt/pull/683
    Timer::after_millis(10).await;

    let chip_id = unwrap!(otp::get_chipid());
    info!("Unique id:{:X}", chip_id);

    let private_rand = unwrap!(otp::get_private_random_number());
    info!("Private Rand:{:X}", private_rand);

    loop {
        Timer::after_secs(1).await;
    }
}
