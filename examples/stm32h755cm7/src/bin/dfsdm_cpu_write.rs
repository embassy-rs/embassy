#![no_std]
#![no_main]

//! Parallel input via CPU writes -> DFSDM results, verified against a software sum.
//!
//! Writes one 16-bit sample at a time into DATINR (no DMA) and reads each
//! integrated result, comparing against a software integration of the same
//! samples.

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::config::{DataRightShift, FilterOrder, FilterParameters};
use embassy_stm32::dfsdm::{FilterConfig, Flt0};
use embassy_stm32::peripherals::DFSDM1;
use embassy_stm32::{SharedData, bind_interrupts, dfsdm};
use panic_probe as _;

/// Integrator oversampling ratio; the source size, filter and manual model all
/// derive from this, so it can be toggled in one place.
const IOSR: u16 = 32;
/// Number of integrated outputs to compare against the manual sum.
const N_OUT: usize = 4;

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

bind_interrupts! (struct Irqs{
    DFSDM1_FLT0 => dfsdm::InterruptHandler<DFSDM1, Flt0>;
});

/// Deterministic 16-bit pseudo-random generator.
fn lcg(i: u32) -> u16 {
    (i.wrapping_mul(1664525).wrapping_add(1013904223) >> 16) as u16
}

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

    // Pseudorandom data.
    // Standard packing: one 16-bit sample per CPU write.
    const TOTAL: usize = IOSR as usize * N_OUT;
    let samples: [u16; TOTAL] = core::array::from_fn(|i| lcg(i as u32));

    // Setup.
    let ch = split
        .ch0
        .build_parallel_standard(&common)
        .set_data_right_shift(DataRightShift::new(0))
        .enable();

    let flt_cfg = FilterConfig {
        filter_params: FilterParameters::try_new(FilterOrder::Disabled, IOSR).expect("inside bounds"),
        enable_continuous_regular: true,
        enable_fast_regular: false,
        ..Default::default()
    };

    let mut flt0 = split.flt0.build(&common, Irqs).enable_no_dma(&ch, [&ch], &flt_cfg);
    flt0.regular.start_conversion();

    // Manual integration.
    let manual: [i32; N_OUT] = core::array::from_fn(|k| {
        samples[k * IOSR as usize..(k + 1) * IOSR as usize]
            .iter()
            .map(|&s| (s as i16) as i32)
            .sum()
    });

    // Feed + comparison.
    // Write one integrated group, read its result, so the filter never overruns.
    let mut all_ok = true;
    for k in 0..N_OUT {
        for i in 0..IOSR as usize {
            ch.write(samples[k * IOSR as usize + i]);
        }

        let data = loop {
            if let Ok(r) = flt0.regular.try_get_result() {
                break r.data;
            }
        };

        all_ok &= data == manual[k];
        info!("out {}: dfsdm = {}, manual = {}", k, data, manual[k]);
    }
    info!("{}", if all_ok { "PASS" } else { "FAIL" });
}
