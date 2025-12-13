#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clkout::{ClockOut, ClockOutSel, Config, Div4};
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::gpio::{DriveStrength, Level, Output, SlewRate};
use embassy_time::Timer;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

/// Demonstrate CLKOUT, using Pin P4.2
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());
    let mut pin = p.P4_2;
    let mut clkout = p.CLKOUT;

    loop {
        defmt::info!("Set Low...");
        let mut output = Output::new(pin.reborrow(), Level::Low, DriveStrength::Normal, SlewRate::Slow);
        Timer::after_millis(500).await;

        defmt::info!("Set High...");
        output.set_high();
        Timer::after_millis(400).await;

        defmt::info!("Set Low...");
        output.set_low();
        Timer::after_millis(500).await;

        defmt::info!("16k...");
        // Run Clock Out with the 16K clock
        let _clock_out = ClockOut::new(
            clkout.reborrow(),
            pin.reborrow(),
            Config {
                sel: ClockOutSel::Clk16K,
                div: Div4::no_div(),
                level: PoweredClock::NormalEnabledDeepSleepDisabled,
            },
        )
        .unwrap();

        Timer::after_millis(3000).await;

        defmt::info!("Set Low...");
        drop(_clock_out);

        let _output = Output::new(pin.reborrow(), Level::Low, DriveStrength::Normal, SlewRate::Slow);
        Timer::after_millis(500).await;

        // Run Clock Out with the 12M clock, divided by 3
        defmt::info!("4M...");
        let _clock_out = ClockOut::new(
            clkout.reborrow(),
            pin.reborrow(),
            Config {
                sel: ClockOutSel::Fro12M,
                div: const { Div4::from_divisor(3).unwrap() },
                level: PoweredClock::NormalEnabledDeepSleepDisabled,
            },
        )
        .unwrap();

        // Let it run for 3 seconds...
        Timer::after_millis(3000).await;
    }
}
