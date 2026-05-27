#![no_std]
#![no_main]

// Hardware Setup for NUCLEO-F072RB:
// - I2C1 pins: PB8 (SCL), PB9 (SDA) on CN5 connector
// - Connect to I2C slave device (e.g., Digilent Analog Discovery I2C slave, or EEPROM)
// - Default slave address: 0x50
// - Pull-up resistors: 4.7kΩ on both SCL and SDA
// - CN5 Pin 10 (PB8/SCL) and CN5 Pin 9 (PB9/SDA)
//
// Analog Discovery - Waveforms Setup:
// - Increase buffer size: Settings -> Device Manager -> Option 4
// - Run Protocol Analyzer
// - Configure as I2C Slave at address 0x50
// - Connect and configure DIO pins for SCL and SDA
// - Frequency: 100kHz - [✓] Clock Stretching

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Config, I2c, Master};
use embassy_stm32::mode::{Async, Blocking};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, dma, i2c, peripherals};
use embassy_time::Timer;
use embedded_hal_1::i2c::Operation;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
    DMA1_CH2_3_DMA2_CH1_2 => dma::InterruptHandler<peripherals::DMA1_CH2>, dma::InterruptHandler<peripherals::DMA1_CH3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Run stm32 I2C v2 Master Tests...");

    let mut i2c_peri = p.I2C1;
    let mut scl = p.PB8;
    let mut sda = p.PB9;

    let mut config = Config::default();
    config.frequency = Hertz(100_000);

    // I2C slave address for Analog Discovery or test EEPROM
    let slave_addr = 0x50u8;

    // Wait for slave device to be ready
    Timer::after_millis(100).await;

    // ========== BLOCKING DIRECT API TESTS ==========
    info!("========== BLOCKING DIRECT API TESTS ==========");
    {
        let mut i2c = I2c::new_blocking(i2c_peri.reborrow(), scl.reborrow(), sda.reborrow(), config);

        info!("=== Test 1: Direct blocking_write ===");
        test_blocking_write(&mut i2c, slave_addr);

        info!("=== Test 2: Direct blocking_read ===");
        test_blocking_read(&mut i2c, slave_addr);

        info!("=== Test 3: Direct blocking_write_read ===");
        test_blocking_write_read(&mut i2c, slave_addr);

        info!("=== Test 4: Direct blocking_write_vectored ===");
        test_blocking_write_vectored(&mut i2c, slave_addr);

        info!("=== Test 5: Large buffer (>255 bytes) ===");
        test_blocking_large_buffer(&mut i2c, slave_addr);

        info!("Blocking direct API tests OK");
    }

    Timer::after_millis(100).await;

    // ========== BLOCKING TRANSACTION TESTS ==========
    info!("========== BLOCKING TRANSACTION TESTS ==========");
    {
        let mut i2c = I2c::new_blocking(i2c_peri.reborrow(), scl.reborrow(), sda.reborrow(), config);

        info!("=== Test 6: Consecutive Writes (Should Merge) ===");
        test_consecutive_writes_blocking(&mut i2c, slave_addr);

        info!("=== Test 7: Consecutive Reads (Should Merge) ===");
        test_consecutive_reads_blocking(&mut i2c, slave_addr);

        info!("=== Test 8: Write then Read (RESTART) ===");
        test_write_then_read_blocking(&mut i2c, slave_addr);

        info!("=== Test 9: Read then Write (RESTART) ===");
        test_read_then_write_blocking(&mut i2c, slave_addr);

        info!("=== Test 10: Complex Mixed Sequence ===");
        test_mixed_sequence_blocking(&mut i2c, slave_addr);

        info!("=== Test 11: Single Operations ===");
        test_single_operations_blocking(&mut i2c, slave_addr);

        info!("Blocking transaction tests OK");
    }

    Timer::after_millis(100).await;

    // ========== ASYNC TESTS (DMA) ==========
    info!("========== ASYNC TESTS (DMA) ==========");
    {
        let tx_dma = p.DMA1_CH2;
        let rx_dma = p.DMA1_CH3;

        let mut i2c = I2c::new(i2c_peri, scl, sda, tx_dma, rx_dma, Irqs, config);

        // Direct API tests (reusing same I2C instance)
        info!("=== Direct API Test 1: write() ===");
        test_async_write(&mut i2c, slave_addr).await;

        info!("=== Direct API Test 2: read() ===");
        test_async_read(&mut i2c, slave_addr).await;

        info!("=== Direct API Test 3: write_read() ===");
        test_async_write_read(&mut i2c, slave_addr).await;

        info!("=== Direct API Test 4: write_vectored() ===");
        test_async_write_vectored(&mut i2c, slave_addr).await;

        info!("=== Direct API Test 5: Large buffer (>255 bytes) ===");
        test_async_large_buffer(&mut i2c, slave_addr).await;

        info!("Async Direct API tests OK");

        // Transaction tests
        info!("=== Transaction Test 6: Consecutive Writes (Should Merge) ===");
        test_consecutive_writes_async(&mut i2c, slave_addr).await;

        info!("=== Transaction Test 7: Consecutive Reads (Should Merge) ===");
        test_consecutive_reads_async(&mut i2c, slave_addr).await;

        info!("=== Transaction Test 8: Write then Read (RESTART) ===");
        test_write_then_read_async(&mut i2c, slave_addr).await;

        info!("=== Transaction Test 9: Read then Write (RESTART) ===");
        test_read_then_write_async(&mut i2c, slave_addr).await;

        info!("=== Transaction Test 10: Complex Mixed Sequence ===");
        test_mixed_sequence_async(&mut i2c, slave_addr).await;

        info!("=== Transaction Test 11: Single Operations ===");
        test_single_operations_async(&mut i2c, slave_addr).await;

        info!("Async transaction tests OK");
    }

    info!("All tests OK");
    cortex_m::asm::bkpt();
}

