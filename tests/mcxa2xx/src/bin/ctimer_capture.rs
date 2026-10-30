//! CTimer Capture

#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use core::sync::atomic::{AtomicU32, Ordering};

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::{Duration, Instant, Timer};
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::ctimer::CTimer;
use hal::ctimer::capture::{self, Capture, Edge, InterruptHandler};
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use hal::peripherals::CTIMER2;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    CTIMER2 => InterruptHandler<CTIMER2>;
});

static PULSE_INTERVAL_US: AtomicU32 = AtomicU32::new(0);

fn within(x: f32, target: f32, epsilon: f32) -> bool {
    (x - target).abs() <= epsilon
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    let pin = Output::new(p.P1_8, Level::Low, DriveStrength::Normal, SlewRate::Fast);

    let ctimer = CTimer::new(p.CTIMER2, Default::default()).unwrap();
    let mut config = capture::Config::default();
    config.edge = Edge::RisingEdge;
    let mut capture = Capture::new_with_input_pin(ctimer, p.CTIMER2_CH0, p.P2_4, Irqs, config).unwrap();

    spawner.spawn(pin_task(pin).unwrap());

    let one = capture.capture().await.unwrap();
    let two = capture.capture().await.unwrap();
    let diff = two - one;
    let period = diff.to_period(capture.frequency());
    let pulse_interval_us = PULSE_INTERVAL_US.load(Ordering::Acquire);
    assert!(pulse_interval_us >= 1_000_000);
    let expected_period = pulse_interval_us as f32 / 1_000_000.0;
    // Test capture accuracy, not the wakeup precision of the timer driving the
    // GPIO. Keep the 1ms tolerance against the independently measured interval.
    assert!(within(period, expected_period, 1e-3));

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}

#[embassy_executor::task]
async fn pin_task(mut pin: Output<'static>) {
    Timer::after_secs(1).await;

    let first_pulse = Instant::now();
    pin.set_high();
    // Hold the level long enough for capture input sampling before generating
    // the second rising edge. Start low so both edges are explicit GPIO writes.
    Timer::after_millis(1).await;
    pin.set_low();
    Timer::at(first_pulse + Duration::from_secs(1)).await;
    let pulse_interval = first_pulse.elapsed().as_micros();
    PULSE_INTERVAL_US.store(pulse_interval as u32, Ordering::Release);
    pin.set_high();

    // Keep ownership of the output until the capture task finishes.
    Timer::after_secs(1).await;
}
