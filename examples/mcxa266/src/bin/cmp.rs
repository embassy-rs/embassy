//! CMP0 continuous/filter lifecycle diagnostics on P1_3 (CMP0_IN1).
//!
//! The negative input is the internal DAC at VDD/2. Supply a known, in-range
//! analog voltage to P1_3 for meaningful threshold/edge measurements. Probe-only
//! execution checks lifecycle/priming/rearming, not analog accuracy; the board's
//! other connections to this pad must be considered before applying a stimulus.
//! No output pad, DMA, windowing or round-robin configuration is used.

#![no_std]
#![no_main]

use core::future::Future;
use core::num::NonZeroU8;
use core::pin::pin;
use core::task::{Context, Poll, Waker};

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::{Duration, Timer, with_timeout};
use hal::cmp::{Cmp, ConfigError, DacInput, Error, Filter, InterruptHandler, PinInput, SampleCount};
use hal::interrupt::typelevel::Interrupt;
use panic_probe as _;

hal::bind_interrupts!(struct Irqs {
    CMP0 => InterruptHandler<hal::peripherals::CMP0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = hal::init(Default::default());
    // A pending interrupt must not outlive the owner and access its gated clock.
    let diagnostic = Cmp::new_async(
        p.CMP0.reborrow(),
        PinInput::new(p.P1_3.reborrow()),
        DacInput::new(127),
        Irqs,
        Default::default(),
    )
    .unwrap();
    cortex_m::interrupt::free(|_| {
        hal::interrupt::typelevel::CMP0::pend();
        drop(diagnostic);
        assert!(!hal::interrupt::typelevel::CMP0::is_enabled());
        assert!(!hal::interrupt::typelevel::CMP0::is_pending());
    });
    defmt::info!("CMP pending-interrupt teardown verified; reconstructing owner");
    let mut cmp = Cmp::new_async(
        p.CMP0,
        PinInput::new(p.P1_3),
        DacInput::new(127),
        Irqs,
        Default::default(),
    )
    .unwrap();
    assert!(!cmp.is_enabled());
    assert_eq!(cmp.is_high(), Err(Error::Disabled));
    assert_eq!(cmp.wait_for_any_edge().await, Err(Error::Disabled));

    // CMP1's filter is clocked from VDD_CORE FRO16K, unlike CMP0's VSYS input.
    let mut aux_config = hal::cmp::Config::default();
    aux_config.filter = Filter::Consecutive {
        samples: SampleCount::Two,
        period: NonZeroU8::new(1).unwrap(),
    };
    let mut aux = Cmp::new_blocking(p.CMP1, PinInput::new(p.P1_5), DacInput::new(127), aux_config).unwrap();
    aux.enable();
    Timer::after_millis(5).await;
    defmt::info!(
        "CMP1 filtered COUT (not a calibrated voltage): {}",
        aux.is_high().unwrap()
    );
    aux.disable();
    drop(aux);

    let mut config = hal::cmp::Config::default();
    loop {
        cmp.enable();
        cmp.enable(); // Idempotent: must not restart priming.
        Timer::after_millis(200).await;
        assert!(cmp.is_enabled());
        defmt::info!("CMP0 COUT (supply/2 threshold): {}", cmp.is_high().unwrap());
        let level = cmp.is_high().unwrap();
        let mut cx = Context::from_waker(Waker::noop());
        let immediately_ready = if level {
            let mut wait = pin!(cmp.wait_for_high());
            matches!(wait.as_mut().poll(&mut cx), Poll::Ready(Ok(())))
        } else {
            let mut wait = pin!(cmp.wait_for_low());
            matches!(wait.as_mut().poll(&mut cx), Poll::Ready(Ok(())))
        };
        // Diagnostic only: without a stable stimulus the input can change
        // between sampling the level and polling its matching wait.
        defmt::info!("CMP matching-level first poll ready: {}", immediately_ready);
        assert_eq!(cmp.set_config(&config), Err(ConfigError::Enabled));
        assert_eq!(cmp.set_dac_code(128), Err(ConfigError::Enabled));
        assert!(cmp.is_enabled());
        assert!(cmp.is_high().is_ok()); // Rejected writes did not restart priming.

        // Either edges or timeout are valid without a known external waveform.
        // Repeating after timeout exercises cancellation without disabling CMP.
        for _ in 0..2 {
            match with_timeout(Duration::from_millis(20), cmp.wait_for_any_edge()).await {
                Ok(result) => {
                    result.unwrap();
                    defmt::info!("CMP0 edge observed");
                }
                Err(_) => defmt::info!("CMP0 edge timeout; comparison remains enabled"),
            }
            assert!(cmp.is_enabled());
        }

        cmp.disable();
        cmp.disable();
        assert_eq!(cmp.is_high(), Err(Error::Disabled));
        cmp.set_dac_code(127).unwrap();
        config.filter = match config.filter {
            Filter::Disabled => Filter::Consecutive {
                samples: SampleCount::Seven,
                period: NonZeroU8::new(255).unwrap(),
            },
            _ => Filter::Disabled,
        };
        cmp.set_config(&config).unwrap();
        assert!(!cmp.is_enabled());
        if config.filter != Filter::Disabled {
            cmp.enable();
            assert_eq!(cmp.is_high(), Err(Error::NotReady));
            // This wait must yield during the >100ms filter-priming interval.
            assert!(
                with_timeout(Duration::from_millis(1), cmp.wait_for_any_edge())
                    .await
                    .is_err()
            );
            assert!(cmp.is_enabled());
            cmp.disable();
        }
        defmt::info!("CMP lifecycle verified; threshold/edge qualification needs known analog stimulus");
        Timer::after_secs(1).await;
    }
}
