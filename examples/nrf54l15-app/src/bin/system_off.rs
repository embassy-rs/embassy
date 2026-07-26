#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::pac::gpio::vals::Sense;
use embassy_nrf::{pac, power};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    // LED 0 on the nRF54L15-DK. Lit while awake, so wakeup is visible.
    let mut led = Output::new(p.P2_09, Level::High, OutputDrive::Standard);

    info!("Entering System OFF in 5 seconds. Press Button 0 to wake up.");
    Timer::after_secs(5).await;
    led.set_low();

    // Button 0 on the nRF54L15-DK is on P1.13, active low.
    let _button = Input::new(p.P1_13, Pull::Up);
    // Enable SENSE on the button pin so it wakes the chip from System OFF.
    pac::P1.pin_cnf(13).modify(|w| w.set_sense(Sense::Low));

    info!("Entering System OFF now.");
    power::set_system_off();

    // The chip is now in System OFF. Wakeup causes a reset.
    loop {}
}
