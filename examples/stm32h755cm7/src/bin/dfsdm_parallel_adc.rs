#![no_std]
#![no_main]

//! Parallel input from the internal ADC: ADC3 converts VREFINT continuously, and
//! DFSDM channel 2 reads the results via the DATMPX=1 hardware path.
//!
//! The mapping is fixed: ADC[y+1] writes DFSDM_CHyDATINR, so ADC3 feeds channel
//! 2. The ADC is started with `Adc::start_dfsdm`, which routes the
//! results to the DFSDM (DMNGT/DFSDMCFG) and leaves them unread.

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::dfsdm::config::{DataRightShift, FilterOrder, FilterParameters};
use embassy_stm32::dfsdm::{Error, FilterConfig, Flt0, ResultRegular};
use embassy_stm32::peripherals::DFSDM1;
use embassy_stm32::{SharedData, bind_interrupts, dfsdm};
use panic_probe as _;

/// Integrator oversampling ratio: the DFSDM sums this many ADC samples per
/// output, which also brings the output rate down so the loop can keep up.
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
        config.rcc.pll2 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul50,
            divp: Some(PllDiv::Div8),
            divq: None,
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
        config.rcc.mux.adcsel = mux::Adcsel::Pll2P;
    }

    let p = embassy_stm32::init_primary(config, &SHARED_DATA);
    info!("Hello World!");

    // ADC3: convert VREFINT continuously, routed to the DFSDM.
    let mut adc = Adc::new_blocking(p.ADC3, Default::default());
    let mut vrefint = adc.enable_vrefint();

    // DFSDM: channel 2 reads ADC3's parallel output (DATMPX=1).
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

    let ch = split
        .ch2
        .build_parallel_adc(&common)
        .set_data_right_shift(DataRightShift::new(0))
        .enable();

    let flt_cfg = FilterConfig {
        filter_params: FilterParameters::try_new(FilterOrder::Disabled, IOSR).expect("inside bounds"),
        enable_continuous_regular: true,
        ..Default::default()
    };
    let mut flt0 = split.flt0.build(&common, Irqs).enable_no_dma(&ch, [&ch], &flt_cfg);

    // Start the ADC converting continuously, routed to the DFSDM; each EOC
    // feeds one DFSDM sample.
    adc.start_dfsdm(&mut vrefint, SampleTime::Cycles3875, None);

    flt0.regular.start_conversion();

    loop {
        match flt0.regular.read().await {
            Ok(ResultRegular { data, .. }) => {
                // `data` is the sum of IOSR samples; divide to get the average.
                info!("vrefint: {}", data / IOSR as i32);
            }
            Err(Error::Overrun) => error!("Overrun!"),
            Err(err) => error!("{:?}", err),
        }
    }
}
