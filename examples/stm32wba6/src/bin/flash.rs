//! STM32WBA6 blocking flash example.
//!
//! Demonstrates quad-word (16-byte) programming with WDW/BSY handling:
//! - Erase one page (8 KB) in bank 1
//! - Write 32 bytes (2 quad-words) at 16-byte-aligned offset
//! - Read back and verify

#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::flash::Flash;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale, mux,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL30,
        divr: Some(PllDiv::DIV5),
        divq: None,
        divp: Some(PllDiv::DIV30),
        frac: Some(0),
    });
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    config.rcc.sys = Sysclk::PLL1_R;

    // SAI clock source: HSI (flash example does not use SAI; avoids requiring PLL1.P)
    config.rcc.mux.sai1sel = mux::Sai1sel::HSI;

    let p = embassy_stm32::init(config);
    info!("STM32WBA6 Flash example (quad-word programming)");

    // Use high offset in bank 1 to avoid overwriting program (page-aligned: 8 KB).
    const PAGE_SIZE: u32 = 8 * 1024;
    const ADDR: u32 = 0x7_0000; // 448 KB offset in bank 1

    Timer::after_millis(100).await;

    let mut f = Flash::new_blocking(p.FLASH).into_blocking_regions().bank1_region;

    info!("Reading before erase...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing one page ({} bytes)...", PAGE_SIZE);
    unwrap!(f.blocking_erase(ADDR, ADDR + PAGE_SIZE));

    info!("Reading after erase...");
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read after erase: {=[u8]:x}", buf);

    info!("Writing 32 bytes (2 quad-words)...");
    unwrap!(f.blocking_write(
        ADDR,
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32,
        ]
    ));

    info!("Reading after write...");
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    assert_eq!(
        &buf[..],
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32,
        ],
        "Flash read-back mismatch"
    );
    info!("Flash verify OK");
}
