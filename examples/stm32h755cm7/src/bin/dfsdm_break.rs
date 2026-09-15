#![no_std]
#![no_main]

//! Short-circuit detection on the mic triggers a TIM1 break, proving the
//! DFSDM -> TIM break path.
//!
//! TIM1 runs a free-running counter (no LED on this board, so activity is
//! polled from `cnt`). A short-circuit on the mic (stuck data line) asserts the
//! DFSDM BKSCD break wire, routed to TIM1 BRK1; the example polls the TIM1
//! break flag (`SR.BIF`) to report when the break fired.
//!
//! Two details matter: the DFSDM side is armed *before* the TIM break is
//! enabled (so the wire is at its idle level), and the break polarity must be
//! `ActiveHigh` (the DFSDM break output is active high).

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::config::{
    BreakSignals, CkoutDivider, DataRightShift, FilterOrder, FilterParameters, InternalSpiMode,
};
use embassy_stm32::dfsdm::{Detectors, FilterConfig, Flt0, ShortCircuitAssignment};
use embassy_stm32::peripherals::DFSDM1;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{BreakInputPolarity, MasterMode, RoundTo, Timer};
use embassy_stm32::{SharedData, bind_interrupts, dfsdm, rcc};
use embassy_time::Timer as EmbassyTimer;
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

    // DFSDM mic on ch1 + short-circuit detector routed to DFSDM1_BREAK0.
    // Set up first so the break wire is idle before TIM1 starts listening.
    let mic_clk_freq = Hertz::mhz(2);
    let prescaler = rcc::frequency::<DFSDM1>() / mic_clk_freq;
    let dfsdm1 = dfsdm::Dfsdm::new_ckout(
        p.DFSDM1,
        p.PC2,
        dfsdm::config::CkoutSource::System,
        CkoutDivider::try_from(prescaler as u16).expect("Divider wrong?"),
    );

    let (common, split) = dfsdm1.configure_pins(|creator| {
        (
            creator.ch0.none(),
            creator.ch1.datin(p.PC3),
            creator.ch2.none(),
            creator.ch3.none(),
            creator.ch4.none(),
            creator.ch5.none(),
            creator.ch6.none(),
            creator.ch7.none(),
        )
    });

    let channel_mic = split
        .ch1
        .build_spi_int(&common, InternalSpiMode::SpiRising)
        .set_data_right_shift(DataRightShift::new(0))
        .enable();

    let flt_cfg = FilterConfig {
        filter_params: FilterParameters::try_new(FilterOrder::Sinc3 { fosr: 100 }, 50).expect("inside bounds"),
        ..Default::default()
    };
    let mut _flt0 = split
        .flt0
        .build(&common, Irqs)
        .enable_no_dma(&channel_mic, [&channel_mic], &flt_cfg);

    let Detectors {
        mut short_circuit,
        clock_absence: _,
    } = split.detectors.build(&common, Irqs);
    short_circuit.assign_transceivers([ShortCircuitAssignment::new(&channel_mic, 12)]);
    short_circuit.assign_break_signals(&channel_mic, BreakSignals::BREAK0);

    // TIM1: free-running counter, break routed from DFSDM1_BREAK0.
    let tim1 = Timer::new(p.TIM1);
    tim1.set_frequency(Hertz::khz(10), RoundTo::Slower);
    tim1.set_master_mode(MasterMode::Update);
    tim1.set_break_enable(true);
    // The DFSDM break output is active high; ActiveLow would fire constantly.
    tim1.set_break_polarity(BreakInputPolarity::ActiveHigh);
    tim1.set_break_dfsdm_enable(true);
    tim1.start();

    // Clear any break flag latched during setup.
    tim1.regs_advanced().sr().modify(|w| w.set_bif(0, false));

    info!("Trigger a short-circuit on the mic to break TIM1");
    loop {
        let broken = tim1.regs_advanced().sr().read().bif(0);
        info!("cnt: {}, break: {}", tim1.get_counter(), broken);
        EmbassyTimer::after_millis(200).await;
    }
}
