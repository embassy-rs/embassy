#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Config, I2c, Master};
use embassy_stm32::mode::Blocking;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_time::Timer;
use embedded_hal_1::i2c::Operation;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // For STM32F072RB on NUCLEO board
    let p = embassy_stm32::init(Default::default());

    info!("I2C Transaction Test Starting...");

    // Initialize I2C1: PB6=SCL, PB7=SDA
    let mut config = Config::default();
    config.frequency = Hertz(100_000);
    let mut i2c = I2c::new_blocking(
        p.I2C1,
        p.PB8,  // SCL
        p.PB9,  // SDA
        config,
    );

    let slave_addr = 0x50u8;

    // Wait for devices to initialize
    Timer::after_millis(100).await;

    info!("=== Test 1: Consecutive Writes (Should Merge) ===");
    test_consecutive_writes(&mut i2c, slave_addr);
    Timer::after_millis(500).await;

    info!("=== Test 2: Consecutive Reads (Should Merge) ===");
    test_consecutive_reads(&mut i2c, slave_addr);
    Timer::after_millis(500).await;

    info!("=== Test 3: Write then Read (RESTART) ===");
    test_write_then_read(&mut i2c, slave_addr);
    Timer::after_millis(500).await;

    info!("=== Test 4: Read then Write (RESTART) ===");
    test_read_then_write(&mut i2c, slave_addr);
    Timer::after_millis(500).await;

    info!("=== Test 5: Complex Mixed Sequence ===");
    test_mixed_sequence(&mut i2c, slave_addr);
    Timer::after_millis(500).await;

    info!("=== Test 6: Single Operations ===");
    test_single_operations(&mut i2c, slave_addr);

    info!("All tests complete!");

    loop {
        Timer::after_secs(1).await;
    }
}

fn test_consecutive_writes(i2c: &mut I2c<'static, Blocking, Master>, addr: u8) {
    // Expected on bus: START, ADDR+W, data1, data2, data3, STOP
    // NO intermediate RESTART/STOP between writes
    let data1 = [0x10, 0x11, 0x12];
    let data2 = [0x20, 0x21];
    let data3 = [0x30, 0x31, 0x32, 0x33];

    let mut ops = [
        Operation::Write(&data1),
        Operation::Write(&data2),
        Operation::Write(&data3),
    ];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => info!("✓ Consecutive writes succeeded"),
        Err(e) => warn!("✗ Consecutive writes failed: {:?}", e),
    }

    info!("Expected: START, ADDR+W, [9 bytes], STOP");
    info!("Check Analog Discovery: No RESTART between writes");
}

fn test_consecutive_reads(i2c: &mut I2c<'static, Blocking, Master>, addr: u8) {
    // Expected on bus: START, ADDR+R, data1, data2, data3, NACK, STOP
    // NO intermediate RESTART/STOP between reads
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
            info!("✓ Consecutive reads succeeded");
            info!("  buf1: {:02x}", buf1);
            info!("  buf2: {:02x}", buf2);
            info!("  buf3: {:02x}", buf3);
        }
        Err(e) => warn!("✗ Consecutive reads failed: {:?}", e),
    }

    info!("Expected: START, ADDR+R, [9 bytes], NACK on last, STOP");
    info!("Check Analog Discovery: No RESTART between reads");
}

fn test_write_then_read(i2c: &mut I2c<'static, Blocking, Master>, addr: u8) {
    // Expected: START, ADDR+W, data, RESTART, ADDR+R, data, NACK, STOP
    let write_data = [0xAA, 0xBB];
    let mut read_buf = [0u8; 4];

    let mut ops = [
        Operation::Write(&write_data),
        Operation::Read(&mut read_buf),
    ];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => {
            info!("✓ Write-then-read succeeded");
            info!("  Written: {:02x}", write_data);
            info!("  Read: {:02x}", read_buf);
        }
        Err(e) => warn!("✗ Write-then-read failed: {:?}", e),
    }

    info!("Expected: START, ADDR+W, [2 bytes], RESTART, ADDR+R, [4 bytes], NACK, STOP");
    info!("Check Analog Discovery: RESTART between write and read");
}

fn test_read_then_write(i2c: &mut I2c<'static, Blocking, Master>, addr: u8) {
    // Expected: START, ADDR+R, data, NACK, RESTART, ADDR+W, data, STOP
    let mut read_buf = [0u8; 3];
    let write_data = [0xCC, 0xDD, 0xEE];

    let mut ops = [
        Operation::Read(&mut read_buf),
        Operation::Write(&write_data),
    ];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => {
            info!("✓ Read-then-write succeeded");
            info!("  Read: {:02x}", read_buf);
            info!("  Written: {:02x}", write_data);
        }
        Err(e) => warn!("✗ Read-then-write failed: {:?}", e),
    }

    info!("Expected: START, ADDR+R, [3 bytes], NACK, RESTART, ADDR+W, [3 bytes], STOP");
    info!("Check Analog Discovery: RESTART between read and write");
}

fn test_mixed_sequence(i2c: &mut I2c<'static, Blocking, Master>, addr: u8) {
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
            info!("  r1: {:02x}", r1);
            info!("  r2: {:02x}", r2);
            info!("  r3: {:02x}", r3);
        }
        Err(e) => warn!("✗ Mixed sequence failed: {:?}", e),
    }

    info!("Expected sequence:");
    info!("  START, ADDR+W, [4 bytes merged], RESTART,");
    info!("  ADDR+R, [4 bytes merged], NACK, RESTART,");
    info!("  ADDR+W, [1 byte], RESTART,");
    info!("  ADDR+R, [1 byte], NACK, STOP");
}

fn test_single_operations(i2c: &mut I2c<'static, Blocking, Master>, addr: u8) {
    // Test single write
    let write_data = [0xFF];
    let mut ops = [Operation::Write(&write_data)];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => info!("✓ Single write succeeded"),
        Err(e) => warn!("✗ Single write failed: {:?}", e),
    }

    // Test single read
    let mut read_buf = [0u8; 1];
    let mut ops = [Operation::Read(&mut read_buf)];

    match i2c.blocking_transaction(addr, &mut ops) {
        Ok(_) => info!("✓ Single read succeeded, data: 0x{:02x}", read_buf[0]),
        Err(e) => warn!("✗ Single read failed: {:?}", e),
    }
}
