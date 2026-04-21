#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::SharedData;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // 100mhz
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
    }
    let p = embassy_stm32::init_primary(config, &SHARED_DATA);
    info!("Hello World!");

    let mut led1 = Output::new(p.PI12, Level::High, Speed::Low);
    let mut led2 = Output::new(p.PI13, Level::High, Speed::Low);
    let mut led3 = Output::new(p.PI14, Level::High, Speed::Low);
    let mut led4 = Output::new(p.PI15, Level::High, Speed::Low);

    let mut count = 0u8;
    loop {
        led1.set_level(if count & 1 == 0 { Level::High } else { Level::Low });
        led2.set_level(if count & (1 << 1) == 0 { Level::High } else { Level::Low });
        led3.set_level(if count & (1 << 2) == 0 { Level::High } else { Level::Low });
        led4.set_level(if count & (1 << 3) == 0 { Level::High } else { Level::Low });

        Timer::after_millis(500).await;
        count = count.wrapping_add(1);
    }
}
