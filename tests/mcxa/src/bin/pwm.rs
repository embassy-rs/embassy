#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::clocks::periph_helpers::CTimerClockSel;
use hal::config::Config;
use hal::ctimer::CTimer;
use hal::ctimer::capture::{self, Capture, Edge, InterruptHandler};
use hal::ctimer::pwm::{SetDutyCycle, SinglePwm};
use hal::peripherals::CTIMER2;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    CTIMER2 => InterruptHandler<CTIMER2>;
});

fn within(x: f32, target: f32, epsilon: f32) -> bool {
    (x - target).abs() <= epsilon
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let mut p = hal::init(config);

    let mut config: hal::ctimer::Config = Default::default();
    config.source = CTimerClockSel::Clk1M;
    let pin_ctimer = CTimer::new(p.CTIMER1.reborrow(), config).unwrap();
    let mut config: hal::ctimer::pwm::Config = Default::default();
    config.freq = 20_000;
    let mut pin_pwm = SinglePwm::new(pin_ctimer, p.CTIMER1_CH0, p.CTIMER1_CH2, p.P2_4, config).unwrap();

    pin_pwm.set_duty_cycle_percent(50).unwrap();

    let ctimer = CTimer::new(p.CTIMER2, Default::default()).unwrap();
    let mut config = capture::Config::default();
    config.edge = Edge::RisingEdge;
    let mut capture = Capture::new_with_input_pin(ctimer, p.CTIMER2_CH0, p.P1_8, Irqs, config).unwrap();

    let one = capture.capture().await.unwrap();
    let two = capture.capture().await.unwrap();
    let diff = two - one;
    let freq = diff.to_frequency(capture.frequency());
    assert!(within(freq, 20_000.0, 0.1));

    pin_pwm.set_duty_cycle_percent(75).unwrap();

    let one = capture.capture().await.unwrap();
    let two = capture.capture().await.unwrap();
    let diff = two - one;
    let freq = diff.to_frequency(capture.frequency());
    assert!(within(freq, 20_000.0, 0.1));

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
