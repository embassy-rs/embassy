#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nxp::gpio::{Input, Level, Output, Pull};
// Must include `embassy_imxrt1064_evk_examples` to ensure the FCB gets linked.
use {defmt_rtt as _, embassy_imxrt1064_evk_examples as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nxp::init(Default::default());
    info!("Hello world!");

    // User LED (D18)
    let led = p.GPIO_AD_B0_09;
    // User button (SW8)
    let button = p.WAKEUP;
    let button = Input::new(button, Pull::Up100K);
    let mut led = Output::new(led, Level::Low);
    led.set_high();

    loop {
        if button.is_high() {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}
