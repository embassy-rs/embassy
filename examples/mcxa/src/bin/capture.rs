#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::ctimer::CTimer;
use hal::ctimer::capture::{self, Capture, Edge, InterruptHandler};
use hal::ctimer::pwm::{SetDutyCycle, SinglePwm};
use hal::peripherals::CTIMER2;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    CTIMER2 => InterruptHandler<CTIMER2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("Pwm example");

    let pwm_ctimer = CTimer::new(p.CTIMER1, Default::default()).unwrap();
    let mut pwm = SinglePwm::new(pwm_ctimer, p.CTIMER1_CH2, p.CTIMER1_CH0, p.P3_12, Default::default()).unwrap();

    let cap_ctimer = CTimer::new(p.CTIMER2, Default::default()).unwrap();
    let mut config = capture::Config::default();
    config.edge = Edge::RisingEdge;
    let mut capture = Capture::new_with_input_pin(cap_ctimer, p.CTIMER2_CH0, p.P3_14, Irqs, config).unwrap();

    pwm.set_duty_cycle_percent(50).unwrap();

    loop {
        let one = capture.capture().await.unwrap();
        let two = capture.capture().await.unwrap();
        let diff = two - one;
        defmt::info!(
            "{} s {} Hz",
            diff.to_period(capture.frequency()),
            diff.to_frequency(capture.frequency())
        );
    }
}
