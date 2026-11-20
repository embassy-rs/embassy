//! Route a 1 kHz CTIMER2 PWM signal through AOI0 to CTIMER0 capture inputs.
//! No jumper wires are needed. P2_22 (green LED) is driven by the source PWM.
//! EVENT0 passes the source via C/PT2; EVENT1 inverts it via D/PT3.
//! EVENT2 is A & !B with identical sources; EVENT3 is A | !A.
//! The last two must never produce capture edges after initialization.

#![no_std]
#![no_main]

use core::future::{Future, poll_fn};
use core::pin::pin;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::{Duration, Timer, with_timeout};
use hal::aoi::{Aoi, Event, EventConfig, Factor, Function, InputSource, ProductTerm};
use hal::clocks::config::Div8;
use hal::clocks::periph_helpers::CTimerClockSel;
use hal::ctimer::capture::{self, Capture, Edge, InterruptHandler};
use hal::ctimer::pwm::SinglePwm;
use hal::ctimer::{self, CTimer};
use panic_probe as _;

hal::bind_interrupts!(struct Irqs {
    CTIMER0 => InterruptHandler<hal::peripherals::CTIMER0>;
});

async fn check_period(capture: &mut Capture<'_>) {
    let first = with_timeout(Duration::from_millis(100), capture.capture())
        .await
        .unwrap()
        .unwrap();
    let second = with_timeout(Duration::from_millis(100), capture.capture())
        .await
        .unwrap()
        .unwrap();
    let ticks = (second - first).0.unsigned_abs();
    let expected = capture.frequency() / 1_000;
    assert!(ticks.abs_diff(expected) <= expected / 50 + 1);
    defmt::info!("AOI period: {} ticks (expected {})", ticks, expected);
}

async fn check_channels(quiet: &mut Capture<'_>, active: &mut Capture<'_>) {
    let mut quiet = pin!(quiet.capture());
    let mut active = pin!(active.capture());
    // Poll the quiet channel first, like select(quiet, active). An active
    // channel must not depend on a sibling consuming its wake notification.
    with_timeout(
        Duration::from_millis(100),
        poll_fn(|cx| {
            assert!(quiet.as_mut().poll(cx).is_pending());
            active.as_mut().poll(cx)
        }),
    )
    .await
    .unwrap()
    .unwrap();
}

async fn check_phase(pass: &mut Capture<'_>, inverted: &mut Capture<'_>) {
    let first = with_timeout(Duration::from_millis(100), pass.capture())
        .await
        .unwrap()
        .unwrap();
    let second = with_timeout(Duration::from_millis(100), inverted.capture())
        .await
        .unwrap()
        .unwrap();
    let period = pass.frequency() / 1_000;
    let phase = (second - first).0.rem_euclid(period as i32) as u32;
    assert!(phase.abs_diff(period / 2) <= period / 50 + 1);
    defmt::info!("AOI inversion phase: {} ticks (expected {})", phase, period / 2);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = hal::config::Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);
    let p = hal::init(config);

    let mut clock = ctimer::Config::default();
    clock.source = CTimerClockSel::Clk1M;
    let source_timer = CTimer::new(p.CTIMER2, clock.clone()).unwrap();
    let mut pwm = hal::ctimer::pwm::Config::default();
    pwm.freq = 1_000;
    pwm.duty_cycle = 50;
    let _source = SinglePwm::new(source_timer, p.CTIMER2_CH2, p.CTIMER2_CH0, p.P2_22, pwm).unwrap();

    let a = ProductTerm {
        a: Factor::Direct,
        ..ProductTerm::TRUE
    };
    let not_a = ProductTerm {
        a: Factor::Inverted,
        ..ProductTerm::TRUE
    };
    let contradiction = ProductTerm {
        a: Factor::Direct,
        b: Factor::Inverted,
        ..ProductTerm::TRUE
    };
    let source = InputSource::Ctimer2Match(Event::Two);
    let unused = InputSource::Lptmr0;
    let pass_term = ProductTerm {
        c: Factor::Direct,
        ..ProductTerm::TRUE
    };
    let invert_term = ProductTerm {
        d: Factor::Inverted,
        ..ProductTerm::TRUE
    };
    let mut config = hal::aoi::Config::default();
    config.events = [
        Some(EventConfig::new(
            [unused, unused, source, unused],
            Function::new([ProductTerm::FALSE, ProductTerm::FALSE, pass_term, ProductTerm::FALSE]),
        )),
        Some(EventConfig::new(
            [unused, unused, unused, source],
            Function::new([ProductTerm::FALSE, ProductTerm::FALSE, ProductTerm::FALSE, invert_term]),
        )),
        Some(EventConfig::new(
            [source, source, unused, unused],
            Function::new([
                contradiction,
                ProductTerm::FALSE,
                ProductTerm::FALSE,
                ProductTerm::FALSE,
            ]),
        )),
        Some(EventConfig::new(
            [source, unused, unused, unused],
            Function::new([a, not_a, ProductTerm::FALSE, ProductTerm::FALSE]),
        )),
    ];
    let aoi = Aoi::new(p.AOI0, config).unwrap();

    let timer = CTimer::new(p.CTIMER0, clock).unwrap();
    let mut config = capture::Config::default();
    config.edge = Edge::RisingEdge;
    let mut pass = Capture::new_with_aoi(timer.clone(), p.CTIMER0_CH0, aoi.output(Event::Zero), Irqs, config).unwrap();
    let mut inverted =
        Capture::new_with_aoi(timer.clone(), p.CTIMER0_CH1, aoi.output(Event::One), Irqs, config).unwrap();
    let mut low = Capture::new_with_aoi(timer.clone(), p.CTIMER0_CH2, aoi.output(Event::Two), Irqs, config).unwrap();
    let mut high = Capture::new_with_aoi(timer, p.CTIMER0_CH3, aoi.output(Event::Three), Irqs, config).unwrap();

    loop {
        check_channels(&mut low, &mut pass).await;
        check_period(&mut pass).await;
        check_period(&mut inverted).await;
        check_phase(&mut pass, &mut inverted).await;
        assert!(with_timeout(Duration::from_millis(20), low.capture()).await.is_err());
        assert!(with_timeout(Duration::from_millis(20), high.capture()).await.is_err());
        defmt::info!("AOI verified: pass/invert at 1 kHz; A & !A and A | !A have no edges");
        Timer::after_secs(1).await;
    }
}
