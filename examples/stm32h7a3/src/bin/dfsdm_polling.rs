#![no_main]
#![no_std]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::{BDMA1_CH0, BDMA2_CH0};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, dfsdm, peripherals};
use embassy_time::Timer;
use panic_probe as _;

// DFSDM1 has one "DFSDM1 filter x interrupt" per filter-channel
// DFSDM2 has one "DFSDM2 interrupt" as it only has one filter-channel
// BDMA1 has only one "BDMA1 global interrupt" as it only serves DFSDM1
// BDMA2 has one "BDMA2 channel x interrupt" per channel as it serves other peripherals aswell (it has 8 channels, only a single one may be used for DFSDM2 at any time)
bind_interrupts!(struct Irqs {
    DFSDM1_FLT0 => dfsdm::InterruptHandler<peripherals::DFSDM1>;
    BDMA1 => embassy_stm32::dma::InterruptHandler<BDMA1_CH0>;
    DFSDM2 => dfsdm::InterruptHandler<peripherals::DFSDM2>;
    BDMA2_CHANNEL0 => embassy_stm32::dma::InterruptHandler<BDMA2_CH0>;
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

    let dfsdm: dfsdm::Dfsdm<'_, peripherals::DFSDM1, dfsdm::ExternalClock> =
        dfsdm::Dfsdm::new(p.DFSDM1, dfsdm::Config::default());
    let dfsdm::TransceiverChannelBuilders8 {
        ch0,
        ch1,
        ch2,
        ch3,
        ch4,
        ch5,
        ch6,
        ch7,
    } = dfsdm.split_8ch_8flt();

    drop(ch0);

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
