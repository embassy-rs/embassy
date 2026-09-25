#![no_std]
#![no_main]

//! Dual parallel input -> DFSDM -> two DMA ring buffers, verified against software sums.
//!
//! Like `dfsdm_dma_to_dma.rs` with dual (paired) packing: the even and odd
//! channels feed two filters and two ring buffers, each compared against its
//! own software integration.

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::config::{DataRightShift, FilterOrder, FilterParameters};
use embassy_stm32::dfsdm::{FilterConfig, Flt0, Flt1, ResultRegular};
use embassy_stm32::dma::{self, Channel, TransferOptions};
use embassy_stm32::peripherals::{self, DFSDM1};
use embassy_stm32::{SharedData, bind_interrupts, dfsdm};
use panic_probe as _;

/// Integrator oversampling ratio; the source size, filter and manual model all
/// derive from this, so it can be toggled in one place.
const IOSR: u16 = 32;
/// Number of integrated outputs per channel to compare against the manual sum.
const N_OUT: usize = 4;

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

bind_interrupts! (struct Irqs{
    DFSDM1_FLT0 => dfsdm::InterruptHandler<DFSDM1, Flt0>;
    DFSDM1_FLT1 => dfsdm::InterruptHandler<DFSDM1, Flt1>;
    MDMA => dma::InterruptHandler<peripherals::MDMA_CH0>;
    DMA1_STREAM0 => dma::InterruptHandler<peripherals::DMA1_CH0>;
    DMA1_STREAM1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
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
    // Dual packing: one 16-bit sample for the even channel (INDAT0) and one for
    // the odd channel (INDAT1) per u32 word.
    const TOTAL: usize = IOSR as usize * N_OUT;
    let even: [u16; TOTAL] = core::array::from_fn(|i| lcg(2 * i as u32));
    let odd: [u16; TOTAL] = core::array::from_fn(|i| lcg(2 * i as u32 + 1));
    let source: [u32; TOTAL] = core::array::from_fn(|i| (even[i] as u32) | ((odd[i] as u32) << 16));

    // Setup.
    let pair = split
        .ch0
        .build_parallel_dual(&common, split.ch1)
        .set_data_right_shift([DataRightShift::new(0); 2])
        .enable();

    let filter_params = FilterParameters::try_new(FilterOrder::Disabled, IOSR).expect("inside bounds");
    let flt_cfg0 = FilterConfig::<DFSDM1, Flt0> {
        filter_params,
        enable_continuous_regular: true,
        enable_fast_regular: false,
        ..Default::default()
    };
    let flt_cfg1 = FilterConfig::<DFSDM1, Flt1> {
        filter_params,
        enable_continuous_regular: true,
        enable_fast_regular: false,
        ..Default::default()
    };

    let mut flt0 = split
        .flt0
        .build(&common, Irqs)
        .enable_reg_dma(&pair.even, [&pair.even], &flt_cfg0);
    let mut flt1 = split
        .flt1
        .build(&common, Irqs)
        .enable_reg_dma(&pair.odd, [&pair.odd], &flt_cfg1);

    let mut buffer_even = [0u32; 2 * N_OUT];
    let mut buffer_odd = [0u32; 2 * N_OUT];
    let mut ring_even = flt0.regular.ring_buffered(p.DMA1_CH0, Irqs, &mut buffer_even);
    let mut ring_odd = flt1.regular.ring_buffered(p.DMA1_CH1, Irqs, &mut buffer_odd);
    ring_even.start();
    ring_odd.start();
    ring_even.start_conversion();
    ring_odd.start_conversion();

    // Feed the samples via an MDMA mem2mem transfer into the even channel's DATINR.
    let mut dma_ch = Channel::new(p.MDMA_CH0, Irqs);
    let tfer =
        unsafe { dma_ch.write_mem2mem::<u32, u32>(0, &source, pair.get_datinr_as_ptr(), TransferOptions::default()) };
    tfer.await;

    // Manual integration.
    let manual_even: [i32; N_OUT] = core::array::from_fn(|k| {
        even[k * IOSR as usize..(k + 1) * IOSR as usize]
            .iter()
            .map(|&s| (s as i16) as i32)
            .sum()
    });
    let manual_odd: [i32; N_OUT] = core::array::from_fn(|k| {
        odd[k * IOSR as usize..(k + 1) * IOSR as usize]
            .iter()
            .map(|&s| (s as i16) as i32)
            .sum()
    });

    // Comparison.
    let mut result_even = [0u32; N_OUT];
    let mut result_odd = [0u32; N_OUT];
    ring_even.read(&mut result_even).await.unwrap();
    ring_odd.read(&mut result_odd).await.unwrap();

    let mut all_ok = true;
    for k in 0..N_OUT {
        let e = ResultRegular::from_word(result_even[k]);
        let o = ResultRegular::from_word(result_odd[k]);
        all_ok &= e.data == manual_even[k] && o.data == manual_odd[k];
        info!(
            "out {}: even dfsdm {} vs manual {}, odd dfsdm {} vs manual {}",
            k, e.data, manual_even[k], o.data, manual_odd[k]
        );
    }
    info!("{}", if all_ok { "PASS" } else { "FAIL" });
}
