//! I2C Async Loopback Test - Comprehensive slave mode testing
//!
//! Tests the async I2C master and slave implementations using
//! a loopback configuration with I2C1 as slave and I2C2 as master
//! on the same device.
//!
//! Hardware setup (for NUCLEO-F072RB):
//! - Connect PB8 (I2C1 SCL) to PB10 (I2C2 SCL)
//! - Connect PB9 (I2C1 SDA) to PB11 (I2C2 SDA)
//! - Add 4.7k pull-up resistors to 3.3V on both SCL and SDA lines
//!
//! Pin locations on NUCLEO-F072RB:
//! - PB8:  CN5 pin 10 (D15)
//! - PB9:  CN5 pin 9  (D14)
//! - PB10: CN10 pin 25
//! - PB11: CN10 pin 18
//!
//! # Test Coverage
//!
//! - W1: Basic Write Operations
//! - W2: Early STOP (master sends less than buffer)
//! - W3: Excess Data (master sends more than buffer) - SKIPPED (see below)
//! - R1: Basic Read Operations
//! - R2: Short Read (master reads less than slave offers)
//! - M1: Mixed Operations
//! - S1: Stress Tests
//! - WR1: Write-Read with RESTART (varying write size)
//! - WR2: Write-Read with RESTART (varying read size)
//! - WR3: Multiple Write-Read Operations
//!
//! # Skipped Tests
//!
//! Some tests are skipped in loopback mode due to executor limitations:
//!
//! - **W2.4 (Empty write)**: The master's empty write uses a blocking path,
//!   preventing the executor from running the slave task to release clock stretching.
//!
//! - **W3.x (Excess data)**: After the slave's DMA buffer is full, it busy-waits
//!   in `drain_rxdr_until_stop()` for the STOP condition, but the master can't
//!   send STOP because its poll_fn needs executor time.
//!
//! These tests pass with an external master (e.g., Analog Discovery) where master
//! and slave run on separate processors.

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
const SLAVE_BUFFER_SIZE: usize = 32;
const EXPECTED_READ: [u8; 8] = [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7];

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C2 => i2c::EventInterruptHandler<peripherals::I2C2>, i2c::ErrorInterruptHandler<peripherals::I2C2>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>, dma::InterruptHandler<peripherals::DMA1_CH3>;
    DMA1_CHANNEL4_5_6_7 => dma::InterruptHandler<peripherals::DMA1_CH4>, dma::InterruptHandler<peripherals::DMA1_CH5>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("==============================================");
    info!("I2C Async Loopback Test - Full Suite");
    info!("I2C1 (slave) PB8/PB9, I2C2 (master) PB10/PB11");
    info!("==============================================");

    let i2c_config = {
        let mut config = i2c::Config::default();
        config.frequency = khz(100);
        config
    };

    let i2c_slave = I2c::new(p.I2C1, p.PB8, p.PB9, p.DMA1_CH2, p.DMA1_CH3, Irqs, i2c_config)
        .into_slave_multimaster(SlaveAddrConfig::basic(I2C_ADDR));

    let i2c_master = I2c::new(p.I2C2, p.PB10, p.PB11, p.DMA1_CH4, p.DMA1_CH5, Irqs, i2c_config);

    join(slave_task(i2c_slave), master_task(i2c_master)).await;
}

async fn slave_task(mut i2c: I2c<'static, embassy_stm32::mode::Async, i2c::mode::MultiMaster>) {
    info!(
        "[Slave] Ready at 0x{:02X}, buffer {} bytes",
        I2C_ADDR, SLAVE_BUFFER_SIZE
    );

    loop {
        match i2c.listen().await {
            Ok(command) => match command.kind {
                SlaveCommandKind::Write => {
                    let mut buffer = [0u8; SLAVE_BUFFER_SIZE];
                    match i2c.respond_to_write(&mut buffer).await {
                        Ok(n) => info!("[Slave] RX {} bytes", n),
                        Err(e) => error!("[Slave] RX error: {:?}", e),
                    }
                }
                SlaveCommandKind::Read => match i2c.respond_to_read(&EXPECTED_READ).await {
                    Ok(_) => info!("[Slave] TX complete"),
                    Err(e) => error!("[Slave] TX error: {:?}", e),
                },
            },
            Err(e) => {
                warn!("[Slave] Listen error: {:?}", e);
                Timer::after_millis(100).await;
            }
        }
    }
}

