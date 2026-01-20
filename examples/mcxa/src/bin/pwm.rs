#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::ctimer::CTimer;
use hal::ctimer::pwm::{Pwm, SetDutyCycle};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let mut p = hal::init(config);

    defmt::info!("Pwm example");

    let led_ctimer = CTimer::new(p.CTIMER2.reborrow(), Default::default()).unwrap();
    let mut red_pwm = Pwm::new(
        led_ctimer.clone(),
        p.CTIMER2_CH3,
        p.CTIMER2_CH0,
        p.P3_18,
        Default::default(),
    )
    .unwrap();
    let mut green_pwm = Pwm::new(led_ctimer, p.CTIMER2_CH2, p.CTIMER2_CH1, p.P3_19, Default::default()).unwrap();

    let pin_ctimer = CTimer::new(p.CTIMER1.reborrow(), Default::default()).unwrap();
    let mut pin_pwm = Pwm::new(pin_ctimer, p.CTIMER1_CH0, p.CTIMER1_CH2, p.P3_12, Default::default()).unwrap();

    let mut duty: u8 = 0;
    let mut delta: i8 = 1;

    loop {
        // Fade LED in and out
        red_pwm.set_duty_cycle_percent(duty).unwrap();
        green_pwm.set_duty_cycle_percent(100 - duty).unwrap();
        pin_pwm.set_duty_cycle_percent(duty).unwrap();
        duty = ((duty as i8) + delta) as u8;

        if duty == 100 || duty == 0 {
            delta *= -1;
        }

        Timer::after_millis(10).await;
    }
}
