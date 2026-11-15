#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Input, Level, Output, Pull, SlewRate};
use hal::{bind_interrupts, gpio, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    GPIO00 => gpio::InterruptHandler<peripherals::GPIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    defmt::info!("GPIO interrupt example");

    // SW2 ("WAKEUP") on the FRDM-MCXN947 pulls P0_23 low when pressed.
    let mut button = Input::new_async(p.P0_23, Irqs, Pull::Up);
    let mut red = Output::new(p.P0_10, Level::High, DriveStrength::Normal, SlewRate::Fast);

    let mut press_count = 0u32;

    loop {
        button.wait_for_falling_edge().await;

        press_count += 1;
        red.toggle();

        defmt::info!("Button pressed! Count: {}", press_count);
        Timer::after_millis(50).await;
    }
}
