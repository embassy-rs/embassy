#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::executor::Spawner;
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::time::U32Ext;
use embassy_stm32::{interrupt, Config, Peripherals};
use example_common::*;

fn config() -> Config {
    let mut config = Config::default();
    config.rcc.hse = Some(8.mhz().into());
    config.rcc.hclk = Some(48.mhz().into());
    config.rcc.pclk2 = Some(48.mhz().into());
    config.rcc.pll48 = true;
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) -> ! {
    info!("Hello World, dude!");

    let irq = interrupt::take!(SDIO);

    let mut sdmmc = unsafe {
        Sdmmc::new(
            p.SDIO,
            (p.PC12, p.PD2, p.PC8, p.PC9, p.PC10, p.PC11),
            irq,
            Default::default(),
            p.DMA2_CH3,
        )
    };

    info!("Configured clock: {}", sdmmc.clock.0);

    unwrap!(sdmmc.init_card(25.mhz()).await);

    let card = unwrap!(sdmmc.card());

    info!("Card: {:#?}", Debug2Format(card));

    loop {}
}
