#![no_std]
#![no_main]

//! Analog watchdog (AWD) demo.
//!
//! Feeds a deterministic ramp through the parallel (CPU-write) input and
//! checks the watchdog end to end:
//!
//! 1. a high-threshold crossing raises a high event, and the event flags
//!    are actually cleared afterwards,
//! 2. a low-threshold crossing raises a low event,
//! 3. the default thresholds never trigger, including with fast mode
//!    (AWFSEL) enabled - they saturate to the i24 "never trigger" extremes.
//!
//! With the filter order disabled, each conversion output is the sum of
//! the last IOSR samples (verified by the `dfsdm_cpu_write` example), so
//! thresholds are exactly predictable.

use core::mem::MaybeUninit;

use defmt::{assert_eq, info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_stm32::dfsdm::config::{DataRightShift, FilterOrder, FilterParameters};
use embassy_stm32::dfsdm::{AnalogWatchdogConfig, AnalogWatchdogEvent, FilterConfig, Flt0};
use embassy_stm32::peripherals::DFSDM1;
use embassy_stm32::{SharedData, bind_interrupts, dfsdm};
use embassy_time::Timer;
use panic_probe as _;

/// Integrator oversampling ratio; conversion outputs are IOSR * sample.
const IOSR: u16 = 32;

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

bind_interrupts!(struct Irqs {
    DFSDM1_FLT0 => dfsdm::InterruptHandler<DFSDM1, Flt0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul50,
            divp: Some(PllDiv::Div2),
            divq: Some(PllDiv::Div8),
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div2;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.apb3_pre = APBPrescaler::Div2;
        config.rcc.apb4_pre = APBPrescaler::Div2;
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
    }

    let p = embassy_stm32::init_primary(config, &SHARED_DATA);
    info!("Hello World!");

    let dfsdm1 = dfsdm::Dfsdm::new(p.DFSDM1);
    let (common, split) = dfsdm1.configure_pins(|creator| {
        (
            creator.ch0.none(),
            creator.ch1.none(),
            creator.ch2.none(),
            creator.ch3.none(),
            creator.ch4.none(),
            creator.ch5.none(),
            creator.ch6.none(),
            creator.ch7.none(),
        )
    });

    // Standard packing: one 16-bit sample per CPU write.
    let ch = split
        .ch0
        .build_parallel_standard(&common)
        .set_data_right_shift(DataRightShift::new(0))
        .enable();

    // Disabled order: each conversion output = sum of the last IOSR samples.
    let flt_cfg = FilterConfig {
        filter_params: FilterParameters::try_new(FilterOrder::Disabled, IOSR).expect("inside bounds"),
        enable_continuous_regular: true,
        enable_fast_regular: false,
        ..Default::default()
    };

    let mut flt0 = split.flt0.build(&common, Irqs).enable_no_dma(&ch, [&ch], &flt_cfg);
    flt0.regular.start_conversion();
    flt0.awd.assign_transceivers([&ch]);

    // Phase 1: input +1000/sample => output +32000 > high threshold +16000.
    flt0.awd.configure(AnalogWatchdogConfig {
        low_threshold: -16000,
        high_threshold: 16000,
        ..Default::default()
    });
    for _ in 0..2 * IOSR as u32 {
        ch.write(1000);
    }
    match flt0.awd.wait_for_event().await {
        AnalogWatchdogEvent::HighThreshold { transceivers } => {
            info!("high event, channels {:b}", transceivers);
            assert_eq!(transceivers, 0b1);
        }
        AnalogWatchdogEvent::LowThreshold { .. } => panic!("expected a high event, got a low one"),
    }
    // Regression check: the event flags must read back cleared.
    assert_eq!(flt0.awd.flags_high(), 0);
    assert_eq!(flt0.awd.flags_low(), 0);

    // Phase 2: input -1000/sample => output -32000 < low threshold -16000.
    for _ in 0..2 * IOSR as u32 {
        ch.write((-1000i16) as u16);
    }
    match flt0.awd.wait_for_event().await {
        AnalogWatchdogEvent::LowThreshold { transceivers } => {
            info!("low event, channels {:b}", transceivers);
            assert_eq!(transceivers, 0b1);
        }
        AnalogWatchdogEvent::HighThreshold { .. } => panic!("expected a low event, got a high one"),
    }
    assert_eq!(flt0.awd.flags_high(), 0);
    assert_eq!(flt0.awd.flags_low(), 0);

    // Phase 3: default thresholds ("never trigger") with fast mode enabled.
    // Flush the integrator with zeros first, clear any residue, then drive
    // alternating +/-30000 (safely inside the i16 extremes under any
    // comparison-source interpretation).
    for _ in 0..2 * IOSR as u32 {
        ch.write(0);
    }
    flt0.awd.clear_flags_high();
    flt0.awd.clear_flags_low();
    flt0.awd.configure(AnalogWatchdogConfig {
        fastmode: true,
        ..Default::default()
    });
    for i in 0..4 * IOSR as u32 {
        ch.write(if i % 2 == 0 { 30000 } else { (-30000i16) as u16 });
    }
    match select(flt0.awd.wait_for_event(), Timer::after_millis(50)).await {
        Either::Second(_) => info!("PASS: default thresholds never trigger (fast mode)"),
        Either::First(_) => panic!("default thresholds triggered (regression)"),
    }
}
