#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::clocks::config::Div8;
use hal::clocks::periph_helpers::CTimerClockSel;
use hal::config::Config;
use hal::ctimer::pwm::{DualPwm, SetDutyCycle, SinglePwm};
use hal::ctimer::{self, CTimer};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let mut p = hal::init(config);

    defmt::info!("Pwm example");

    let mut config = ctimer::Config::default();
    config.source = CTimerClockSel::Clk1M;
    let red_ctimer = CTimer::new(p.CTIMER4.reborrow(), config.clone()).unwrap();
    let mut red = SinglePwm::new(red_ctimer, p.CTIMER4_CH2, p.CTIMER4_CH0, p.P2_14, Default::default()).unwrap();

    let green_blue_ctimer = CTimer::new(p.CTIMER2.reborrow(), config).unwrap();
    let green_blue = DualPwm::new(
        green_blue_ctimer,
        p.CTIMER2_CH2,
        p.CTIMER2_CH3,
        p.CTIMER2_CH0,
        p.P2_22,
        p.P2_23,
        Default::default(),
    )
    .unwrap();

    let (mut green, mut blue) = green_blue.split();
    red.set_duty_cycle_fully_off().unwrap();
    green.set_duty_cycle_fully_off().unwrap();
    blue.set_duty_cycle_fully_off().unwrap();

    loop {
        defmt::info!("Red");
        for duty in (0u8..=100).chain((0..=100).rev()) {
            red.set_duty_cycle_percent(duty).unwrap();
            Timer::after_millis(10).await;
        }

        defmt::info!("Green");
        for duty in (0u8..=100).chain((0..=100).rev()) {
            green.set_duty_cycle_percent(duty).unwrap();
            Timer::after_millis(10).await;
        }

        defmt::info!("Blue");
        for duty in (0u8..=100).chain((0..=100).rev()) {
            blue.set_duty_cycle_percent(duty).unwrap();
            Timer::after_millis(10).await;
        }
    }
}
