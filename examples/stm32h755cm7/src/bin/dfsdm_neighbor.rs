#![no_std]
#![no_main]

//! DFSDM neighbor-pin build demo.
//!
//! Builds channel 1 as `build_spi_int_neighbor`, so it reads *channel 2's*
//! DATIN pin instead of declaring its own. Exercises the two-reservation pin
//! model: channel 1's own slot is disclaimed at build time, and channel 2's
//! slot reservation is released when the transceiver drops.

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::config::{CkoutDivider, FilterOrder, FilterParameters, InternalSpiMode};
use embassy_stm32::dfsdm::{FilterConfig, Flt0, ResultRegular};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::DFSDM1;
use embassy_stm32::rcc::{self};
use embassy_stm32::time::Hertz;
use embassy_stm32::{SharedData, bind_interrupts, dfsdm};
use panic_probe as _;

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
            divq: Some(PllDiv::Div8), // 100mhz
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::Div2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
    }

    let p = embassy_stm32::init_primary(config, &SHARED_DATA);
    info!("Hello World!");

    // Mic as left channel: data valid at clock low, sampled on rising edge.
    let _mic_sel = Output::new(p.PA3, Level::Low, Speed::Low);

    let mic_clk_freq = Hertz::mhz(2);
    let prescaler = rcc::frequency::<DFSDM1>() / mic_clk_freq;

    // CKOUT on PC2.
    let dfsdm1 = dfsdm::Dfsdm::new_ckout(
        p.DFSDM1,
        p.PC2,
        dfsdm::config::CkoutSource::System,
        CkoutDivider::try_from(prescaler as u16).expect("Divider wrong?"),
    );

    // Declare the mic data pin on channel 2. Channel 1 will borrow it.
    let (common, split) = dfsdm1.configure_pins(|creator| {
        (
            creator.ch0.none(),
            creator.ch1.none(),
            creator.ch2.datin(p.PC5),
            creator.ch3.none(),
            creator.ch4.none(),
            creator.ch5.none(),
            creator.ch6.none(),
            creator.ch7.none(),
        )
    });

    // Channel 1 reads channel 2's DATIN pin (CHINSEL=1, next channel).
    let channel_mic = split
        .ch1
        .build_spi_int_neighbor(&common, InternalSpiMode::SpiRising)
        .set_data_right_shift(
            FilterParameters::try_new(FilterOrder::Sinc3 { fosr: 100 }, 50)
                .expect("inside bounds")
                .recommended_shift()
                .try_into()
                .unwrap(),
        )
        .enable();

    let flt_cfg = FilterConfig {
        filter_params: FilterParameters::try_new(FilterOrder::Sinc3 { fosr: 100 }, 50).expect("inside bounds"),
        ..Default::default()
    };

    let mut flt0 = split
        .flt0
        .build(&common, Irqs)
        .enable_no_dma(&channel_mic, [&channel_mic], &flt_cfg);

    flt0.regular.start_conversion();
    info!("Reading neighbor-fed channel 1; polling for 2s...");

    // Bounded polling: this is a build/pin-lifecycle smoke test, not a data
    // test (the pin may be floating). It must run without panicking.
    let mut count = 0u32;
    let mut polls = 0u32;
    while polls < 200_000 {
        polls += 1;
        if let Ok(ResultRegular { data, .. }) = flt0.regular.try_get_result() {
            flt0.regular.start_conversion();
            count += 1;
            if count % 25 == 0 {
                info!("sample {}: {}", count, data);
            }
        }
    }
    info!("PASS: neighbor-fed build ran; {} conversions seen", count);
}
