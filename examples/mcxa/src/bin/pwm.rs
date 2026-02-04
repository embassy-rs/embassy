#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::ctimer::CTimer;
use hal::ctimer::pwm::{SetDutyCycle, SinglePwm, TriplePwm};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let mut p = hal::init(config);

    defmt::info!("Pwm example");

    let led_ctimer = CTimer::new(p.CTIMER2.reborrow(), Default::default()).unwrap();
    let mut pwm = TriplePwm::new(
        led_ctimer,
        p.CTIMER2_CH0,
        p.CTIMER2_CH1,
        p.CTIMER2_CH3,
        p.CTIMER2_CH2,
        p.P3_18,
        p.P3_19,
        p.P3_21,
        Default::default(),
    )
    .unwrap();

    let pin_ctimer = CTimer::new(p.CTIMER1.reborrow(), Default::default()).unwrap();
    let mut pin_pwm = SinglePwm::new(pin_ctimer, p.CTIMER1_CH2, p.CTIMER1_CH0, p.P3_12, Default::default()).unwrap();

    defmt::info!("Before split");
    for _ in 0..10 {
        for duty in (0u8..=100).chain((0..=100).rev()) {
            pin_pwm.set_duty_cycle_percent(duty).unwrap();
            pwm.pwm0.set_duty_cycle_percent(duty).unwrap();
            pwm.pwm1.set_duty_cycle_percent(100 - duty).unwrap();
            pwm.pwm2.set_duty_cycle_percent(duty).unwrap();
            Timer::after_millis(10).await;
        }
    }

    let (mut red, mut green, mut blue) = pwm.split();

    defmt::info!("After split");

    for _ in 0..10 {
        for duty in (0u8..=100).chain((0..=100).rev()) {
            pin_pwm.set_duty_cycle_percent(duty).unwrap();
            red.set_duty_cycle_percent(duty).unwrap();
            green.set_duty_cycle_percent(100 - duty).unwrap();
            blue.set_duty_cycle_percent(duty).unwrap();
            Timer::after_millis(10).await;
        }
    }
}
