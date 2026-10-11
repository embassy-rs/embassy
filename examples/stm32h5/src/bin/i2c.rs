//! I2C master test against an Adafruit MB85RC256V I2C FRAM breakout
//! (256 Kbit / 32 KB Fujitsu FRAM) on a Nucleo-H563ZI.
//!
//! Requires 4 jumper wires between the Zio/Arduino connector CN7 and the
//! breakout:
//!   PB8 (D15, CN7) -> SCL
//!   PB9 (D14, CN7) -> SDA
//!   3V3   (CN7)    -> VCC
//!   GND   (CN7)    -> GND
//!
//! I2C1 runs in fast mode (400 kHz). The breakout ships with A0/A1/A2 tied
//! low, so the device address is 0x50. The MB85RC256V has no page
//! boundaries: writes and reads can span the full 32 KB array.
//!
//! Test 1: 16-byte write at address 0x0000, then write_read (repeated start)
//!         and verify the data read back matches.
//! Test 2: 64-byte write at address 0x0100 across a 256-byte offset
//!         boundary, read back, verify. Then overwrite the location from
//!         test 1 to confirm re-writability.
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Address, Config as I2cConfig, Error, I2c};
use embassy_stm32::rcc::{AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk};
use embassy_stm32::time::mhz;
use embassy_stm32::{Config, bind_interrupts, dma, i2c, peripherals};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    GPDMA1_CHANNEL4 => dma::InterruptHandler<peripherals::GPDMA1_CH4>;
    GPDMA1_CHANNEL5 => dma::InterruptHandler<peripherals::GPDMA1_CH5>;
});

/// MB85RC256V fixed device code with A0/A1/A2 = GND.
const ADDRESS: u8 = 0x50;

const N: usize = 16;
const M: usize = 64;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    config.rcc.hse = Some(Hse {
        freq: mhz(8),
        mode: HseMode::Oscillator,
    });

    config.rcc.pll1 = Some(Pll {
        source: PllSource::Hse,
        prediv: PllPreDiv::Div4,
        mul: PllMul::Mul250,
        divp: Some(PllDiv::Div2),
        divq: Some(PllDiv::Div2),
        divr: Some(PllDiv::Div2),
    });

    config.rcc.sys = Sysclk::Pll1P;
    config.rcc.ahb_pre = AHBPrescaler::Div1;
    config.rcc.apb1_pre = APBPrescaler::Div1;
    config.rcc.apb2_pre = APBPrescaler::Div1;
    config.rcc.apb3_pre = APBPrescaler::Div1;

    let p = embassy_stm32::init(config);

    let mut config = I2cConfig::default();
    config.frequency = mhz(1);

    let mut i2c = I2c::new(p.I2C1, p.PB8, p.PB9, p.GPDMA1_CH4, p.GPDMA1_CH5, Irqs, config);

    info!("hello world");

    // ---------- Test 1: 16-byte write then write_read ----------
    let mut tx = [0u8; N];
    let mut rx = [0u8; N];
    for i in 0..N {
        tx[i] = 0xA0 + i as u8;
    }

    // FRAM memory addresses are 16-bit, sent MSB first.
    let addr: [u8; 2] = [0x00, 0x00];
    match i2c.write_vectored(Address::SevenBit(ADDRESS), &[&addr, &tx]).await {
        Ok(()) => {}
        Err(Error::Timeout) => defmt::panic!("test1: write timed out — is the FRAM wired up?"),
        Err(e) => defmt::panic!("test1: write failed: {:?}", e),
    }

    match i2c.write_read(ADDRESS, &addr, &mut rx).await {
        Ok(()) => {}
        Err(Error::Timeout) => defmt::panic!("test1: read timed out"),
        Err(e) => defmt::panic!("test1: read failed: {:?}", e),
    }

    info!("test1: read back {}", rx);
    defmt::assert!(rx == tx, "test1: data read back does not match data written");
    info!("test1 PASSED: {} bytes written and verified at 0x0000", N);

    // ---------- Test 2: 64-byte write at a higher offset, then rewrite test 1 ----------
    let mut tx2 = [0u8; M];
    let mut rx2 = [0u8; M];
    for i in 0..M {
        tx2[i] = 0x50 + (i % 0x40) as u8;
    }

    let addr2: [u8; 2] = [0x01, 0x00];
    i2c.write_vectored(Address::SevenBit(ADDRESS), &[&addr2, &tx2])
        .await
        .unwrap();
    i2c.write_read(ADDRESS, &addr2, &mut rx2).await.unwrap();

    info!("test2: read back {}", rx2);
    defmt::assert!(rx2 == tx2, "test2: data read back does not match data written");
    info!("test2 PASSED: {} bytes written and verified at 0x0100", M);

    // Overwrite test 1's location with a new pattern to prove re-writability.
    let mut tx3 = [0u8; N];
    for i in 0..N {
        tx3[i] = 0xC0 + i as u8;
    }
    i2c.write_vectored(Address::SevenBit(ADDRESS), &[&addr, &tx3])
        .await
        .unwrap();
    i2c.write_read(ADDRESS, &addr, &mut rx).await.unwrap();
    defmt::assert!(rx == tx3, "test2: re-written data does not match");
    info!("test2 PASSED: FRAM re-write verified");

    info!("DONE");
    cortex_m::asm::bkpt();
    loop {
        Timer::after_secs(1).await;
    }
}
