#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::time::mhz;
use embassy_stm32::{interrupt, Config};
use {defmt_rtt as _, panic_probe as _};

#[repr(align(4))]
struct AlignedBuffer([u8; 512]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(mhz(48));
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let irq = interrupt::take!(SDIO);

    let mut sdmmc = Sdmmc::new_4bit(
        p.SDIO,
        irq,
        p.DMA2_CH3,
        p.PC12,
        p.PD2,
        p.PC8,
        p.PC9,
        p.PC10,
        p.PC11,
        Default::default(),
    );

    // Should print 400kHz for initialization
    info!("Configured clock: {}", sdmmc.clock().0);

    unwrap!(sdmmc.init_card(mhz(2)).await);

    let card = unwrap!(sdmmc.card());

    info!("Card: {:#?}", Debug2Format(card));
    info!("Clock: {}", sdmmc.clock());

    let mut buf = AlignedBuffer([0u8; 512]);

    info!("read");
    sdmmc.read_block(512, &mut buf.0).await.unwrap();
    info!("read done: {:?}, {:?}", buf.0[..10], buf.0[502..]);

    let buf = AlignedBuffer([6u8; 512]);

    info!("writing");
    sdmmc.write_block(0, &buf.0).await.unwrap();
    info!("Write done!");

    loop {}
}
