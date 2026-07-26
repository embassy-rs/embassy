#![no_std]
#![no_main]

use embassy_ambiq::gpio::Output;
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;

// SparkFun RedBoard Artemis STAT LED is on GPIO5 (D5).
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_ambiq::init();
    let mut led = Output::new(p.P5);

    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}
