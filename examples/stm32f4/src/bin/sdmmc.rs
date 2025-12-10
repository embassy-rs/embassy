#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::sdmmc::{CmdBlock, DataBlock, Sdmmc, StorageDevice};
use embassy_stm32::time::{Hertz, mhz};
use embassy_stm32::{Config, bind_interrupts, peripherals, sdmmc};
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
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL168,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 168 / 2 = 168Mhz.
            divq: Some(PllQDiv::DIV7), // 8mhz / 4 * 168 / 7 = 48Mhz.
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }
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

    let mut cmd_block = CmdBlock::new();

    let mut storage = StorageDevice::new_sd_card(&mut sdmmc, &mut cmd_block, mhz(24))
        .await
        .unwrap();

    let card = storage.card();

    info!("Card: {:#?}", Debug2Format(&card));
    info!("Clock: {}", storage.sdmmc.clock());

    // Arbitrary block index
    let block_idx = 16;

    // SDMMC uses `DataBlock` instead of `&[u8]` to ensure 4 byte alignment required by the hardware.
    let mut block = DataBlock([0u8; 512]);

    storage.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);

    if !ALLOW_WRITES {
        info!("Writing is disabled.");
        loop {}
    }

    info!("Filling block with 0x55");
    block.fill(0x55);
    storage.write_block(block_idx, &block).await.unwrap();
    info!("Write done");

    storage.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);

    info!("Filling block with 0xAA");
    block.fill(0xAA);
    storage.write_block(block_idx, &block).await.unwrap();
    info!("Write done");

    storage.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);
}
