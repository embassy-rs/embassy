#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxn as hal;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Exercises clocks::init() (verifies the reset-default main clock) and
    // enables the PORT0-4/GPIO0-4 clock gates.
    let _p = hal::init(hal::config::Config::default());

    defmt::info!("Hello, world!");

    hal::clocks::with_clocks(|c| {
        defmt::info!("main_clk = {} Hz, system_clk = {} Hz", c.main_clk_hz, c.system_clk_hz);
    });

    loop {
        defmt::info!("tick");
        cortex_m::asm::delay(48_000_000);
    }
}
