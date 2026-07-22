#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::gpio::{Level, Output};
use embassy_mspm0::sysctl::{ClkOut, ClkOutDiv, ClkOutSource};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_mspm0::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PA0, Level::High);

    // SYSRST must be initiated for the output to actually change. probe-rs currently does not do this.
    // To see changes you will need to flash and then power cycle or press reset button on your board.
    let division = Some(ClkOutDiv::Div2);
    let _clk_out = ClkOut::new(p.CLK_OUT, p.PA22, ClkOutSource::Sysosc(division));

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(500).await;

        info!("low");
        led.set_low();
        Timer::after_millis(500).await;
    }
}
