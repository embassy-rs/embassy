#![no_std]
#![no_main]

use defmt::{info, unwrap};

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::{
    gpio::{Level, Output, OutputDrive},
    vpr,
};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    // Turn on LED0
    let led0 = Output::new(p.P2_09, Level::Low, OutputDrive::Standard);

    spawner.spawn(unwrap!(blinker(led0)));

    // Start the riscv core
    const RISCV_ENTRY_ADDR: u32 = 0x00010000;

    let mut vpr = unwrap!(vpr::Vpr::new(p.VPR, RISCV_ENTRY_ADDR));

    info!("Start VPR core from address {:#010x}", RISCV_ENTRY_ADDR);

    vpr.start();

    // Turn on LED2
    let led2 = Output::new(p.P2_07, Level::Low, OutputDrive::Standard);
    spawner.spawn(unwrap!(blinker(led2)));
}

#[embassy_executor::task(pool_size = 2)]
async fn blinker(mut led: Output<'static>) -> ! {
    loop {
        led.set_high();
        Timer::after_millis(300).await;
        led.set_low();
        Timer::after_millis(300).await;
    }
}
