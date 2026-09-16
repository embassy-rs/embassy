#![no_std]
#![no_main]

//! Triggered injected conversions on the STM32H755 (CM7).
//!
//! Like `dfsdm_pwm_injected_sc.rs` without the short-circuit handling: the
//! software conversion start is replaced by an external trigger — TIM1 TRGO
//! (update event) launches each injected conversion, so `flt0.injected.read()`
//! simply awaits the next triggered result.
//!
//! The filter produces ~400 Hz of conversions (2 MHz CKOUT / (FOSR=100 *
//! IOSR=50)); TIM1 is set slightly slower (350 Hz) so each conversion finishes
//! before the next trigger and never overruns.

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::config::{CkoutDivider, FilterOrder, FilterParameters, InternalSpiMode, TriggerEdge};
use embassy_stm32::dfsdm::{FilterConfig, Flt0, InjectedTrigger, ResultInjected};
use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::peripherals::DFSDM1;
use embassy_stm32::rcc::{self};
use embassy_stm32::time::{Hertz, khz};
use embassy_stm32::timer::low_level::{MasterMode, RoundTo, Timer};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::triggers::TIM1_TRGO;
use embassy_stm32::{SharedData, bind_interrupts, dfsdm};
use embassy_stm32h755cm7_examples::dsp::{LevelDsp, Meter};
use embassy_time::Instant;
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

    // A0   PA3     MIC_SEL
    // A2   PC3_C   MIT_DAT
    // A4   PC2_C   MIC_CLK

    let p = embassy_stm32::init_primary(config, &SHARED_DATA);
    info!("Hello World!");

    // LED PWM on PB14 (TIM12).
    let ld2_pwm_pin: PwmPin<'_, embassy_stm32::peripherals::TIM12, embassy_stm32::timer::Ch1> =
        PwmPin::new(p.PB14, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM12,
        Some(ld2_pwm_pin),
        None,
        None,
        None,
        khz(10),
        Default::default(),
    );
    let mut pwm_ld2 = pwm.ch1();
    pwm_ld2.enable();

    // Mic as left channel: data valid at clock low, sampled on rising edge.
    let _mic_sel = Output::new(p.PA3, Level::Low, Speed::Low);

    let mic_clk_freq = Hertz::mhz(2);
    let prescaler = rcc::frequency::<DFSDM1>() / mic_clk_freq;

    println!("Trying prescaler={}", prescaler);
    let dfsdm1 = dfsdm::Dfsdm::new_ckout(
        p.DFSDM1,
        p.PC2,
        dfsdm::config::CkoutSource::System,
        CkoutDivider::try_from(prescaler as u16).expect("Divider wrong?"),
    );
    println!("Running with prescaler={}", prescaler);

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

    // 2MHz/100/50 = 400Hz conversion rate.
    let filter_params =
        FilterParameters::try_new(FilterOrder::Sinc3 { fosr: 100 }, 50).expect("This is inside the bounds");

    let channel_mic = split
        .ch1
        .build_spi_int(&common, InternalSpiMode::SpiRising)
        .set_data_right_shift(filter_params.recommended_shift().try_into().unwrap())
        .enable();

    // TIM1 TRGO (update event) launches each injected conversion.
    let flt_cfg = FilterConfig {
        filter_params,
        trigger: InjectedTrigger::from(TIM1_TRGO, TriggerEdge::Rising),
        ..Default::default()
    };
    let mut flt0 = split
        .flt0
        .build(&common, Irqs)
        .enable_no_dma(&channel_mic, [&channel_mic], &flt_cfg);

    // TIM1: 350 Hz trigger. Slightly slower than the 400 Hz ceiling so each
    // conversion finishes before the next trigger (no overrun).
    let tim1 = Timer::new(p.TIM1);
    tim1.set_frequency(Hertz::hz(350), RoundTo::Slower);
    tim1.set_master_mode(MasterMode::Update);
    tim1.start();

    let mut dsp = LevelDsp::new();
    let mut meter = Meter::new();
    let mut wait_start = Instant::now();

    loop {
        let ResultInjected { data, .. } = flt0.injected.read().await.expect("Error");
        let ready_at = Instant::now();

        let duty = dsp.process(data, pwm_ld2.max_duty_cycle());
        pwm_ld2.set_duty_cycle(duty);

        meter.record(ready_at - wait_start, Instant::now() - ready_at);
        wait_start = Instant::now();
        meter.report();
    }
}
