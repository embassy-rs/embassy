#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::{Dfsdm, Flt0, Flt1, InjectedTrigger, TransceiverTrait};
use embassy_stm32::gpio::{Level, Output, Speed};
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
    let p = embassy_stm32::init_primary(config, &SHARED_DATA);
    info!("Hello World!");

    let mut led = Output::new(p.PB0, Level::High, Speed::Low);
    let mut led = Output::new(p.PB3, Level::High, Speed::Low);
    let mut led = Output::new(p.PE1, Level::High, Speed::Low);

    let dfsdm1 = dfsdm::Dfsdm::new_ckout(p.DFSDM1, p.PC2, dfsdm::Config::default());
    let a = dfsdm1.configure_pins(|creator| {
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
    let (mut ch2, mut ch3) = a.ch2.new_parallel_dma_dual(a.ch3);
    a.ch6.new_parallel_dma_dual(a.ch7);

    let mut channel_mic = a.ch1.new_synchronous_ext();

    loop {
        info!("led on!");
        led.set_high();
        Timer::after_millis(500).await;

        info!("led off!");
        led.set_low();
        Timer::after_millis(500).await;
    }
}
