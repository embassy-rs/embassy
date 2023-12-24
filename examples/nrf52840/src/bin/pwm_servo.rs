#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::pwm::{Prescaler, SimplePwm};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut pwm = SimplePwm::new_1ch(p.PWM0, p.P0_05);
    // sg90 microervo requires 50hz or 20ms period
    // set_period can only set down to 125khz so we cant use it directly
    // Div128 is 125khz or 0.000008s or 0.008ms, 20/0.008 is 2500 is top
    pwm.set_prescaler(Prescaler::Div128);
    pwm.set_max_duty(2500);
    info!("pwm initialized!");

    Timer::after_millis(5000).await;

    // 1ms 0deg (1/.008=125), 1.5ms 90deg (1.5/.008=187.5), 2ms 180deg (2/.008=250),
    loop {
        info!("45 deg");
        // poor mans inverting, subtract our value from max_duty
        pwm.set_duty(0, 2500 - 156);
        Timer::after_millis(5000).await;

        info!("90 deg");
        pwm.set_duty(0, 2500 - 187);
        Timer::after_millis(5000).await;

        info!("135 deg");
        pwm.set_duty(0, 2500 - 218);
        Timer::after_millis(5000).await;

        info!("180 deg");
        pwm.set_duty(0, 2500 - 250);
        Timer::after_millis(5000).await;

        info!("0 deg");
        pwm.set_duty(0, 2500 - 125);
        Timer::after_millis(5000).await;
    }
}
