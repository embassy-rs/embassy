#![no_main]
#![no_std]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::{Dfsdm, Flt0, Flt1, TransceiverTrait};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::{BDMA1_CH0, BDMA2_CH0, DFSDM1};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, dfsdm, peripherals};
use embassy_time::Timer;
use panic_probe as _;

// DFSDM1 has one "DFSDM1 filter x interrupt" per filter-channel
// DFSDM2 has one "DFSDM2 interrupt" as it only has one filter-channel
// BDMA1 has only one "BDMA1 global interrupt" as it only serves DFSDM1
// BDMA2 has one "BDMA2 channel x interrupt" per channel as it serves other peripherals aswell (it has 8 channels, only a single one may be used for DFSDM2 at any time)
bind_interrupts!(struct Irqs1 {
    DFSDM2 => dfsdm::InterruptHandler<peripherals::DFSDM2, Flt0>;
    BDMA2_CHANNEL0 => embassy_stm32::dma::InterruptHandler<BDMA2_CH0>;
});

bind_interrupts!(struct Irqs2 {
    DFSDM1_FLT1 => dfsdm::InterruptHandler<peripherals::DFSDM1, Flt1>;
    BDMA1 => embassy_stm32::dma::InterruptHandler<BDMA1_CH0>; //Five it to the dma through the DFSDM driver, somehow it needs it
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // RCC config
    let mut config = Config::default();
    info!("START");
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.csi = true;
        // Needed for USB
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
        // External oscillator 25MHZ
        config.rcc.hse = Some(Hse {
            freq: Hertz(25_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5,
            mul: PllMul::Mul112,
            divp: Some(PllDiv::Div2),
            divq: Some(PllDiv::Div2),
            divr: Some(PllDiv::Div2),
        });
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div2;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.apb3_pre = APBPrescaler::Div2;
        config.rcc.apb4_pre = APBPrescaler::Div2;
        config.rcc.voltage_scale = VoltageScale::Scale0;
    }

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    let mut led = Output::new(p.PE3, Level::Low, Speed::Low);

    let _tim12_ll = embassy_stm32::timer::low_level::Timer::new(p.TIM12);

    // let dfsdm1 = dfsdm::Dfsdm::new(p.DFSDM1, dfsdm::Config::default());
    let dfsdm1 = dfsdm::Dfsdm::new_ckout(p.DFSDM1, p.PC2, dfsdm::Config::default());
    let dfsdm2 = dfsdm::Dfsdm::new(p.DFSDM2, dfsdm::Config::default());

    // let a: dfsdm::Filter<peripherals::DFSDM1, Flt0> = dfsdm1.get_filter_test();
    // a.wait_for_irq().await;
    let test = dfsdm1.get_filter_test::<Flt0>().get_regs_test();
    // let dfsdm1_channels = dfsdm1.split();
    // let dfsdm2_channels = dfsdm2.split();

    // let dfsdm1_ = dfsdm1_channels.ch0.new_clk_int(p.PC1);

    // let dfsdm2_ch0 = dfsdm2_channels.ch0.new_parallel();
    // let _: usize = dfsdm1_.index();
    // let _: usize = dfsdm2_ch0.index();

    let a = dfsdm1.configure_pins(|creator| {
        (
            creator.ch0.datin_ckin(p.PC1, p.PC0),
            creator.ch1.datin(p.PC3),
            creator.ch2.datin_ckin(p.PC5, p.PC4),
            creator.ch3.none(),
            creator.ch4.none(),
            creator.ch5.none(),
            creator.ch6.none(),
            creator.ch7.none(),
        )
    });
    a.ch0.new_synchronous_int();
    a.ch1.new_synchronous_int_neighbor();
    a.ch2.new_synchronous_int();

    let b = dfsdm2.configure_pins(|creator| {
        (
            creator.ch0.datin_ckin(p.PC11, p.PC10),
            creator.ch1.datin_ckin(p.PA7, p.PA2),
        )
    });
    b.ch0.new_parallel_dma();
    b.ch1.new_synchronous_int_neighbor();
    // let dfsdm::ChannelSelectors8 {
    //     ch0,
    //     ch1,
    //     ch2,
    //     ch3,
    //     ch4,
    //     ch5,
    //     ch6,
    //     ch7,
    // } = dfsdm::ChannelSelectors8::<DFSDM1>::new();

    // let ch0 = ch0.datin_ckin(p.PC1, p.PC0);
    // let ch1 = ch1.datin(p.PC3);
    // let ch2 = ch2.datin(p.PC5);

    // let a = dfsdm::DataClk = Pin {};

    // drop(dfsdm1_);

    // dfsdm::TransceiverChannelImp::new_ext_clk(&dfsdm, p.PC0, p.PC1);
    // dfsdm::TransceiverChannelImp::<_, dfsdm::Tcv0, _>::new_parallel(&dfsdm);

    // dfsdm::TransceiverChannelImp::new_ext_clk(dfsdm, ckin, datin)
    // dfsdm::TransceiverChannelImp::new(p.DFSDM1, p.PB2, p.PC3);
    // dfsdm::TransceiverChannelImp::new(p.DFSDM1, p.PC0, p.PC3);
    // let transceiver = dfsdm::TransceiverChannelImp::new_ch0(p.DFSDM1, p.PC0, p.PC1);
    // let dfsdm1 = dfsdm::Dfsdm::new(p.DFSDM1, p.BDMA1_CH0, Irqs, p.PC0, dfsdm::Config::default());
    // let dfsdm2 = dfsdm::Dfsdm::new(p.DFSDM2, p.BDMA2_CH0, Irqs, p.PC10, dfsdm::Config::default());

    loop {
        led.toggle();
        Timer::after_millis(1000).await;
    }
}
