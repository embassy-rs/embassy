#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll, Waker};

use defmt_rtt as _;
use embassy_embedded_hal::SetConfig;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::{Duration, Instant, Timer, with_timeout};
use hal::clocks::periph_helpers::FreqmeTargetClockSel;
use hal::freqme::{
    ClockSource, Config, ConversionError, Error, Freqme, InterruptHandler, Measurement, Mode, ReferenceScale,
};
use hal::interrupt::typelevel::Interrupt;
use hal::{bind_interrupts, pac, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    FREQME0 => InterruptHandler<peripherals::FREQME0>;
});

fn assert_idle() {
    let status = pac::FREQME0.ctrlstat().read();
    assert_eq!(status.measure_in_progress().to_bits(), 0);
    assert!(!status.continuous_mode_en());
    assert!(!status.result_ready_int_en());
    assert!(!status.lt_min_int_en());
    assert!(!status.gt_max_int_en());
    assert!(!status.result_ready_stat());
    assert!(!status.lt_min_stat());
    assert!(!status.gt_max_stat());
}

fn assert_not_ready<M: Mode>(driver: &Freqme<'_, M>) {
    assert_eq!(driver.result().unwrap_err(), Error::NotReady);
}

async fn wait_result<M: Mode>(driver: &Freqme<'_, M>) -> Result<Measurement, Error> {
    with_timeout(Duration::from_millis(100), async {
        loop {
            match driver.result() {
                Err(Error::NotReady) => {}
                result => return result,
            }
            Timer::after_micros(100).await;
        }
    })
    .await
    .unwrap()
}

// Exercise the exact same public controls through both Mode instantiations.
// The configured same-FRO12M scale16 window takes about5.46ms per result.
async fn exercise_shared_controls<M: Mode>(driver: &mut Freqme<'_, M>) {
    assert_not_ready(driver);
    driver.continuous();
    driver.stop();
    assert_not_ready(driver);
    assert_idle();
    driver.oneshot();
    assert_not_ready(driver);
    assert!((65_530..=65_540).contains(&wait_result(driver).await.unwrap().raw_count()));
    assert!(!pac::FREQME0.ctrlstat().read().continuous_mode_en());

    driver.continuous();
    assert_not_ready(driver);
    for sample in 0..4 {
        let result = wait_result(driver).await.unwrap();
        assert!((65_530..=65_540).contains(&result.raw_count()));
        assert!(pac::FREQME0.ctrlstat().read().continuous_mode_en());
        assert!(!pac::FREQME0.ctrlstat().read().result_ready_int_en());
        // Reading latest is non-consuming and must not stop counting.
        assert!(driver.result().is_ok());
        // Independently acknowledge ready so the next iteration proves that
        // another window completes, not merely that the first flag stayed set.
        if sample != 3 {
            pac::FREQME0.ctrlstat().write(|w| w.set_result_ready_stat(true));
        }
    }

    // Switching to one-shot starts a fresh window and stops repetition.
    driver.oneshot();
    assert_not_ready(driver);
    assert!((65_530..=65_540).contains(&wait_result(driver).await.unwrap().raw_count()));
    assert!(!pac::FREQME0.ctrlstat().read().continuous_mode_en());
    pac::FREQME0.ctrlstat().write(|w| w.set_result_ready_stat(true));
    Timer::after_millis(15).await;
    assert_not_ready(driver);
    assert_eq!(pac::FREQME0.ctrlstat().read().measure_in_progress().to_bits(), 0);

    driver.continuous();
    assert!((65_530..=65_540).contains(&wait_result(driver).await.unwrap().raw_count()));
    driver.stop();
    assert_not_ready(driver);
    assert_idle();
    Timer::after_millis(10).await;
    assert_not_ready(driver);
    assert_idle();
}