async fn master_task(mut i2c: I2c<'static, embassy_stm32::mode::Async, i2c::mode::Master>) {
    Timer::after_millis(100).await;

    info!("");
    info!("Starting comprehensive test suite...");
    info!("");

    let mut passed = 0u32;
    let mut failed = 0u32;
    let skipped = 4u32; // W2.4 + W3.1-3 (blocking paths incompatible with loopback)

    // Helper macro for write tests
    macro_rules! test_write {
        ($id:expr, $desc:expr, $data:expr) => {{
            info!("--- {} ---", $id);
            match i2c.write(I2C_ADDR, $data).await {
                Ok(()) => {
                    info!("[PASS] {}: {}", $id, $desc);
                    passed += 1;
                }
                Err(e) => {
                    error!("[FAIL] {}: {:?}", $id, e);
                    failed += 1;
                }
            }
            Timer::after_millis(50).await;
        }};
    }

    // Helper macro for read tests with validation
    macro_rules! test_read {
        ($id:expr, $desc:expr, $len:expr) => {{
            info!("--- {} ---", $id);
            let mut buf = [0u8; $len];
            match i2c.read(I2C_ADDR, &mut buf).await {
                Ok(()) => {
                    if buf == EXPECTED_READ[..$len] {
                        info!("[PASS] {}: {}", $id, $desc);
                        passed += 1;
                    } else {
                        error!(
                            "[FAIL] {}: got {:02X}, expected {:02X}",
                            $id,
                            buf,
                            &EXPECTED_READ[..$len]
                        );
                        failed += 1;
                    }
                }
                Err(e) => {
                    error!("[FAIL] {}: {:?}", $id, e);
                    failed += 1;
                }
            }
            Timer::after_millis(50).await;
        }};
    }

    // Helper macro for write_read tests with validation
    macro_rules! test_write_read {
        ($id:expr, $desc:expr, $write:expr, $read_len:expr) => {{
            info!("--- {} ---", $id);
            let mut buf = [0u8; $read_len];
            match i2c.write_read(I2C_ADDR, $write, &mut buf).await {
                Ok(()) => {
                    if buf == EXPECTED_READ[..$read_len] {
                        info!("[PASS] {}: {}", $id, $desc);
                        passed += 1;
                    } else {
                        error!(
                            "[FAIL] {}: got {:02X}, expected {:02X}",
                            $id,
                            buf,
                            &EXPECTED_READ[..$read_len]
                        );
                        failed += 1;
                    }
                }
                Err(e) => {
                    error!("[FAIL] {}: {:?}", $id, e);
                    failed += 1;
                }
            }
            Timer::after_millis(50).await;
        }};
    }

    // =========================================================================
    // W1: Basic Write Operations
    // =========================================================================
    info!("========== W1: Basic Write Operations ==========");
    test_write!("W1.1", "Single byte write", &[0x11]);
    test_write!("W1.2", "4 bytes write", &[0x11, 0x22, 0x33, 0x44]);
    test_write!(
        "W1.3",
        "8 bytes write",
        &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
    );
    test_write!(
        "W1.4",
        "32 bytes write",
        &[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11,
            0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
        ]
    );

    // =========================================================================
    // W2: Early STOP (master sends less than buffer)
    // =========================================================================
    info!("========== W2: Early STOP ==========");
    test_write!("W2.1", "2 of 32 bytes", &[0xAA, 0xBB]);
    test_write!(
        "W2.2",
        "10 of 32 bytes",
        &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A]
    );
    test_write!(
        "W2.3",
        "31 of 32 bytes",
        &[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11,
            0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        ]
    );
    // W2.4 SKIPPED: Empty write uses blocking path, causes executor deadlock in loopback
    info!("[SKIP] W2.4: Empty write - blocking path incompatible with loopback");

    // =========================================================================
    // W3: Excess Data (master sends more than buffer)
    // SKIPPED: drain_rxdr_until_stop() busy-waits, blocking master from sending STOP
    // =========================================================================
    info!("========== W3: Excess Data (SKIPPED) ==========");
    info!("[SKIP] W3.1-3: Excess data tests require external master");

    // =========================================================================
    // R1: Basic Read Operations
    // =========================================================================
    info!("========== R1: Basic Read Operations ==========");
    test_read!("R1.1", "Read 1 byte", 1);
    test_read!("R1.2", "Read 4 bytes", 4);
    test_read!("R1.3", "Read 8 bytes", 8);

    // =========================================================================
    // R2: Short Read (master reads less than slave offers)
    // =========================================================================
    info!("========== R2: Short Read ==========");
    test_read!("R2.1", "Read 2 of 8 bytes", 2);
    test_read!("R2.2", "Read 5 of 8 bytes", 5);
    test_read!("R2.3", "Read 7 of 8 bytes", 7);

    // =========================================================================
    // M1: Mixed Operations
    // =========================================================================
    info!("========== M1: Mixed Operations ==========");

    // M1.1: Write then Read
    info!("--- M1.1 ---");
    {
        let mut ok = true;
        if i2c.write(I2C_ADDR, &[0x11, 0x22, 0x33]).await.is_err() {
            ok = false;
        }
        Timer::after_millis(50).await;
        let mut buf = [0u8; 4];
        if i2c.read(I2C_ADDR, &mut buf).await.is_err() || buf != EXPECTED_READ[..4] {
            ok = false;
        }
        if ok {
            info!("[PASS] M1.1: Write then Read");
            passed += 1;
        } else {
            error!("[FAIL] M1.1: Write then Read");
            failed += 1;
        }
    }
    Timer::after_millis(50).await;

    // M1.2: Read then Write
    info!("--- M1.2 ---");
    {
        let mut ok = true;
        let mut buf = [0u8; 4];
        if i2c.read(I2C_ADDR, &mut buf).await.is_err() || buf != EXPECTED_READ[..4] {
            ok = false;
        }
        Timer::after_millis(50).await;
        if i2c.write(I2C_ADDR, &[0xAA, 0xBB]).await.is_err() {
            ok = false;
        }
        if ok {
            info!("[PASS] M1.2: Read then Write");
            passed += 1;
        } else {
            error!("[FAIL] M1.2: Read then Write");
            failed += 1;
        }
    }
    Timer::after_millis(50).await;

    // M1.3: Multiple writes
    info!("--- M1.3 ---");
    {
        let mut ok = true;
        for data in [&[0x11][..], &[0x22, 0x33], &[0x44, 0x55, 0x66]] {
            if i2c.write(I2C_ADDR, data).await.is_err() {
                ok = false;
            }
            Timer::after_millis(30).await;
        }
        if ok {
            info!("[PASS] M1.3: Multiple writes");
            passed += 1;
        } else {
            error!("[FAIL] M1.3: Multiple writes");
            failed += 1;
        }
    }
    Timer::after_millis(50).await;

    // M1.4: Multiple reads
    info!("--- M1.4 ---");
    {
        let mut ok = true;
        for len in [2usize, 4, 8] {
            let mut buf = [0u8; 8];
            if i2c.read(I2C_ADDR, &mut buf[..len]).await.is_err() || buf[..len] != EXPECTED_READ[..len] {
                ok = false;
            }
            Timer::after_millis(30).await;
        }
        if ok {
            info!("[PASS] M1.4: Multiple reads");
            passed += 1;
        } else {
            error!("[FAIL] M1.4: Multiple reads");
            failed += 1;
        }
    }
    Timer::after_millis(50).await;

    // =========================================================================
    // S1: Stress Tests
    // =========================================================================
    info!("========== S1: Stress Tests ==========");

    // S1.1: 10x write same data
    info!("--- S1.1 ---");
    {
        let mut successes = 0;
        for _ in 0..10 {
            if i2c.write(I2C_ADDR, &[0x11, 0x22, 0x33, 0x44]).await.is_ok() {
                successes += 1;
            }
            Timer::after_millis(20).await;
        }
        if successes == 10 {
            info!("[PASS] S1.1: 10x write");
            passed += 1;
        } else {
            error!("[FAIL] S1.1: {}/10 writes", successes);
            failed += 1;
        }
    }
    Timer::after_millis(50).await;

    // S1.2: 10x read
    info!("--- S1.2 ---");
    {
        let mut successes = 0;
        for _ in 0..10 {
            let mut buf = [0u8; 4];
            if i2c.read(I2C_ADDR, &mut buf).await.is_ok() && buf == EXPECTED_READ[..4] {
                successes += 1;
            }
            Timer::after_millis(20).await;
        }
        if successes == 10 {
            info!("[PASS] S1.2: 10x read");
            passed += 1;
        } else {
            error!("[FAIL] S1.2: {}/10 reads", successes);
            failed += 1;
        }
    }
    Timer::after_millis(50).await;

    // S1.3: Alternating W/R x10
    info!("--- S1.3 ---");
    {
        let mut successes = 0;
        for i in 0..10u8 {
            if i2c.write(I2C_ADDR, &[i]).await.is_ok() {
                Timer::after_millis(20).await;
                let mut buf = [0u8; 4];
                if i2c.read(I2C_ADDR, &mut buf).await.is_ok() && buf == EXPECTED_READ[..4] {
                    successes += 1;
                }
            }
            Timer::after_millis(20).await;
        }
        if successes == 10 {
            info!("[PASS] S1.3: 10x alternating W/R");
            passed += 1;
        } else {
            error!("[FAIL] S1.3: {}/10 pairs", successes);
            failed += 1;
        }
    }
    Timer::after_millis(50).await;

    // =========================================================================
    // WR1: Write-Read with RESTART (varying write size)
    // =========================================================================
    info!("========== WR1: Write-Read RESTART ==========");
    test_write_read!("WR1.1", "Write 1, Read 4", &[0x51], 4);
    test_write_read!("WR1.2", "Write 2, Read 4", &[0x61, 0x62], 4);
    test_write_read!("WR1.3", "Write 4, Read 4", &[0x71, 0x72, 0x73, 0x74], 4);
    test_write_read!(
        "WR1.4",
        "Write 8, Read 8",
        &[0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88],
        8
    );

    // =========================================================================
    // WR2: Write-Read with RESTART (varying read size)
    // =========================================================================
    info!("========== WR2: Write-Read varying read ==========");
    test_write_read!("WR2.1", "Write 1, Read 1", &[0x91], 1);
    test_write_read!("WR2.2", "Write 1, Read 8", &[0xA1], 8);
    test_write_read!("WR2.3", "Write 1, Read 2", &[0xB1], 2);

    // =========================================================================
    // WR3: Multiple Write-Read Operations
    // =========================================================================
    info!("========== WR3: Multiple Write-Read ==========");

    // WR3.1: 5x consecutive write_read
    info!("--- WR3.1 ---");
    {
        let mut successes = 0;
        for i in 0..5u8 {
            let mut buf = [0u8; 4];
            if i2c.write_read(I2C_ADDR, &[0x10 + i], &mut buf).await.is_ok() && buf == EXPECTED_READ[..4] {
                successes += 1;
            }
            Timer::after_millis(30).await;
        }
        if successes == 5 {
            info!("[PASS] WR3.1: 5x write_read");
            passed += 1;
        } else {
            error!("[FAIL] WR3.1: {}/5", successes);
            failed += 1;
        }
    }
    Timer::after_millis(50).await;

    // WR3.2: 10x consecutive write_read
    info!("--- WR3.2 ---");
    {
        let mut successes = 0;
        for i in 0..10u8 {
            let mut buf = [0u8; 4];
            if i2c.write_read(I2C_ADDR, &[0x20 + i], &mut buf).await.is_ok() && buf == EXPECTED_READ[..4] {
                successes += 1;
            }
            Timer::after_millis(20).await;
        }
        if successes == 10 {
            info!("[PASS] WR3.2: 10x write_read");
            passed += 1;
        } else {
            error!("[FAIL] WR3.2: {}/10", successes);
            failed += 1;
        }
    }
    Timer::after_millis(50).await;

    // WR3.3: Alternating write_read and separate write+read
    info!("--- WR3.3 ---");
    {
        let mut ok = true;

        // write_read (RESTART)
        let mut buf = [0u8; 4];
        if i2c.write_read(I2C_ADDR, &[0x30], &mut buf).await.is_err() || buf != EXPECTED_READ[..4] {
            ok = false;
        }
        Timer::after_millis(30).await;

        // Separate write + read (STOP between)
        if i2c.write(I2C_ADDR, &[0x31, 0x32]).await.is_err() {
            ok = false;
        }
        Timer::after_millis(30).await;
        let mut buf = [0u8; 4];
        if i2c.read(I2C_ADDR, &mut buf).await.is_err() || buf != EXPECTED_READ[..4] {
            ok = false;
        }
        Timer::after_millis(30).await;

        // write_read again
        let mut buf = [0u8; 4];
        if i2c.write_read(I2C_ADDR, &[0x33], &mut buf).await.is_err() || buf != EXPECTED_READ[..4] {
            ok = false;
        }

        if ok {
            info!("[PASS] WR3.3: Mixed RESTART/STOP");
            passed += 1;
        } else {
            error!("[FAIL] WR3.3: Mixed RESTART/STOP");
            failed += 1;
        }
    }
    Timer::after_millis(50).await;

    // =========================================================================
    // TEST SUMMARY
    // =========================================================================
    info!("");
    info!("==============================================");
    info!("TOTAL: {} passed, {} failed, {} skipped", passed, failed, skipped);
    if failed == 0 {
        info!("ALL TESTS PASSED!");
    } else {
        error!("{} TESTS FAILED", failed);
    }
    info!("==============================================");

    loop {
        Timer::after_millis(1000).await;
    }
}
