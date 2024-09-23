#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use defmt::{assert, info};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_time::{Duration, Instant, Timer};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut input = Input::new(peri!(p, PIN_A), Pull::Up);
    let mut output = Output::new(peri!(p, PIN_B), Level::Low, OutputDrive::Standard);

    let fut1 = async {
        Timer::after_millis(100).await;
        output.set_high();
    };
    let fut2 = async {
        let start = Instant::now();
        input.wait_for_high().await;
        let dur = Instant::now() - start;
        info!("took {} ms", dur.as_millis());
        assert!((Duration::from_millis(90)..Duration::from_millis(110)).contains(&dur));
    };

    join(fut1, fut2).await;

    let fut1 = async {
        Timer::after_millis(100).await;
        output.set_low();
    };
    let fut2 = async {
        let start = Instant::now();
        input.wait_for_low().await;
        let dur = Instant::now() - start;
        info!("took {} ms", dur.as_millis());
        assert!((Duration::from_millis(90)..Duration::from_millis(110)).contains(&dur));
    };

    join(fut1, fut2).await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}