async fn exercise_threshold_errors<M: Mode>(driver: &mut Freqme<'_, M>) {
    for (minimum, maximum, expected) in [
        (70_000, 0x7fff_ffff, Error::BelowMinimum),
        (0, 60_000, Error::AboveMaximum),
    ] {
        pac::FREQME0.min().write(|w| w.set_min_value(0));
        pac::FREQME0.max().write(|w| w.set_max_value(0x7fff_ffff));
        driver.continuous();
        assert!((65_530..=65_540).contains(&wait_result(driver).await.unwrap().raw_count()));
        // Keep READY set for the valid sample, then make the next window fail.
        pac::FREQME0.min().write(|w| w.set_min_value(minimum));
        pac::FREQME0.max().write(|w| w.set_max_value(maximum));
        let error = with_timeout(Duration::from_millis(100), async {
            loop {
                match driver.result() {
                    Ok(_) | Err(Error::NotReady) => Timer::after_micros(100).await,
                    Err(error) => return error,
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(error, expected);
        let status = pac::FREQME0.ctrlstat().read();
        assert!(!status.continuous_mode_en());
        assert_eq!(status.lt_min_stat(), expected == Error::BelowMinimum);
        assert_eq!(status.gt_max_stat(), expected == Error::AboveMaximum);
        // Error reads are non-consuming, just like successful result reads.
        assert_eq!(driver.result().unwrap_err(), expected);
        pac::FREQME0.ctrlstat().write(|w| w.set_result_ready_stat(true));
        assert!(!pac::FREQME0.ctrlstat().read().result_ready_stat());
        assert_eq!(driver.result().unwrap_err(), expected);
        driver.stop();
        assert_not_ready(driver);
        assert_idle();
    }
    pac::FREQME0.min().write(|w| w.set_min_value(0));
    pac::FREQME0.max().write(|w| w.set_max_value(0x7fff_ffff));
    driver.oneshot();
    assert!((65_530..=65_540).contains(&wait_result(driver).await.unwrap().raw_count()));
    driver.stop();
}

async fn exercise_reconfigure<M: Mode>(driver: &mut Freqme<'_, M>, base: Config, guards_before: bool) {
    let mut config = base;
    config.reference_scale = ReferenceScale::new(8).unwrap();
    driver.continuous();
    driver.set_config(&config).unwrap();
    assert_idle();
    assert_not_ready(driver);
    driver.oneshot();
    let result = wait_result(driver).await.unwrap();
    assert!((250..=260).contains(&result.raw_count()));
    // A stale cached scale16 would turn this into about46kHz.
    assert!((11_600_000..=12_200_000).contains(&result.frequency_hz().unwrap()));

    // Reference-frequency caching must change along with INPUTMUX routing.
    config.reference = ClockSource::Clk16k;
    driver.set_config(&config).unwrap();
    driver.oneshot();
    let result = wait_result(driver).await.unwrap();
    assert!((168_750..=206_250).contains(&result.raw_count()));
    assert!((10_800_000..=13_200_000).contains(&result.frequency_hz().unwrap()));

    config = base;
    config.target = FreqmeTargetClockSel::SlowClk;
    driver.set_config(&config).unwrap();
    driver.oneshot();
    let result = wait_result(driver).await.unwrap();
    let expected = hal::clocks::with_clocks(|c| c.cpu_system_clk.as_ref().unwrap().frequency / 6).unwrap();
    assert!(result.frequency_hz().unwrap().abs_diff(expected) <= expected / 10);

    config = base;
    config.power = hal::clocks::PoweredClock::AlwaysEnabled;
    driver.set_config(&config).unwrap();
    assert_eq!(
        critical_section::with(|cs| hal::clocks::active_wake_guards(&cs)),
        guards_before
    );
    driver.set_config(&base).unwrap();
    assert!(critical_section::with(|cs| hal::clocks::active_wake_guards(&cs)));

    for invalid in [
        {
            let mut config = base;
            config.target = FreqmeTargetClockSel::FroHfDiv;
            config
        },
        {
            let mut config = base;
            config.reference = ClockSource::Clk16k;
            config.reference_scale = ReferenceScale::new(22).unwrap();
            config
        },
    ] {
        driver.continuous();
        assert!(driver.set_config(&invalid).is_err());
        assert_idle();
        assert_not_ready(driver);
        // Failure must retain the last accepted routing, frequency and scale.
        driver.oneshot();
        let result = wait_result(driver).await.unwrap();
        assert!((65_530..=65_540).contains(&result.raw_count()));
        assert!((11_998_000..=12_001_000).contains(&result.frequency_hz().unwrap()));
        assert!(critical_section::with(|cs| hal::clocks::active_wake_guards(&cs)));
    }
    driver.stop();
    assert_idle();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Default tree enables FRO12M and FRO16K, but not the HF divider.
    let p = hal::init(Default::default());
    let mut peri = p.FREQME0;
    let guards_before = critical_section::with(|cs| hal::clocks::active_wake_guards(&cs));
    let mut config = Config::default();
    config.reference_scale = ReferenceScale::new(16).unwrap();

    // A source that is not enabled must be rejected for either input.
    let mut disabled = config;
    disabled.reference = ClockSource::FroHfDiv;
    assert!(Freqme::new_blocking(peri.reborrow(), disabled).is_err());
    disabled.reference = ClockSource::Fro12m;
    disabled.target = FreqmeTargetClockSel::FroHfDiv;
    assert!(Freqme::new_async(peri.reborrow(), Irqs, disabled).is_err());
    defmt::info!("PASS: disabled reference and target rejected");

    // 12 MHz / 16,384 Hz * 2^22 = 3,072,000,000 target cycles:
    // this must not silently wrap/saturate the hardware's 31-bit counter.
    let mut overflow = config;
    overflow.reference = ClockSource::Clk16k;
    overflow.reference_scale = ReferenceScale::new(22).unwrap();
    assert!(Freqme::new_blocking(peri.reborrow(), overflow).is_err());
    assert!(Freqme::new_async(peri.reborrow(), Irqs, overflow).is_err());
    defmt::info!("PASS: overflowing target window rejected");

    {
        let mut driver = Freqme::new_blocking(peri.reborrow(), config).unwrap();
        assert!(critical_section::with(|cs| hal::clocks::active_wake_guards(&cs)));
        for _ in 0..4 {
            let result = driver.blocking_measure().unwrap();
            // Same source on both inputs: no oscillator tolerance involved.
            // Allow a few counts for synchronization; the SDK equation is
            // separately checked against the independently observed result.
            assert!((65_530..=65_540).contains(&result.raw_count()));
            assert!((11_998_000..=12_001_000).contains(&result.frequency_hz().unwrap()));
            assert_idle();
        }
        pac::FREQME0.min().write(|w| w.set_min_value(70_000));
        assert_eq!(driver.blocking_measure().unwrap_err(), Error::BelowMinimum);
        assert_idle();
        pac::FREQME0.min().write(|w| w.set_min_value(0));
        exercise_shared_controls(&mut driver).await;
        exercise_threshold_errors(&mut driver).await;
        exercise_reconfigure(&mut driver, config, guards_before).await;
        driver.continuous();
        assert!((65_530..=65_540).contains(&driver.blocking_measure().unwrap().raw_count()));
        // Drop must also stop a live continuous measurement.
        driver.continuous();
    }
    assert_eq!(
        critical_section::with(|cs| hal::clocks::active_wake_guards(&cs)),
        guards_before
    );
    assert_idle();
    // Gate policy belongs to the clock layer, not an erased driver callback.
    critical_section::with(|_| {
        // SAFETY: the driver was dropped and no measurement/source is active.
        unsafe { hal::clocks::disable::<peripherals::FREQME0>() };
    });
    assert!(!pac::MRCC0.mrcc_glb_cc0().read().freqme());
    // Enter the bound handler after explicit gating. It must mask the NVIC
    // line and return without accessing the gated register interface.
    critical_section::with(|_| {
        hal::interrupt::typelevel::FREQME0::pend();
        // SAFETY: Irqs binds the handler, including its gated-entry path.
        unsafe { hal::interrupt::typelevel::FREQME0::enable() };
    });
    for _ in 0..10_000 {
        if !hal::interrupt::typelevel::FREQME0::is_enabled() {
            break;
        }
        cortex_m::asm::nop();
    }
    assert!(!hal::interrupt::typelevel::FREQME0::is_enabled());
    assert!(!hal::interrupt::typelevel::FREQME0::is_pending());
    defmt::info!("PASS: blocking measurements, WakeGuard release and explicit clock gating");
    defmt::info!("PASS: Blocking shared one-shot/continuous controls and continuous drop");
    defmt::info!("PASS: Blocking SetConfig scale, sources, power and failure recovery");
    defmt::info!("PASS: Blocking result/continuous/native-wait threshold errors and recovery");

    {
        let mut always = config;
        always.power = hal::clocks::PoweredClock::AlwaysEnabled;
        let mut driver = Freqme::new_blocking(peri.reborrow(), always).unwrap();
        // Both selected FRO12M inputs run in deep sleep, so this mode must not
        // add an active-only clock vote. This does not exercise sleep entry.
        assert_eq!(
            critical_section::with(|cs| hal::clocks::active_wake_guards(&cs)),
            guards_before
        );
        assert!((65_530..=65_540).contains(&driver.blocking_measure().unwrap().raw_count()));
    }
    assert_eq!(
        critical_section::with(|cs| hal::clocks::active_wake_guards(&cs)),
        guards_before
    );
    defmt::info!("PASS: AlwaysEnabled input validation and WakeGuard policy");

    {
        let mut driver = Freqme::new_async(peri.reborrow(), Irqs, config).unwrap();
        for _ in 0..8 {
            let result = with_timeout(Duration::from_millis(100), driver.measure())
                .await
                .unwrap()
                .unwrap();
            assert!((65_530..=65_540).contains(&result.raw_count()));
            defmt::info!(
                "FRO12M: raw={}, SDK Hz={}",
                result.raw_count(),
                result.frequency_hz().unwrap()
            );
            assert_idle();
        }
        pac::FREQME0.max().write(|w| w.set_max_value(60_000));
        let error = with_timeout(Duration::from_millis(100), driver.measure())
            .await
            .unwrap()
            .unwrap_err();
        assert_eq!(error, Error::AboveMaximum);
        assert_idle();
        pac::FREQME0.max().write(|w| w.set_max_value(0x7fff_ffff));
        exercise_shared_controls(&mut driver).await;
        exercise_threshold_errors(&mut driver).await;
        exercise_reconfigure(&mut driver, config, guards_before).await;
        driver.continuous();
        // Drop a polled future with a real outstanding WaitCell subscription.
        assert!(
            with_timeout(Duration::from_micros(100), driver.measure())
                .await
                .is_err()
        );
        assert_idle();
        let start = Instant::now();
        let result = with_timeout(Duration::from_millis(100), driver.measure())
            .await
            .unwrap()
            .unwrap();
        assert!(start.elapsed() >= Duration::from_millis(4));
        assert!((65_530..=65_540).contains(&result.raw_count()));
        assert_idle();

        // Completion before a second poll must not lose the wake or the ready
        // latch. The IRQ masks the peripheral source and saves RESULT before
        // CTRL_W can reset it; the future must still receive the saved count.
        {
            let mut measurement = pin!(driver.measure());
            let mut cx = Context::from_waker(Waker::noop());
            assert!(measurement.as_mut().poll(&mut cx).is_pending());
            Timer::after_millis(10).await;
            assert!(pac::FREQME0.ctrlstat().read().result_ready_stat());
            assert!(!pac::FREQME0.ctrlstat().read().result_ready_int_en());
            match measurement.as_mut().poll(&mut cx) {
                Poll::Ready(Ok(result)) => assert!((65_530..=65_540).contains(&result.raw_count())),
                _ => panic!("completed measurement was not ready"),
            }
        }
        assert_idle();
        driver.continuous();
    }
    assert_idle();
    defmt::info!("PASS: async IRQ, cancellation/reuse and delayed polling");
    defmt::info!("PASS: Async shared one-shot/continuous controls and continuous drop");
    defmt::info!("PASS: Async SetConfig scale, sources, power and failure recovery");
    defmt::info!("PASS: Async result/continuous/native-wait threshold errors and recovery");

    // A short measurement can complete before the first wait subscription.
    // Small raw counts remain accessible, but conversion must not underflow.
    {
        let mut driver = Freqme::new_async(peri.reborrow(), Irqs, Config::default()).unwrap();
        for _ in 0..8 {
            let result = with_timeout(Duration::from_millis(100), driver.measure())
                .await
                .unwrap()
                .unwrap();
            if result.raw_count() < 2 {
                assert_eq!(result.frequency_hz(), Err(ConversionError::ResultTooSmall));
            }
            assert_idle();
        }
    }
    defmt::info!("PASS: minimum window and checked conversion");

    // A second ratio with independent oscillators exercises routing in both
    // directions. Allow 10% oscillator tolerance, not an exact calibrated Hz.
    config.reference = ClockSource::Clk16k;
    config.target = FreqmeTargetClockSel::Fro12m;
    config.reference_scale = ReferenceScale::new(8).unwrap();
    {
        let mut driver = Freqme::new_async(peri.reborrow(), Irqs, config).unwrap();
        let result = with_timeout(Duration::from_millis(100), driver.measure())
            .await
            .unwrap()
            .unwrap();
        defmt::info!(
            "FRO16K reference / FRO12M target: raw={}, Hz={}",
            result.raw_count(),
            result.frequency_hz().unwrap()
        );
        assert!((10_800_000..=13_200_000).contains(&result.frequency_hz().unwrap()));
    }
    config.reference = ClockSource::Fro12m;
    config.target = FreqmeTargetClockSel::Clk16k;
    config.reference_scale = ReferenceScale::new(20).unwrap();
    {
        let mut driver = Freqme::new_async(peri.reborrow(), Irqs, config).unwrap();
        let result = with_timeout(Duration::from_millis(200), driver.measure())
            .await
            .unwrap()
            .unwrap();
        defmt::info!(
            "FRO12M reference / FRO16K target: raw={}, Hz={}",
            result.raw_count(),
            result.frequency_hz().unwrap()
        );
        assert!((14_745..=18_023).contains(&result.frequency_hz().unwrap()));
    }
    assert_idle();
    assert_eq!(
        critical_section::with(|cs| hal::clocks::active_wake_guards(&cs)),
        guards_before
    );
    defmt::info!("PASS: alternate input routing and reconstruction");

    // SLOW_CLK is a target-only selection: the system/CPU clock divided by
    // six with the reset divider setting, not an always-on oscillator.
    config.reference = ClockSource::Fro12m;
    config.target = FreqmeTargetClockSel::SlowClk;
    config.reference_scale = ReferenceScale::new(16).unwrap();
    let mut always = config;
    always.power = hal::clocks::PoweredClock::AlwaysEnabled;
    assert!(Freqme::new_blocking(peri.reborrow(), always).is_err());
    assert!(Freqme::new_async(peri.reborrow(), Irqs, always).is_err());
    let expected = hal::clocks::with_clocks(|clocks| clocks.cpu_system_clk.as_ref().unwrap().frequency / 6).unwrap();
    {
        let mut driver = Freqme::new_async(peri.reborrow(), Irqs, config).unwrap();
        let selector = critical_section::with(|_| {
            // SAFETY: serialize register-clock access with all INPUTMUX setup;
            // no reset or routing change is performed by this readback.
            let _parts =
                unsafe { hal::clocks::enable::<peripherals::INPUTMUX0>(&hal::clocks::periph_helpers::NoConfig) }
                    .unwrap();
            let selector = pac::INPUTMUX0.freqmeas_tar().read().inp().to_bits();
            unsafe { hal::clocks::disable::<peripherals::INPUTMUX0>() };
            selector
        });
        assert_eq!(selector, 6);
        let result = with_timeout(Duration::from_millis(100), driver.measure())
            .await
            .unwrap()
            .unwrap();
        let hz = result.frequency_hz().unwrap();
        defmt::info!(
            "SLOW_CLK target: raw={}, Hz={}, nominal Hz={}",
            result.raw_count(),
            hz,
            expected
        );
        assert!(hz.abs_diff(expected) <= expected / 10);
        assert_idle();
    }
    assert_eq!(
        critical_section::with(|cs| hal::clocks::active_wake_guards(&cs)),
        guards_before
    );
    defmt::info!("PASS: SLOW_CLK target routing, frequency and active-only validation");
    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
