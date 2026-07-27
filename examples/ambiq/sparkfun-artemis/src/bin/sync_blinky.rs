#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_ambiq::gpio::Output;
use embassy_time::{Duration, block_for};
use panic_halt as _;

#[entry]
fn main() -> ! {
    let p = embassy_ambiq::init();

    // Blue STAT LED is on PAD5 on the SparkFun RedBoard Artemis ->
    let mut led = Output::new(p.P5);

    loop {
        led.set_high();
        block_for(Duration::from_millis(500));

        led.set_low();
        block_for(Duration::from_millis(500));
    }
}
