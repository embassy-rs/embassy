#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, dma, peripherals, sdmmc};
use embassy_time::Delay;
use sdio::BlockDevice;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SDMMC1 => sdmmc::InterruptHandler<peripherals::SDMMC1>;
    DMA2_STREAM3 => dma::InterruptHandler<peripherals::DMA2_CH3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::Hse;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul216,
            divp: Some(PllPDiv::Div2), // 8mhz / 4 * 216 / 2 = 216Mhz
            divq: Some(PllQDiv::Div9), // 8mhz / 4 * 216 / 9 = 48Mhz
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div4;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.sys = Sysclk::Pll1P;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut sdmmc = Sdmmc::new_4bit(
        p.SDMMC1,
        p.DMA2_CH3,
        Irqs,
        p.PC12,
        p.PD2,
        p.PC8,
        p.PC9,
        p.PC10,
        p.PC11,
        Default::default(),
    );

    let storage = loop {
        if let Ok(storage) = BlockDevice::<_, _, _, 512>::new_sd_card(&mut sdmmc, 24_000_000, Delay).await {
            break storage;
        }
    };

    let card = storage.card();

    info!("Card: {:#?}", Debug2Format(&card));
}
