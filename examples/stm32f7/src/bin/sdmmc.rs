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
    config.rcc.sys_ck = Some(200.mhz().into());
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) -> ! {
    info!("Hello World!");

    let irq = interrupt::take!(SDMMC1);

    let mut sdmmc = Sdmmc::new(
        p.SDMMC1,
        (p.PC12, p.PD2, p.PC8, p.PC9, p.PC10, p.PC11),
        irq,
        Default::default(),
        p.DMA2_CH3,
    );

    // Should print 400kHz for initialization
    info!("Configured clock: {}", sdmmc.clock().0);

    unwrap!(sdmmc.init_card(25.mhz()).await);

    let card = unwrap!(sdmmc.card());

    info!("Card: {:#?}", Debug2Format(card));

    loop {}
}
