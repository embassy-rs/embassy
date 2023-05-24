#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::sdmmc::{DataBlock, Sdmmc};
use embassy_stm32::time::mhz;
use embassy_stm32::{bind_interrupts, peripherals, sdmmc, Config};
use {defmt_rtt as _, panic_probe as _};

/// This is a safeguard to not overwrite any data on the SD card.
/// If you don't care about SD card contents, set this to `true` to test writes.
const ALLOW_WRITES: bool = false;

bind_interrupts!(struct Irqs {
    SDIO => sdmmc::InterruptHandler<peripherals::SDIO>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(mhz(48));
    config.rcc.pll48 = true;
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut sdmmc = Sdmmc::new_4bit(
        p.SDIO,
        Irqs,
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

    let mut err = None;
    loop {
        match sdmmc.init_card(mhz(24)).await {
            Ok(_) => break,
            Err(e) => {
                if err != Some(e) {
                    info!("waiting for card error, retrying: {:?}", e);
                    err = Some(e);
                }
            }
        }
    }

    let card = unwrap!(sdmmc.card());

    info!("Card: {:#?}", Debug2Format(card));
    info!("Clock: {}", sdmmc.clock());

    // Arbitrary block index
    let block_idx = 16;

    // SDMMC uses `DataBlock` instead of `&[u8]` to ensure 4 byte alignment required by the hardware.
    let mut block = DataBlock([0u8; 512]);

    sdmmc.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);

    if !ALLOW_WRITES {
        info!("Writing is disabled.");
        loop {}
    }

    info!("Filling block with 0x55");
    block.fill(0x55);
    sdmmc.write_block(block_idx, &block).await.unwrap();
    info!("Write done");

    sdmmc.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);

    info!("Filling block with 0xAA");
    block.fill(0xAA);
    sdmmc.write_block(block_idx, &block).await.unwrap();
    info!("Write done");

    sdmmc.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);
}
