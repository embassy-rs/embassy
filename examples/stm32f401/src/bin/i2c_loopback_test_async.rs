//! I2C Async Loopback Test - Comprehensive slave mode testing
//!
//! Tests the async I2C master and slave implementations using
//! a loopback configuration with I2C1 as slave and I2C2 as master
//! on the same device.
//!
//! Hardware setup (for NUCLEO-F401RE):
//! - Connect PB8 (I2C1 SCL) to PB10 (I2C2 SCL)
//! - Connect PB9 (I2C1 SDA) to PB3 (I2C2 SDA)
//! - Add 4.7k pull-up resistors to 3.3V on both SCL and SDA lines
//!
//! # Test Coverage
//!
//! This test covers:
//! - Write operations (slave receiving data from master)
//! - Read operations (slave sending data to master)
//! - Separate write + read transactions
//! - Combined write_read transactions (RESTART condition)

#![no_std]
#![no_main]

use defmt::{error, info, warn};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::i2c::{self, I2c, SlaveAddrConfig, SlaveCommandKind};
use embassy_stm32::time::khz;
use embassy_stm32::{bind_interrupts, dma, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const I2C_ADDR: u8 = 0x42;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
    DMA1_STREAM6 => dma::InterruptHandler<peripherals::DMA1_CH6>;
    DMA1_STREAM0 => dma::InterruptHandler<peripherals::DMA1_CH0>;
    DMA1_STREAM7 => dma::InterruptHandler<peripherals::DMA1_CH7>;
    DMA1_STREAM3 => dma::InterruptHandler<peripherals::DMA1_CH3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let i2c_frequency = khz(400);

    info!("==============================================");
    info!("I2C Async Loopback Test @ {} kHz", i2c_frequency.0 / 1000);
    info!("I2C1 (slave) on PB8/PB9, I2C2 (master) on PB10/PB3");
    info!("==============================================");

    // I2C1 as slave (PB8=SCL, PB9=SDA)
    let i2c1_config = {
        let mut config = i2c::Config::default();
        config.frequency = i2c_frequency;
        config
    };
    let i2c_slave = I2c::new(p.I2C1, p.PB8, p.PB9, p.DMA1_CH6, p.DMA1_CH0, Irqs, i2c1_config)
        .into_slave_multimaster(SlaveAddrConfig::basic(I2C_ADDR));

    // I2C2 as master (PB10=SCL, PB3=SDA)
    let i2c2_config = {
        let mut config = i2c::Config::default();
        config.frequency = i2c_frequency;
        config
    };
    let i2c_master = I2c::new(p.I2C2, p.PB10, p.PB3, p.DMA1_CH7, p.DMA1_CH3, Irqs, i2c2_config);

    join(slave_task(i2c_slave), master_task(i2c_master)).await;
}

async fn slave_task(mut i2c: I2c<'static, embassy_stm32::mode::Async, i2c::mode::MultiMaster>) {
    info!("[Slave] Ready at address 0x{:02X}", I2C_ADDR);

    loop {
        match i2c.listen().await {
            Ok(command) => match command.kind {
                SlaveCommandKind::Write => {
                    let mut buffer = [0u8; 32];
                    match i2c.respond_to_write(&mut buffer).await {
                        Ok(bytes_received) => {
                            info!(
                                "[Slave] Received {} bytes: {:02X}",
                                bytes_received,
                                &buffer[..bytes_received]
                            );
                        }
                        Err(e) => {
                            error!("[Slave] respond_to_write error: {:?}", e);
                        }
                    }
                }
                SlaveCommandKind::Read => {
                    // Send incrementing pattern for easy verification
                    let response_data = [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7];
                    match i2c.respond_to_read(&response_data).await {
                        Ok(bytes_sent) => {
                            info!("[Slave] Sent {} bytes", bytes_sent);
                        }
                        Err(e) => {
                            error!("[Slave] respond_to_read error: {:?}", e);
                        }
                    }
                }
            },
            Err(e) => {
                warn!("[Slave] Listen error: {:?}", e);
                Timer::after_millis(100).await;
            }
        }
    }
}

async fn master_task(mut i2c: I2c<'static, embassy_stm32::mode::Async, i2c::mode::Master>) {
    // Give slave time to initialize
    Timer::after_millis(100).await;

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
        match i2c.write(I2C_ADDR, &write_data).await {
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
        let write_data = [0x21, 0x22, 0x23, 0x24];
        info!("[Master] Writing {:02X}", write_data);
        match i2c.write(I2C_ADDR, &write_data).await {
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
        let write_data = [0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38];
        info!("[Master] Writing {:02X}", write_data);
        match i2c.write(I2C_ADDR, &write_data).await {
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
        match i2c.read(I2C_ADDR, &mut read_buf).await {
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
        match i2c.read(I2C_ADDR, &mut read_buf).await {
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
        match i2c.read(I2C_ADDR, &mut read_buf).await {
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
    // SEPARATE WRITE + READ TESTS (STOP between transactions)
    // =========================================================================

    // Test: Separate write (2 bytes) + read (4 bytes)
    test_num += 1;
    info!("--- Test {}: Separate write (2 bytes) + read (4 bytes) ---", test_num);
    {
        let write_data = [0x41, 0x42];
        info!("[Master] Writing {:02X}", write_data);
        if let Err(e) = i2c.write(I2C_ADDR, &write_data).await {
            error!("[Master] Write error: {:?}", e);
            failed += 1;
        } else {
            Timer::after_millis(50).await;

            let mut read_buf = [0u8; 4];
            info!("[Master] Reading {} byte(s)", read_buf.len());
            match i2c.read(I2C_ADDR, &mut read_buf).await {
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
        match i2c.write_read(I2C_ADDR, &write_data, &mut read_buf).await {
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
        match i2c.write_read(I2C_ADDR, &write_data, &mut read_buf).await {
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
        match i2c.write_read(I2C_ADDR, &write_data, &mut read_buf).await {
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
