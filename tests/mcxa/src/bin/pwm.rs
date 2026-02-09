#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_mcxa::clocks::periph_helpers::CTimerClockSel;
use embassy_time::Instant;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::ctimer::CTimer;
use hal::ctimer::pwm::{SetDutyCycle, SinglePwm};
use hal::gpio::Input;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

fn within(x: u64, target: u64, deviation: u64) -> bool {
    x.abs_diff(target) <= deviation
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let mut p = hal::init(config);

    let mut config: hal::ctimer::Config = Default::default();
    config.source = CTimerClockSel::Clk16K;
    let pin_ctimer = CTimer::new(p.CTIMER1.reborrow(), config).unwrap();
    let mut config: hal::ctimer::pwm::Config = Default::default();
    config.freq = u16::MAX / 16 / 1024 * 1000;
    let mut pin_pwm = SinglePwm::new(pin_ctimer, p.CTIMER1_CH0, p.CTIMER1_CH2, p.P2_4, config).unwrap();

    pin_pwm.set_duty_cycle_percent(50).unwrap();

    let mut input = Input::new(p.P1_8, embassy_mcxa::gpio::Pull::Up);
    input.wait_for_high().await;
    input.wait_for_low().await;
    let start = Instant::now();
    input.wait_for_high().await;
    let duration_ms = start.elapsed().as_millis();

    assert!(within(duration_ms, 10, 1));

    pin_pwm.set_duty_cycle_percent(75).unwrap();
    input.wait_for_high().await;
    input.wait_for_low().await;
    let start = Instant::now();
    input.wait_for_high().await;
    let duration_ms = start.elapsed().as_millis();

    assert!(within(duration_ms, 15, 1));

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
