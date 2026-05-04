//! CTimer Capture

#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::ctimer::CTimer;
use hal::ctimer::capture::{self, Capture, Edge, InterruptHandler};
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use hal::peripherals::CTIMER2;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    CTIMER2 => InterruptHandler<CTIMER2>;
});

fn within(x: f32, target: f32, epsilon: f32) -> bool {
    (x - target).abs() <= epsilon
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    let pin = Output::new(p.P1_8, Level::High, DriveStrength::Normal, SlewRate::Fast);

    let ctimer = CTimer::new(p.CTIMER2, Default::default()).unwrap();
    let mut config = capture::Config::default();
    config.edge = Edge::RisingEdge;
    let mut capture = Capture::new_with_input_pin(ctimer, p.CTIMER2_CH0, p.P2_4, Irqs, config).unwrap();

    spawner.spawn(pin_task(pin).unwrap());

    let one = capture.capture().await.unwrap();
    let two = capture.capture().await.unwrap();
    let diff = two - one;
    let freq = diff.to_period(capture.frequency());
    assert!(within(freq, 1.0, 1e-3));

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}

#[embassy_executor::task]
async fn pin_task(mut pin: Output<'static>) {
    Timer::after_secs(1).await;

    pin.set_high();
    pin.set_low();
    Timer::after_secs(1).await;
    pin.set_high();
    pin.set_low();

    Timer::after_secs(1).await;
}
