#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use core::ops::Div;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::config_types::{CkoutDivider, FilterOrder, FilterParameters, InternalSpiMode};
use embassy_stm32::dfsdm::{
    Dfsdm, FilterConfig, Flt0, Flt1, InjectedTrigger, TransceiverConfig, TransceiverConfigOnline, TransceiverTrait,
};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::DFSDM1;
use embassy_stm32::rcc::{self, Sysclk};
use embassy_stm32::spi::Spi;
use embassy_stm32::time::Hertz;
use embassy_stm32::{SharedData, dfsdm};
use embassy_time::Timer;
use panic_probe as _;

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

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

    //==================================================
    // Goal: Dim LED according to PDM mic input using DFSM, DMA and PWM;
    // Maybe enable breakinput to blink rapidly when detecting disconnection
    //==================================================

    // A0   PA3     MIC_SEL
    // A2   PC3_C   MIC_CLK
    // A4   PC2_C   MIT_DAT

    let p = embassy_stm32::init_primary(config, &SHARED_DATA);
    info!("Hello World!");

    // let mut led = Output::new(p.PB0, Level::High, Speed::Low);
    // let mut led = Output::new(p.PB3, Level::High, Speed::Low);
    // let mut led = Output::new(p.PE1, Level::High, Speed::Low);

    // Setup mic as left channel, data is valid at clock low, to rising clock edge samples signal
    let _mic_sel = Output::new(p.PA3, Level::Low, Speed::Low);

    let prescaler = rcc::frequency::<DFSDM1>() / Hertz::mhz(2);

    println!("Trying prescaler={}", prescaler);
    // Start driver instantiation using DFSDM1 with a CKOUT pin on pin C2
    let dfsdm1 = dfsdm::Dfsdm::new_ckout(
        p.DFSDM1,
        p.PC2,
        dfsdm::config_types::CkoutSource::System,
        CkoutDivider::try_from(prescaler as u16).expect("Divider wrong?"),
    );
    println!("Running with prescaler={}", prescaler);

    let split = dfsdm1.configure_pins(|creator| {
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
    let tcv_cfg = TransceiverConfig::default();
    let tcv_cfg_online = TransceiverConfigOnline::default();
    let channel_mic = split
        .ch1
        .new_spi_int(InternalSpiMode::SpiRising)
        .configure(&tcv_cfg, &tcv_cfg_online)
        .enable();
    // let (mut ch2, mut ch3) = split.ch2.new_parallel_dma_dual(split.ch3);
    // let mut ch2 = ch2.configure(&tcv_cfg, &tcv_cfg_online).enable();
    // let mut ch3 = ch3.configure(&tcv_cfg, &tcv_cfg_online).enable();
    // split.ch6.new_parallel_dma_dual(split.ch7);
    let ch_test = split
        .ch6
        .new_parallel_dma(dfsdm::config_types::DataPackingModeReduced::Standard)
        .configure(&tcv_cfg, &tcv_cfg_online)
        .enable();
    //TODO the enable semantics should really also be linked to channel assignments in filters?

    // split.common.disable();  Common is already enabled from the split!

    let flt_cfg = FilterConfig {
        // filter_cfg: FilterParameters::try_new(FilterOrder::Sinc3 { fosr: 5 }, 4).expect("This is inside the bounds"),
        filter_cfg: FilterParameters::try_new(FilterOrder::Sinc2 { fosr: 10 }, 1).expect("This is inside the bounds"),
    };

    let mut flt1 = split
        .flt1
        .configure(&flt_cfg)
        .assign_regular_transceiver(&ch_test)
        .assign_injected_transceiver(&ch_test)
        .enable();

    let mut flt0 = split
        .flt0
        .configure(&flt_cfg)
        .assign_regular_transceiver(&channel_mic)
        .assign_injected_transceiver(&channel_mic)
        .enable();
    // flt0.start_regular_conversion();

    flt1.start_regular_conversion();

    ch_test.write_sample_standard(10);

    // We're injecting only samples of 10. Meaning the integrated average will be 10*ISR*(OSR)^(FORD-1)
    // flt0.start_injected_conversion();

    loop {
        ch_test.write_sample_standard(10);
        if let Some((data, channel, rpend)) = flt0.try_read_regular_result() {
            println!("New regular 0: ");
            println!("Channel: {}", channel);
            println!("Value: {}", data);
            println!("Delayed: {}", rpend);
            flt0.start_regular_conversion();
        }
        if let Some((data, channel)) = flt0.try_read_injected_result() {
            println!("New injected 0: ");
            println!("Channel: {}", channel);
            println!("Value: {}", data);
            flt0.start_injected_conversion();
        }

        if let Some((data, channel, rpend)) = flt1.try_read_regular_result() {
            println!("New regular 1: ");
            println!("Channel: {}", channel);
            println!("Value: {}", data);
            println!("Delayed: {}", rpend);
            flt1.start_regular_conversion();
        }

        // info!("led on!");
        // led.set_high();
        // Timer::after_millis(500).await;

        // info!("led off!");
        // led.set_low();
        // Timer::after_millis(500).await;
    }
}
