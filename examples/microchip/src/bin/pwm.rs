#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use embassy_microchip::gpio::{Input, Pull};
use embassy_microchip::pwm::{Pwm, SetDutyCycle};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_microchip::init(Default::default());

    info!("Hello, world!");

    let mut pwm = Pwm::new(p.PWM0, p.GPIO53, Default::default());
    let mut btn = Input::new(p.GPIO141, Pull::None);

    let max_duty = pwm.max_duty_cycle();
    let step = max_duty / 10;

    loop {
        for duty in (1..max_duty).step_by(step.into()) {
            pwm.set_duty_cycle(duty).unwrap();

            info!("Duty cycle sent to {}. Press button for next step...", duty);
            btn.wait_for_low().await;
            btn.wait_for_high().await;
        }
    }
}
