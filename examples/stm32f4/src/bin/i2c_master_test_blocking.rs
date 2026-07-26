//! I2C Blocking Master Test
//!
//! Tests the blocking I2C master implementation with an external I2C slave device.
//! This is useful for testing with a logic analyzer or external slave like the
//! Digilent Analog Discovery.
//!
//! Hardware setup (for NUCLEO-F401RE):
//! - PB8: I2C1 SCL
//! - PB9: I2C1 SDA
//! - Add 4.7k pull-up resistors to 3.3V on both SCL and SDA lines
//! - Connect to external I2C slave device
//!
//! # Test Coverage
//!
//! This test covers:
//! - Write operations (slave receiving data from master)
//! - Read operations (slave sending data to master)
//! - Combined write_read transactions (RESTART condition)

#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::time::khz;
use embassy_stm32::{bind_interrupts, dma, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const I2C_ADDR: u8 = 0x42;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    DMA1_STREAM0 => dma::InterruptHandler<peripherals::DMA1_CH0>;
    DMA1_STREAM6 => dma::InterruptHandler<peripherals::DMA1_CH6>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let i2c_frequency = khz(100);

    info!("==============================================");
    info!("I2C Blocking Master Test @ {} kHz", i2c_frequency.0 / 1000);
    info!("I2C1 (master) on PB8 (SCL) / PB9 (SDA)");
    info!("Slave address: 0x{:02X}", I2C_ADDR);
    info!("==============================================");

    // I2C1 as master (PB8=SCL, PB9=SDA)
    // Note: We still need DMA channels for the I2c::new constructor, but blocking
    // operations don't use them.
    let i2c_config = {
        let mut config = i2c::Config::default();
        config.frequency = i2c_frequency;
        config
    };
    let mut i2c = I2c::new(p.I2C1, p.PB8, p.PB9, p.DMA1_CH6, p.DMA1_CH0, Irqs, i2c_config);

    // Give external slave time to initialize
    Timer::after_millis(500).await;

    info!("[Master] Starting test suite...");
    info!("");

    let mut test_num = 0;
    let mut passed = 0;
    let mut failed = 0;

    // =========================================================================
    // WRITE TESTS - Master writes to slave
    // =========================================================================

    // Test: Write 1 byte
    test_num += 1;
    info!("--- Test {}: Write 1 byte ---", test_num);
    {
        let write_data = [0x11];
        info!("[Master] Writing {:02X}", write_data);
        match i2c.blocking_write(I2C_ADDR, &write_data) {
            Ok(()) => {
                info!("[Master] Write successful");
                passed += 1;
            }
            Err(e) => {
                error!("[Master] Write error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // Test: Write 2 bytes
    test_num += 1;
    info!("--- Test {}: Write 2 bytes ---", test_num);
    {
        let write_data = [0x21, 0x22];
        info!("[Master] Writing {:02X}", write_data);
        match i2c.blocking_write(I2C_ADDR, &write_data) {
            Ok(()) => {
                info!("[Master] Write successful");
                passed += 1;
            }
            Err(e) => {
                error!("[Master] Write error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // Test: Write 4 bytes
    test_num += 1;
    info!("--- Test {}: Write 4 bytes ---", test_num);
    {
        let write_data = [0x31, 0x32, 0x33, 0x34];
        info!("[Master] Writing {:02X}", write_data);
        match i2c.blocking_write(I2C_ADDR, &write_data) {
            Ok(()) => {
                info!("[Master] Write successful");
                passed += 1;
            }
            Err(e) => {
                error!("[Master] Write error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // Test: Write 8 bytes
    test_num += 1;
    info!("--- Test {}: Write 8 bytes ---", test_num);
    {
        let write_data = [0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48];
        info!("[Master] Writing {:02X}", write_data);
        match i2c.blocking_write(I2C_ADDR, &write_data) {
            Ok(()) => {
                info!("[Master] Write successful");
                passed += 1;
            }
            Err(e) => {
                error!("[Master] Write error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // =========================================================================
    // READ TESTS - Master reads from slave
    // =========================================================================

    // Test: Read 1 byte
    test_num += 1;
    info!("--- Test {}: Read 1 byte ---", test_num);
    {
        let mut read_buf = [0u8; 1];
        info!("[Master] Reading {} byte(s)", read_buf.len());
        match i2c.blocking_read(I2C_ADDR, &mut read_buf) {
            Ok(()) => {
                info!("[Master] Read result: {:02X}", read_buf);
                passed += 1;
            }
            Err(e) => {
                error!("[Master] Read error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // Test: Read 2 bytes
    test_num += 1;
    info!("--- Test {}: Read 2 bytes ---", test_num);
    {
        let mut read_buf = [0u8; 2];
        info!("[Master] Reading {} byte(s)", read_buf.len());
        match i2c.blocking_read(I2C_ADDR, &mut read_buf) {
            Ok(()) => {
                info!("[Master] Read result: {:02X}", read_buf);
                passed += 1;
            }
            Err(e) => {
                error!("[Master] Read error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // Test: Read 4 bytes
    test_num += 1;
    info!("--- Test {}: Read 4 bytes ---", test_num);
    {
        let mut read_buf = [0u8; 4];
        info!("[Master] Reading {} byte(s)", read_buf.len());
        match i2c.blocking_read(I2C_ADDR, &mut read_buf) {
            Ok(()) => {
                info!("[Master] Read result: {:02X}", read_buf);
                passed += 1;
            }
            Err(e) => {
                error!("[Master] Read error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // Test: Read 8 bytes
    test_num += 1;
    info!("--- Test {}: Read 8 bytes ---", test_num);
    {
        let mut read_buf = [0u8; 8];
        info!("[Master] Reading {} byte(s)", read_buf.len());
        match i2c.blocking_read(I2C_ADDR, &mut read_buf) {
            Ok(()) => {
                info!("[Master] Read result: {:02X}", read_buf);
                passed += 1;
            }
            Err(e) => {
                error!("[Master] Read error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // =========================================================================
    // COMBINED WRITE_READ TESTS (RESTART condition)
    // =========================================================================

    // Test: Combined write_read (1 byte write, 4 byte read)
    test_num += 1;
    info!("--- Test {}: Combined write_read (1 byte write) ---", test_num);
    {
        let write_data = [0x51];
        let mut read_buf = [0u8; 4];
        info!(
            "[Master] write_read: writing {:02X}, reading {} bytes",
            write_data,
            read_buf.len()
        );
        match i2c.blocking_write_read(I2C_ADDR, &write_data, &mut read_buf) {
            Ok(()) => {
                info!("[Master] Read result: {:02X}", read_buf);
                passed += 1;
            }
            Err(e) => {
                error!("[Master] write_read error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // Test: Combined write_read (2 byte write, 4 byte read)
    test_num += 1;
    info!("--- Test {}: Combined write_read (2 byte write) ---", test_num);
    {
        let write_data = [0x61, 0x62];
        let mut read_buf = [0u8; 4];
        info!(
            "[Master] write_read: writing {:02X}, reading {} bytes",
            write_data,
            read_buf.len()
        );
        match i2c.blocking_write_read(I2C_ADDR, &write_data, &mut read_buf) {
            Ok(()) => {
                info!("[Master] Read result: {:02X}", read_buf);
                passed += 1;
            }
            Err(e) => {
                error!("[Master] write_read error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // Test: Combined write_read (4 byte write, 4 byte read)
    test_num += 1;
    info!("--- Test {}: Combined write_read (4 byte write) ---", test_num);
    {
        let write_data = [0x71, 0x72, 0x73, 0x74];
        let mut read_buf = [0u8; 4];
        info!(
            "[Master] write_read: writing {:02X}, reading {} bytes",
            write_data,
            read_buf.len()
        );
        match i2c.blocking_write_read(I2C_ADDR, &write_data, &mut read_buf) {
            Ok(()) => {
                info!("[Master] Read result: {:02X}", read_buf);
                passed += 1;
            }
            Err(e) => {
                error!("[Master] write_read error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // Test: Combined write_read (8 byte write, 4 byte read)
    test_num += 1;
    info!("--- Test {}: Combined write_read (8 byte write) ---", test_num);
    {
        let write_data = [0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88];
        let mut read_buf = [0u8; 4];
        info!(
            "[Master] write_read: writing {:02X}, reading {} bytes",
            write_data,
            read_buf.len()
        );
        match i2c.blocking_write_read(I2C_ADDR, &write_data, &mut read_buf) {
            Ok(()) => {
                info!("[Master] Read result: {:02X}", read_buf);
                passed += 1;
            }
            Err(e) => {
                error!("[Master] write_read error: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after_millis(100).await;

    // =========================================================================
    // TEST SUMMARY
    // =========================================================================

    info!("");
    info!("==============================================");
    info!("Test Summary: {} passed, {} failed", passed, failed);
    info!("==============================================");

    // Keep running
    loop {
        Timer::after_millis(1000).await;
    }
}
