#![no_std]
#![no_main]
// required-features: stm32f072rb
//
// Hardware Setup for NUCLEO-F072RB:
// - I2C1 pins: PB8 (SCL), PB9 (SDA) on CN5 connector
// - Connect to I2C slave device (e.g., Digilent Analog Discovery I2C slave, or EEPROM)
// - Default slave address: 0x50
// - Pull-up resistors: 4.7kΩ on both SCL and SDA
// - CN5 Pin 10 (PB8/SCL) and CN5 Pin 9 (PB9/SDA)
//
// Analog Discovery Setup:
// - Configure as I2C Slave at address 0x50
// - DIO 0: SCL
// - DIO 1: SDA
// - Enable pull-ups or use external 4.7kΩ pull-up resistors

#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Config, I2c, Master};
use embassy_stm32::mode::{Async, Blocking};
use embassy_stm32::time::Hertz;
use embassy_time::Timer;
use embedded_hal_1::i2c::Operation;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init();
    info!("I2C v2 Transaction Test Starting...");

    let mut i2c_peri = peri!(p, I2C);
    let mut scl = peri!(p, I2C_SCL);
    let mut sda = peri!(p, I2C_SDA);

    let mut config = Config::default();
    config.frequency = Hertz(100_000);

    // I2C slave address for Analog Discovery or test EEPROM
    let slave_addr = 0x50u8;

    // Wait for slave device to be ready
    Timer::after_millis(100).await;

    // ========== BLOCKING TESTS ==========
    info!("========== BLOCKING TRANSACTION TESTS ==========");
    {
        let mut i2c = I2c::new_blocking(
            i2c_peri.reborrow(),
            scl.reborrow(),
            sda.reborrow(),
            config,
        );

        info!("=== Test 1: Consecutive Writes (Should Merge) ===");
        test_consecutive_writes_blocking(&mut i2c, slave_addr);

        info!("=== Test 2: Consecutive Reads (Should Merge) ===");
        test_consecutive_reads_blocking(&mut i2c, slave_addr);

        info!("=== Test 3: Write then Read (RESTART) ===");
        test_write_then_read_blocking(&mut i2c, slave_addr);

        info!("=== Test 4: Read then Write (RESTART) ===");
        test_read_then_write_blocking(&mut i2c, slave_addr);

        info!("=== Test 5: Complex Mixed Sequence ===");
        test_mixed_sequence_blocking(&mut i2c, slave_addr);

        info!("=== Test 6: Single Operations ===");
        test_single_operations_blocking(&mut i2c, slave_addr);

        info!("Blocking tests OK");
    }

    Timer::after_millis(100).await;

    // ========== ASYNC TESTS ==========
    info!("========== ASYNC TRANSACTION TESTS (DMA) ==========");
    {
        let tx_dma = peri!(p, I2C_TX_DMA);
        let rx_dma = peri!(p, I2C_RX_DMA);
        let irq = irqs!(I2C);

        let mut i2c = I2c::new(i2c_peri, scl, sda, irq, tx_dma, rx_dma, config);

        info!("=== Test 1: Consecutive Writes (Should Merge) ===");
        test_consecutive_writes_async(&mut i2c, slave_addr).await;

        info!("=== Test 2: Consecutive Reads (Should Merge) ===");
        test_consecutive_reads_async(&mut i2c, slave_addr).await;

        info!("=== Test 3: Write then Read (RESTART) ===");
        test_write_then_read_async(&mut i2c, slave_addr).await;

        info!("=== Test 4: Read then Write (RESTART) ===");
        test_read_then_write_async(&mut i2c, slave_addr).await;

        info!("=== Test 5: Complex Mixed Sequence ===");
        test_mixed_sequence_async(&mut i2c, slave_addr).await;

        info!("=== Test 6: Single Operations ===");
        test_single_operations_async(&mut i2c, slave_addr).await;

        info!("Async tests OK");
    }

    info!("All tests OK");
    cortex_m::asm::bkpt();
}

fn test_consecutive_writes_blocking(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) {
    // Expected on bus: START, ADDR+W, data1, data2, data3, STOP
    // NO intermediate RESTART/STOP between writes - they should be merged
    let data1 = [0x10, 0x11, 0x12];
    let data2 = [0x20, 0x21];
    let data3 = [0x30, 0x31, 0x32, 0x33];

    let mut ops = [
        Operation::Write(&data1),
        Operation::Write(&data2),
        Operation::Write(&data3),
    ];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => info!("✓ Consecutive writes succeeded (merged 9 bytes)"),
        Err(e) => {
            error!("✗ Consecutive writes failed: {:?}", e);
            defmt::panic!("Test failed: consecutive writes");
        }
    }
}