// ==================== BLOCKING DIRECT API TEST FUNCTIONS ====================

fn test_blocking_write(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) {
    let write_data = [0x42, 0x43, 0x44, 0x45];

    match i2c.blocking_write(addr, &write_data) {
        Ok(_) => info!("✓ blocking_write succeeded: {:02x}", write_data),
        Err(e) => {
            error!("✗ blocking_write failed: {:?}", e);
            defmt::panic!("Test failed: blocking_write");
        }
    }
}

fn test_blocking_read(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) {
    let mut read_buf = [0u8; 8];

    match i2c.blocking_read(addr, &mut read_buf) {
        Ok(_) => info!("✓ blocking_read succeeded: {:02x}", read_buf),
        Err(e) => {
            error!("✗ blocking_read failed: {:?}", e);
            defmt::panic!("Test failed: blocking_read");
        }
    }
}

fn test_blocking_write_read(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) {
    let write_data = [0x50, 0x51];
    let mut read_buf = [0u8; 6];

    match i2c.blocking_write_read(addr, &write_data, &mut read_buf) {
        Ok(_) => {
            info!("✓ blocking_write_read succeeded");
            info!("  Written: {:02x}", write_data);
            info!("  Read: {:02x}", read_buf);
        }
        Err(e) => {
            error!("✗ blocking_write_read failed: {:?}", e);
            defmt::panic!("Test failed: blocking_write_read");
        }
    }
}

fn test_blocking_write_vectored(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) {
    let buf1 = [0x60, 0x61, 0x62];
    let buf2 = [0x70, 0x71];
    let buf3 = [0x80, 0x81, 0x82, 0x83];
    let bufs = [&buf1[..], &buf2[..], &buf3[..]];

    match i2c.blocking_write_vectored(addr, &bufs) {
        Ok(_) => info!("✓ blocking_write_vectored succeeded (9 bytes total)"),
        Err(e) => {
            error!("✗ blocking_write_vectored failed: {:?}", e);
            defmt::panic!("Test failed: blocking_write_vectored");
        }
    }
}

