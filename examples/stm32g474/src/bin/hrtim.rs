#![no_std]
#![no_main]

use defmt::*;

use embassy_executor::Spawner;
use embassy_stm32::hrtim::{AdvancedPwm, BridgeConverter, ComplementaryPwmPin, PwmPin};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    info!("Hello World!");

    let ch1 = PwmPin::new_cha(p.PA8);
    let ch1n = ComplementaryPwmPin::new_cha(p.PA9);
    let pwm = AdvancedPwm::new(
        p.HRTIM1,
        Some(ch1),
        Some(ch1n),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    info!("pwm constructed");

    let mut buck_converter = BridgeConverter::new(pwm.ch_a, Hertz::khz(5));

    let max_duty = buck_converter.get_max_compare_value();

    info!("max compare value: {}", max_duty);

    buck_converter.set_dead_time(max_duty / 20);
    buck_converter.set_primary_duty(max_duty / 2);
    buck_converter.set_secondary_duty(3 * max_duty / 4);

    buck_converter.start();

    Timer::after_millis(500).await;

    info!("end program");

    cortex_m::asm::bkpt();
}