fn test_consecutive_reads_blocking(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) {
    // Expected on bus: START, ADDR+R, data1, data2, data3, NACK, STOP
    // NO intermediate RESTART/STOP between reads - they should be merged
    let mut buf1 = [0u8; 4];
    let mut buf2 = [0u8; 3];
    let mut buf3 = [0u8; 2];

    let mut ops = [
        Operation::Read(&mut buf1),
        Operation::Read(&mut buf2),
        Operation::Read(&mut buf3),
    ];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => {
            info!("✓ Consecutive reads succeeded (merged 9 bytes)");
            info!("  buf1: {:02x}", buf1);
            info!("  buf2: {:02x}", buf2);
            info!("  buf3: {:02x}", buf3);
        }
        Err(e) => {
            error!("✗ Consecutive reads failed: {:?}", e);
            defmt::panic!("Test failed: consecutive reads");
        }
    }
}

fn test_write_then_read_blocking(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) {
    // Expected: START, ADDR+W, data, RESTART, ADDR+R, data, NACK, STOP
    let write_data = [0xAA, 0xBB];
    let mut read_buf = [0u8; 4];

    let mut ops = [Operation::Write(&write_data), Operation::Read(&mut read_buf)];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => {
            info!("✓ Write-then-read succeeded with RESTART");
            info!("  Written: {:02x}", write_data);
            info!("  Read: {:02x}", read_buf);
        }
        Err(e) => {
            error!("✗ Write-then-read failed: {:?}", e);
            defmt::panic!("Test failed: write-then-read");
        }
    }
}

fn test_read_then_write_blocking(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) {
    // Expected: START, ADDR+R, data, NACK, RESTART, ADDR+W, data, STOP
    let mut read_buf = [0u8; 3];
    let write_data = [0xCC, 0xDD, 0xEE];

    let mut ops = [Operation::Read(&mut read_buf), Operation::Write(&write_data)];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => {
            info!("✓ Read-then-write succeeded with RESTART");
            info!("  Read: {:02x}", read_buf);
            info!("  Written: {:02x}", write_data);
        }
        Err(e) => {
            error!("✗ Read-then-write failed: {:?}", e);
            defmt::panic!("Test failed: read-then-write");
        }
    }
}

fn test_mixed_sequence_blocking(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) {
    // Complex: W, W, R, R, W, R
    // Groups: [W,W] RESTART [R,R] RESTART [W] RESTART [R]
    let w1 = [0x01, 0x02];
    let w2 = [0x03, 0x04];
    let mut r1 = [0u8; 2];
    let mut r2 = [0u8; 2];
    let w3 = [0x05];
    let mut r3 = [0u8; 1];

    let mut ops = [
        Operation::Write(&w1),
        Operation::Write(&w2),
        Operation::Read(&mut r1),
        Operation::Read(&mut r2),
        Operation::Write(&w3),
        Operation::Read(&mut r3),
    ];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => {
            info!("✓ Mixed sequence succeeded");
            info!("  Groups: [W4] RESTART [R4] RESTART [W1] RESTART [R1]");
        }
        Err(e) => {
            error!("✗ Mixed sequence failed: {:?}", e);
            defmt::panic!("Test failed: mixed sequence");
        }
    }
}

fn test_single_operations_blocking(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) {
    // Test single write
    let write_data = [0xFF];
    let mut ops = [Operation::Write(&write_data)];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => info!("✓ Single write succeeded"),
        Err(e) => {
            error!("✗ Single write failed: {:?}", e);
            defmt::panic!("Test failed: single write");
        }
    }

    // Test single read
    let mut read_buf = [0u8; 1];
    let mut ops = [Operation::Read(&mut read_buf)];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => info!("✓ Single read succeeded, data: 0x{:02x}", read_buf[0]),
        Err(e) => {
            error!("✗ Single read failed: {:?}", e);
            defmt::panic!("Test failed: single read");
        }
    }
}

// ==================== ASYNC TEST FUNCTIONS ====================

async fn test_consecutive_writes_async(i2c: &mut I2c<'_, Async, Master>, addr: u8) {
    let data1 = [0x10, 0x11, 0x12];
    let data2 = [0x20, 0x21];
    let data3 = [0x30, 0x31, 0x32, 0x33];

    let mut ops = [
        Operation::Write(&data1),
        Operation::Write(&data2),
        Operation::Write(&data3),
    ];

    match i2c.transaction(addr, &mut ops).await {
        Ok(_) => info!("✓ Consecutive writes succeeded (merged 9 bytes)"),
        Err(e) => {
            error!("✗ Consecutive writes failed: {:?}", e);
            defmt::panic!("Test failed: consecutive writes");
        }
    }
}