fn test_blocking_large_buffer(i2c: &mut I2c<'_, Blocking, Master>, addr: u8) {
    // Test with 300 bytes to verify RELOAD mechanism works (needs chunking at 255 bytes)
    let mut write_buf = [0u8; 300];
    for (i, byte) in write_buf.iter_mut().enumerate() {
        *byte = (i & 0xFF) as u8;
    }

    match i2c.blocking_write(addr, &write_buf) {
        Ok(_) => info!("✓ Large buffer write succeeded (300 bytes, tests RELOAD)"),
        Err(e) => {
            error!("✗ Large buffer write failed: {:?}", e);
            defmt::panic!("Test failed: large buffer write");
        }
    }

    // Test large read
    let mut read_buf = [0u8; 300];
    match i2c.blocking_read(addr, &mut read_buf) {
        Ok(_) => info!("✓ Large buffer read succeeded (300 bytes, tests RELOAD)"),
        Err(e) => {
            error!("✗ Large buffer read failed: {:?}", e);
            defmt::panic!("Test failed: large buffer read");
        }
    }
}

// ==================== BLOCKING TRANSACTION TEST FUNCTIONS ====================

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

// ==================== ASYNC DIRECT API TEST FUNCTIONS ====================

async fn test_async_write(i2c: &mut I2c<'_, Async, Master>, addr: u8) {
    let write_data = [0x42, 0x43, 0x44, 0x45];

    match i2c.write(addr, &write_data).await {
        Ok(_) => info!("✓ async write succeeded: {:02x}", write_data),
        Err(e) => {
            error!("✗ async write failed: {:?}", e);
            defmt::panic!("Test failed: async write");
        }
    }
}

async fn test_async_read(i2c: &mut I2c<'_, Async, Master>, addr: u8) {
    let mut read_buf = [0u8; 8];

    match i2c.read(addr, &mut read_buf).await {
        Ok(_) => info!("✓ async read succeeded: {:02x}", read_buf),
        Err(e) => {
            error!("✗ async read failed: {:?}", e);
            defmt::panic!("Test failed: async read");
        }
    }
}

async fn test_async_write_read(i2c: &mut I2c<'_, Async, Master>, addr: u8) {
    let write_data = [0x50, 0x51];
    let mut read_buf = [0u8; 6];

    match i2c.write_read(addr, &write_data, &mut read_buf).await {
        Ok(_) => {
            info!("✓ async write_read succeeded");
            info!("  Written: {:02x}", write_data);
            info!("  Read: {:02x}", read_buf);
        }
        Err(e) => {
            error!("✗ async write_read failed: {:?}", e);
            defmt::panic!("Test failed: async write_read");
        }
    }
}

async fn test_async_write_vectored(i2c: &mut I2c<'_, Async, Master>, addr: u8) {
    let buf1 = [0x60, 0x61, 0x62];
    let buf2 = [0x70, 0x71];
    let buf3 = [0x80, 0x81, 0x82, 0x83];
    let bufs = [&buf1[..], &buf2[..], &buf3[..]];

    match i2c.write_vectored(addr.into(), &bufs).await {
        Ok(_) => info!("✓ async write_vectored succeeded (9 bytes total)"),
        Err(e) => {
            error!("✗ async write_vectored failed: {:?}", e);
            defmt::panic!("Test failed: async write_vectored");
        }
    }
}

async fn test_async_large_buffer(i2c: &mut I2c<'_, Async, Master>, addr: u8) {
    // Test with 300 bytes to verify RELOAD mechanism works with DMA (needs chunking at 255 bytes)
    let mut write_buf = [0u8; 300];
    for (i, byte) in write_buf.iter_mut().enumerate() {
        *byte = (i & 0xFF) as u8;
    }

    match i2c.write(addr, &write_buf).await {
        Ok(_) => info!("✓ Large buffer async write succeeded (300 bytes, tests RELOAD with DMA)"),
        Err(e) => {
            error!("✗ Large buffer async write failed: {:?}", e);
            defmt::panic!("Test failed: large buffer async write");
        }
    }

    // Test large read
    let mut read_buf = [0u8; 300];
    match i2c.read(addr, &mut read_buf).await {
        Ok(_) => info!("✓ Large buffer async read succeeded (300 bytes, tests RELOAD with DMA)"),
        Err(e) => {
            error!("✗ Large buffer async read failed: {:?}", e);
            defmt::panic!("Test failed: large buffer async read");
        }
    }
}

// ==================== ASYNC TRANSACTION TEST FUNCTIONS ====================

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
