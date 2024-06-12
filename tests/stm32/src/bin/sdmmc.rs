// required-features: sdmmc
#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_stm32::sdmmc::{DataBlock, Sdmmc};
use embassy_stm32::time::mhz;
use embassy_stm32::{bind_interrupts, peripherals, sdmmc};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SDIO => sdmmc::InterruptHandler<peripherals::SDIO>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_stm32::init(config());

    let (mut sdmmc, mut dma, mut clk, mut cmd, mut d0, mut d1, mut d2, mut d3) =
        (p.SDIO, p.DMA2_CH3, p.PC12, p.PD2, p.PC8, p.PC9, p.PC10, p.PC11);

    // Arbitrary block index
    let block_idx = 16;

    let mut pattern1 = DataBlock([0u8; 512]);
    let mut pattern2 = DataBlock([0u8; 512]);
    for i in 0..512 {
        pattern1[i] = i as u8;
        pattern2[i] = !i as u8;
    }

    let mut block = DataBlock([0u8; 512]);

    // ======== Try 4bit. ==============
    info!("initializing in 4-bit mode...");
    let mut s = Sdmmc::new_4bit(
        &mut sdmmc,
        Irqs,
        &mut dma,
        &mut clk,
        &mut cmd,
        &mut d0,
        &mut d1,
        &mut d2,
        &mut d3,
        Default::default(),
    );

    let mut err = None;
    loop {
        match s.init_card(mhz(24)).await {
            Ok(_) => break,
            Err(e) => {
                if err != Some(e) {
                    info!("waiting for card: {:?}", e);
                    err = Some(e);
                }
            }
        }
    }

    let card = unwrap!(s.card());

    info!("Card: {:#?}", Debug2Format(card));
    info!("Clock: {}", s.clock());

    info!("writing pattern1...");
    s.write_block(block_idx, &pattern1).await.unwrap();

    info!("reading...");
    s.read_block(block_idx, &mut block).await.unwrap();
    assert_eq!(block, pattern1);

    info!("writing pattern2...");
    s.write_block(block_idx, &pattern2).await.unwrap();

    info!("reading...");
    s.read_block(block_idx, &mut block).await.unwrap();
    assert_eq!(block, pattern2);

    drop(s);

    // ======== Try 1bit. ==============
    info!("initializing in 1-bit mode...");
    let mut s = Sdmmc::new_1bit(
        &mut sdmmc,
        Irqs,
        &mut dma,
        &mut clk,
        &mut cmd,
        &mut d0,
        Default::default(),
    );

    let mut err = None;
    loop {
        match s.init_card(mhz(24)).await {
            Ok(_) => break,
            Err(e) => {
                if err != Some(e) {
                    info!("waiting for card: {:?}", e);
                    err = Some(e);
                }
            }
        }
    }

    let card = unwrap!(s.card());

    info!("Card: {:#?}", Debug2Format(card));
    info!("Clock: {}", s.clock());

    info!("reading pattern2 written in 4bit mode...");
    s.read_block(block_idx, &mut block).await.unwrap();
    assert_eq!(block, pattern2);

    info!("writing pattern1...");
    s.write_block(block_idx, &pattern1).await.unwrap();

    info!("reading...");
    s.read_block(block_idx, &mut block).await.unwrap();
    assert_eq!(block, pattern1);

    info!("writing pattern2...");
    s.write_block(block_idx, &pattern2).await.unwrap();

    info!("reading...");
    s.read_block(block_idx, &mut block).await.unwrap();
    assert_eq!(block, pattern2);

    drop(s);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