async fn test_consecutive_reads_async(i2c: &mut I2c<'_, Async, Master>, addr: u8) {
    let mut buf1 = [0u8; 4];
    let mut buf2 = [0u8; 3];
    let mut buf3 = [0u8; 2];

    let mut ops = [
        Operation::Read(&mut buf1),
        Operation::Read(&mut buf2),
        Operation::Read(&mut buf3),
    ];

    match i2c.transaction(addr, &mut ops).await {
        Ok(_) => {
            info!("✓ Consecutive reads succeeded (merged 9 bytes)");
            info!("  buf1: {:02x}", buf1);
            info!("  buf2: {:02x}", buf2);
            info!("  buf3: {:02x}", buf3);
        }
        Err(e) => {
            error!("✗ Consecutive reads failed: {:?}", e);
            defmt::panic!("Test failed: consecutive reads");
        }
    }
}

async fn test_write_then_read_async(i2c: &mut I2c<'_, Async, Master>, addr: u8) {
    let write_data = [0xAA, 0xBB];
    let mut read_buf = [0u8; 4];

    let mut ops = [Operation::Write(&write_data), Operation::Read(&mut read_buf)];

    match i2c.transaction(addr, &mut ops).await {
        Ok(_) => {
            info!("✓ Write-then-read succeeded with RESTART");
            info!("  Written: {:02x}", write_data);
            info!("  Read: {:02x}", read_buf);
        }
        Err(e) => {
            error!("✗ Write-then-read failed: {:?}", e);
            defmt::panic!("Test failed: write-then-read");
        }
    }
}

async fn test_read_then_write_async(i2c: &mut I2c<'_, Async, Master>, addr: u8) {
    let mut read_buf = [0u8; 3];
    let write_data = [0xCC, 0xDD, 0xEE];

    let mut ops = [Operation::Read(&mut read_buf), Operation::Write(&write_data)];

    match i2c.transaction(addr, &mut ops).await {
        Ok(_) => {
            info!("✓ Read-then-write succeeded with RESTART");
            info!("  Read: {:02x}", read_buf);
            info!("  Written: {:02x}", write_data);
        }
        Err(e) => {
            error!("✗ Read-then-write failed: {:?}", e);
            defmt::panic!("Test failed: read-then-write");
        }
    }
}

async fn test_mixed_sequence_async(i2c: &mut I2c<'_, Async, Master>, addr: u8) {
    let w1 = [0x01, 0x02];
    let w2 = [0x03, 0x04];
    let mut r1 = [0u8; 2];
    let mut r2 = [0u8; 2];
    let w3 = [0x05];
    let mut r3 = [0u8; 1];

    let mut ops = [
        Operation::Write(&w1),
        Operation::Write(&w2),
        Operation::Read(&mut r1),
        Operation::Read(&mut r2),
        Operation::Write(&w3),
        Operation::Read(&mut r3),
    ];

    match i2c.transaction(addr, &mut ops).await {
        Ok(_) => {
            info!("✓ Mixed sequence succeeded");
            info!("  Groups: [W4] RESTART [R4] RESTART [W1] RESTART [R1]");
        }
        Err(e) => {
            error!("✗ Mixed sequence failed: {:?}", e);
            defmt::panic!("Test failed: mixed sequence");
        }
    }
}

async fn test_single_operations_async(i2c: &mut I2c<'_, Async, Master>, addr: u8) {
    // Test single write
    let write_data = [0xFF];
    let mut ops = [Operation::Write(&write_data)];

    match i2c.transaction(addr, &mut ops).await {
        Ok(_) => info!("✓ Single write succeeded"),
        Err(e) => {
            error!("✗ Single write failed: {:?}", e);
            defmt::panic!("Test failed: single write");
        }
    }

    // Test single read
    let mut read_buf = [0u8; 1];
    let mut ops = [Operation::Read(&mut read_buf)];

    match i2c.transaction(addr, &mut ops).await {
        Ok(_) => info!("✓ Single read succeeded, data: 0x{:02x}", read_buf[0]),
        Err(e) => {
            error!("✗ Single read failed: {:?}", e);
            defmt::panic!("Test failed: single read");
        }
    }
}
