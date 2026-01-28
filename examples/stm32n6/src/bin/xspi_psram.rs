#![no_std]
#![no_main]

//! XSPI PSRAM Example for STM32N6570-DK
//!
//! This example demonstrates how to use the XSPI peripheral to communicate with
//! the external APS256XX PSRAM (32 MB) on the STM32N6570-DK board.
//!
//! The PSRAM is connected via XSPI1 using Hexadeca-SPI interface (16-bit data width):
//! - Memory-mapped address: 0x90000000
//! - Size: 256 Mbit (32 MiB)
//! - Interface: Hexadeca-SPI (16-bit data width, DTR mode)
//!
//! Pin mapping (XSPIM Port 1):
//! - PO4: CLK
//! - PO0: NCS1
//! - PO2: DQS0
//! - PO3: DQS1
//! - PP0-PP15: D0-D15
//!
//! Note: Unlike NOR flash, PSRAM supports both read AND write directly via
//! memory-mapped mode without needing indirect mode for data transfers.
//!
//! IMPORTANT: Sub-word (byte) writes to PSRAM don't work correctly in memory-mapped
//! mode. The exact cause is not yet fully researched (maybe AXI bus or XSPI
//! peripheral behavior). Use the `psram_write_byte()` helper which performs a
//! 64-bit read-modify-write as a workaround.
//!
//! NOTE: Some test failures are expected in DEBUG 2/2b/2c tests - these demonstrate
//! the sub-word write issue. The 64-bit R-M-W workaround tests (DEBUG 6, FINAL TEST)
//! should pass.

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::rcc::{CpuClk, IcConfig, Icint, Icsel, Pll, Plldivm, Pllpdiv, Pllsel, SysClk, XspiClkSrc};
use embassy_stm32::xspi::{
    AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, Instance, MemorySize, MemoryType, TransferConfig,
    WrapSize, Xspi, XspiWidth,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// APS256XX Commands
#[allow(dead_code)]
mod aps256_cmd {
    pub const READ: u32 = 0x00; // Synchronous Read (used by C HAL for memory-mapped)
    pub const READ_LINEAR_BURST: u32 = 0x20; // Linear Burst Read
    pub const WRITE: u32 = 0x80; // Synchronous Write (used by C HAL for memory-mapped)
    pub const WRITE_LINEAR_BURST: u32 = 0xA0; // Linear Burst Write
    pub const READ_REG: u32 = 0x40;
    pub const WRITE_REG: u32 = 0xC0;
    pub const RESET: u32 = 0xFF;
}

/// APS256XX Mode Register Addresses
mod aps256_mr {
    pub const MR0: u32 = 0x00000000;
    pub const MR4: u32 = 0x00000004;
    pub const MR8: u32 = 0x00000008;
}

/// Memory-mapped base address for XSPI1
const MM_BASE: u32 = 0x90000000;

/// Write a single byte to PSRAM via 64-bit read-modify-write.
///
/// Workaround for sub-word write corruption in memory-mapped mode. The exact cause
/// is not yet fully researched - possibly related to AXI bus behavior or XSPI
/// peripheral quirks. This helper reads the full 64-bit word, modifies the target
/// byte, and writes back the complete 64-bit value.

#[inline(never)]
unsafe fn psram_write_byte(addr: *mut u8, val: u8) {
    let aligned_addr = (addr as usize & !7) as *mut u64; // 8-byte align
    let byte_offset = addr as usize & 7;

    // SAFETY: caller guarantees addr is valid for read/write in PSRAM memory-mapped region
    unsafe {
        let current = core::ptr::read_volatile(aligned_addr);
        let shift = byte_offset * 8;
        let mask = !(0xFFu64 << shift);
        let new_val = (current & mask) | ((val as u64) << shift);
        core::ptr::write_volatile(aligned_addr, new_val);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Configure RCC with full clock tree for 200MHz XSPI operation
    let mut config = embassy_stm32::Config::default();

    // Configure PLL1: HSI 64MHz / 4 * 75 = 1200MHz
    config.rcc.pll1 = Some(Pll::Oscillator {
        source: Pllsel::HSI,
        divm: Plldivm::from_bits(4), // 64MHz / 4 = 16MHz VCO input
        fractional: 0,
        divn: 75,                     // 16MHz * 75 = 1200MHz VCO
        divp1: Pllpdiv::from_bits(1), // 1200MHz / 1 = 1200MHz output
        divp2: Pllpdiv::from_bits(1),
    });

    // Configure IC1: CPU clock = PLL1 / 2 = 600 MHz
    config.rcc.cpu = CpuClk::Ic1 {
        source: Icsel::PLL1,
        divider: Icint::from_bits(1), // DIV2: (1+1) = 2
    };

    // Configure IC2/IC6/IC11: System bus = PLL1 / 3 = 400 MHz
    config.rcc.sys = SysClk::Ic2 {
        ic2: IcConfig {
            source: Icsel::PLL1,
            divider: Icint::from_bits(2), // DIV3: (2+1) = 3
        },
        ic6: IcConfig {
            source: Icsel::PLL1,
            divider: Icint::from_bits(3), // DIV4: (3+1) = 4 (C HAL uses /4)
        },
        ic11: IcConfig {
            source: Icsel::PLL1,
            divider: Icint::from_bits(3), // DIV4: (3+1) = 4 (C HAL uses /4)
        },
    };

    // Configure IC3: XSPI1 kernel clock = PLL1 / 6 = 200 MHz
    config.rcc.ic3 = Some(IcConfig {
        source: Icsel::PLL1,
        divider: Icint::from_bits(5), // DIV6: (5+1) = 6
    });

    config.rcc.xspi1_clk_src = XspiClkSrc::IC3;
    config.rcc.vddio2_1v8 = true; // Critical: GPIOO/GPIOP require 1.8V for PSRAM
    let p = embassy_stm32::init(config);

    info!("XSPI PSRAM Example for STM32N6570-DK");

    // === DCACHE DISABLE TEST ===
    // Disable D-cache entirely to test if byte write corruption is cache-related
    info!("Disabling D-cache for testing...");
    let mut cp = unsafe { cortex_m::Peripherals::steal() };
    cp.SCB.disable_dcache(&mut cp.CPUID);
    info!("D-cache disabled");

    // Configure LED for status indication
    let mut led = Output::new(p.PG10, Level::Low, Speed::Low);

    // Configure XSPI for the external PSRAM (APS256XX)
    let spi_config = embassy_stm32::xspi::Config {
        fifo_threshold: FIFOThresholdLevel::_4Bytes,
        memory_type: MemoryType::APMemory16Bits, // AP Memory 16-bit mode for PSRAM
        delay_hold_quarter_cycle: false,         // Match C HAL: disabled for PSRAM
        device_size: MemorySize::_32MiB,         // 256 Mbit = 32 MiB
        chip_select_high_time: ChipSelectHighTime::_5Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        clock_prescaler: 1,       // Start at 100MHz for initialization (C HAL approach)
        sample_shifting: false,   // Match C HAL: no sample shifting
        chip_select_boundary: 14, // 16KB boundary for row crossing
        max_transfer: 0,
        refresh: 0,
    };
    // Create XSPI driver for PSRAM (XSPI1, Port 1 pins)
    // Pin mapping:
    // CLK=PO4, D0-D15=PP0-PP15, NCS=PO0, DQS0=PO2, DQS1=PO3
    let xspi = Xspi::new_blocking_xspi_hexa_dqs_dual(
        p.XSPI1, p.PO4, // CLK
        p.PP0, p.PP1, p.PP2, p.PP3, // D0-D3
        p.PP4, p.PP5, p.PP6, p.PP7, // D4-D7
        p.PP8, p.PP9, p.PP10, p.PP11, // D8-D11
        p.PP12, p.PP13, p.PP14, p.PP15, // D12-D15
        p.PO0,  // NCS1
        p.PO2,  // DQS0
        p.PO3,  // DQS1
        spi_config,
    );

    let mut psram = Aps256Psram::new(xspi);

    // Configure PSRAM mode registers (at 100MHz for reliable initialization)
    info!("Configuring PSRAM mode registers...");
    psram.configure_mode_registers();
    info!("Mode registers configured");

    // Switch to full 200MHz speed after configuration (mirrors C HAL BSP approach)
    info!("Switching to full speed (200MHz)...");
    psram.set_full_speed();
    // Enable memory-mapped mode for PSRAM read/write
    info!("Enabling memory-mapped mode...");
    psram.enable_memory_mapped_mode();
    info!("Memory-mapped mode enabled");

    // === DEBUG 1: Dump XSPI registers after memory-mapped enable ===
    {
        let xspi1 = embassy_stm32::pac::XSPI1;
        info!("XSPI1 registers after MM enable:");
        info!("  CR:   0x{:08x}", xspi1.cr().read().0);
        info!("  CCR:  0x{:08x}", xspi1.ccr().read().0);
        info!("  TCR:  0x{:08x}", xspi1.tcr().read().0);
        info!("  WCCR: 0x{:08x}", xspi1.wccr().read().0);
        info!("  WTCR: 0x{:08x}", xspi1.wtcr().read().0);
        info!("  WIR:  0x{:08x}", xspi1.wir().read().0);
    }

    // Test address at 1 MB offset
    const TEST_ADDR: u32 = 0x00100000;
    const TEST_SIZE: usize = 256;
    let mut total_errors: usize = 0;

    // Note: D-cache disabled at start, so no cache operations needed

    // Test single byte write and readback
    info!("Testing single byte write...");
    let mm_ptr = (MM_BASE + TEST_ADDR) as *mut u8;
    unsafe { core::ptr::write_volatile(mm_ptr, 0xAA) };

    info!("Testing single byte read...");
    let mm_ptr_read = (MM_BASE + TEST_ADDR) as *const u8;
    let val = unsafe { core::ptr::read_volatile(mm_ptr_read) };
    info!("Single read completed, value=0x{:02x}", val);

    // === DEBUG 2: Raw 8-bit writes (some failures expected) ===
    // This test demonstrates sub-word write corruption. The exact cause is not yet
    // fully researched. Some byte offsets will fail.
    info!("DEBUG 2: Raw 8-bit writes with immediate readback (first 32 bytes)...");
    let mut raw8_errors = 0;
    for i in 0..32 {
        let val = i as u8;
        let ptr_i = unsafe { mm_ptr.add(i) };
        let ptr_i_read = unsafe { mm_ptr_read.add(i) };
        unsafe { core::ptr::write_volatile(ptr_i, val) };
        cortex_m::asm::dsb();
        let readback = unsafe { core::ptr::read_volatile(ptr_i_read) };
        if val != readback {
            raw8_errors += 1;
            error!("RAW fail: offset {} wrote 0x{:02x} read 0x{:02x}", i, val, readback);
        } else {
            info!("RAW ok:   offset {} = 0x{:02x}", i, val);
        }
    }
    info!("DEBUG 2: Raw 8-bit writes: {} errors out of 32", raw8_errors);
    total_errors += raw8_errors;

    // === DEBUG 2b: Raw 16-bit writes with error count ===
    info!("DEBUG 2b: Testing raw 16-bit writes (32 bytes = 16 u16)...");
    let mm_ptr_raw16 = (MM_BASE + TEST_ADDR + 0x0800) as *mut u16;
    let mm_ptr_raw16_read = mm_ptr_raw16 as *const u16;
    for i in 0..16usize {
        let val = ((i * 2) as u16) | (((i * 2 + 1) as u16) << 8);
        unsafe { core::ptr::write_volatile(mm_ptr_raw16.add(i), val) };
    }

    let mut raw16_errors = 0;
    for i in 0..16usize {
        let expected = ((i * 2) as u16) | (((i * 2 + 1) as u16) << 8);
        let actual = unsafe { core::ptr::read_volatile(mm_ptr_raw16_read.add(i)) };
        if actual != expected {
            raw16_errors += 1;
            error!("RAW16 fail idx {}: expected 0x{:04x} got 0x{:04x}", i, expected, actual);
        }
    }
    info!("DEBUG 2b: Raw 16-bit writes: {} errors out of 16", raw16_errors);
    total_errors += raw16_errors;

    // === DEBUG 2c: Raw 32-bit writes with error count ===
    info!("DEBUG 2c: Testing raw 32-bit writes (32 bytes = 8 u32)...");
    let mm_ptr_raw32 = (MM_BASE + TEST_ADDR + 0x0C00) as *mut u32;
    let mm_ptr_raw32_read = mm_ptr_raw32 as *const u32;
    for i in 0..8usize {
        let val = ((i * 4) as u32)
            | (((i * 4 + 1) as u32) << 8)
            | (((i * 4 + 2) as u32) << 16)
            | (((i * 4 + 3) as u32) << 24);
        unsafe { core::ptr::write_volatile(mm_ptr_raw32.add(i), val) };
    }

    let mut raw32_errors = 0;
    for i in 0..8usize {
        let expected = ((i * 4) as u32)
            | (((i * 4 + 1) as u32) << 8)
            | (((i * 4 + 2) as u32) << 16)
            | (((i * 4 + 3) as u32) << 24);
        let actual = unsafe { core::ptr::read_volatile(mm_ptr_raw32_read.add(i)) };
        if actual != expected {
            raw32_errors += 1;
            error!("RAW32 fail idx {}: expected 0x{:08x} got 0x{:08x}", i, expected, actual);
        }
    }
    info!("DEBUG 2c: Raw 32-bit writes: {} errors out of 8", raw32_errors);
    total_errors += raw32_errors;

    // === DEBUG 3: Test 16-bit aligned writes ===
    info!("DEBUG 3: Testing 16-bit aligned writes...");
    let mm_ptr_u16 = (MM_BASE + TEST_ADDR + 0x1000) as *mut u16; // Different offset to avoid overlap
    let mm_ptr_u16_read = (MM_BASE + TEST_ADDR + 0x1000) as *const u16;
    for i in 0..16 {
        let val = ((i * 2) as u16) | (((i * 2 + 1) as u16) << 8); // e.g. 0x0100, 0x0302...
        unsafe { core::ptr::write_volatile(mm_ptr_u16.add(i), val) };
    }

    info!("Reading back 16-bit aligned writes...");
    let mut u16_errors = 0;
    for i in 0..16 {
        let expected = ((i * 2) as u16) | (((i * 2 + 1) as u16) << 8);
        let actual = unsafe { core::ptr::read_volatile(mm_ptr_u16_read.add(i)) };
        if actual != expected {
            u16_errors += 1;
            error!(
                "U16 mismatch at idx {}: expected 0x{:04x} got 0x{:04x}",
                i, expected, actual
            );
        } else {
            info!("U16 ok: idx {} = 0x{:04x}", i, actual);
        }
    }
    info!("DEBUG 3: 16-bit writes: {} errors out of 16", u16_errors);
    total_errors += u16_errors;

    // === DEBUG 4: Test 32-bit aligned writes ===
    info!("DEBUG 4: Testing 32-bit aligned writes...");
    let mm_ptr_u32 = (MM_BASE + TEST_ADDR + 0x2000) as *mut u32; // Different offset
    let mm_ptr_u32_read = (MM_BASE + TEST_ADDR + 0x2000) as *const u32;
    for i in 0..8 {
        // Write pattern: bytes i*4, i*4+1, i*4+2, i*4+3 packed into u32
        let val = ((i * 4) as u32)
            | (((i * 4 + 1) as u32) << 8)
            | (((i * 4 + 2) as u32) << 16)
            | (((i * 4 + 3) as u32) << 24);
        unsafe { core::ptr::write_volatile(mm_ptr_u32.add(i), val) };
    }

    info!("Reading back 32-bit aligned writes...");
    let mut u32_errors = 0;
    for i in 0..8 {
        let expected = ((i * 4) as u32)
            | (((i * 4 + 1) as u32) << 8)
            | (((i * 4 + 2) as u32) << 16)
            | (((i * 4 + 3) as u32) << 24);
        let actual = unsafe { core::ptr::read_volatile(mm_ptr_u32_read.add(i)) };
        if actual != expected {
            u32_errors += 1;
            error!(
                "U32 mismatch at idx {}: expected 0x{:08x} got 0x{:08x}",
                i, expected, actual
            );
        } else {
            info!("U32 ok: idx {} = 0x{:08x}", i, actual);
        }
    }
    info!("DEBUG 4: 32-bit writes: {} errors out of 8", u32_errors);
    total_errors += u32_errors;

    // === DEBUG 5: Test 64-bit aligned writes ===
    // Hypothesis: Native 64-bit writes should work at all alignments
    info!("DEBUG 5: Testing 64-bit aligned writes...");
    let mm_ptr_u64 = (MM_BASE + TEST_ADDR + 0x3000) as *mut u64; // Different offset
    let mm_ptr_u64_read = (MM_BASE + TEST_ADDR + 0x3000) as *const u64;
    for i in 0..4usize {
        // Write pattern: bytes i*8..i*8+7 packed into u64
        let val = ((i * 8) as u64)
            | (((i * 8 + 1) as u64) << 8)
            | (((i * 8 + 2) as u64) << 16)
            | (((i * 8 + 3) as u64) << 24)
            | (((i * 8 + 4) as u64) << 32)
            | (((i * 8 + 5) as u64) << 40)
            | (((i * 8 + 6) as u64) << 48)
            | (((i * 8 + 7) as u64) << 56);
        unsafe { core::ptr::write_volatile(mm_ptr_u64.add(i), val) };
    }

    info!("Reading back 64-bit aligned writes...");
    let mut u64_errors = 0;
    for i in 0..4usize {
        let expected = ((i * 8) as u64)
            | (((i * 8 + 1) as u64) << 8)
            | (((i * 8 + 2) as u64) << 16)
            | (((i * 8 + 3) as u64) << 24)
            | (((i * 8 + 4) as u64) << 32)
            | (((i * 8 + 5) as u64) << 40)
            | (((i * 8 + 6) as u64) << 48)
            | (((i * 8 + 7) as u64) << 56);
        let actual = unsafe { core::ptr::read_volatile(mm_ptr_u64_read.add(i)) };
        if actual != expected {
            u64_errors += 1;
            error!(
                "U64 mismatch at idx {}: expected 0x{:016x} got 0x{:016x}",
                i, expected, actual
            );
        } else {
            info!("U64 ok: idx {} = 0x{:016x}", i, actual);
        }
    }
    info!("DEBUG 5: 64-bit writes: {} errors out of 4", u64_errors);
    total_errors += u64_errors;

    // === DEBUG 6: Test 64-bit R-M-W byte writes (workaround) ===
    // This test uses the psram_write_byte() helper which performs 64-bit read-modify-write
    info!("DEBUG 6: Testing 64-bit R-M-W byte writes (workaround)...");
    let mm_ptr_rmw = (MM_BASE + TEST_ADDR + 0x4000) as *mut u8; // Different offset
    let mm_ptr_rmw_read = (MM_BASE + TEST_ADDR + 0x4000) as *const u8;

    // First, zero out the region with 64-bit writes
    let mm_ptr_rmw_u64 = mm_ptr_rmw as *mut u64;
    for i in 0..4 {
        unsafe { core::ptr::write_volatile(mm_ptr_rmw_u64.add(i), 0u64) };
    }

    // Now write bytes using 64-bit R-M-W helper
    for i in 0..32 {
        unsafe { psram_write_byte(mm_ptr_rmw.add(i), i as u8) };
    }

    // Verify
    let mut rmw_errors = 0;
    for i in 0..32 {
        let expected = i as u8;
        let actual = unsafe { core::ptr::read_volatile(mm_ptr_rmw_read.add(i)) };
        if actual != expected {
            rmw_errors += 1;
            error!(
                "RMW fail: offset {} expected 0x{:02x} got 0x{:02x}",
                i, expected, actual
            );
        } else {
            info!("RMW ok:   offset {} = 0x{:02x}", i, actual);
        }
    }
    info!("DEBUG 6: 64-bit R-M-W byte writes: {} errors out of 32", rmw_errors);
    total_errors += rmw_errors;

    // === FINAL TEST: Full 256-byte write using 64-bit R-M-W helper ===
    info!("FINAL TEST: Writing {} bytes using 64-bit R-M-W helper...", TEST_SIZE);
    let mm_ptr_final = (MM_BASE + TEST_ADDR + 0x5000) as *mut u8;
    let mm_ptr_final_read = (MM_BASE + TEST_ADDR + 0x5000) as *const u8;

    // First, zero out the region with 64-bit writes
    let mm_ptr_final_u64 = mm_ptr_final as *mut u64;
    for i in 0..(TEST_SIZE / 8) {
        unsafe { core::ptr::write_volatile(mm_ptr_final_u64.add(i), 0u64) };
    }

    // Write using R-M-W helper
    for i in 0..TEST_SIZE {
        unsafe { psram_write_byte(mm_ptr_final.add(i), i as u8) };
    }

    // Read back and verify
    info!("Verifying {} bytes...", TEST_SIZE);
    let mut errors = 0;
    let mut first_error_positions: [u16; 16] = [0xFFFF; 16];
    for i in 0..TEST_SIZE {
        let expected = i as u8;
        let actual = unsafe { core::ptr::read_volatile(mm_ptr_final_read.add(i)) };
        if actual != expected {
            if errors < 16 {
                first_error_positions[errors] = i as u16;
            }
            errors += 1;
        }
    }

    info!("FINAL TEST: {} errors out of {} bytes", errors, TEST_SIZE);
    if errors > 0 {
        let num_to_show = core::cmp::min(errors, 16);
        for j in 0..num_to_show {
            let pos = first_error_positions[j] as usize;
            let expected = pos as u8;
            let actual = unsafe { core::ptr::read_volatile(mm_ptr_final_read.add(pos)) };
            error!("  offset {}: expected 0x{:02x}, got 0x{:02x}", pos, expected, actual);
        }
    }

    // Show first 16 bytes read back
    let mut read_sample = [0u8; 16];
    for i in 0..16 {
        read_sample[i] = unsafe { core::ptr::read_volatile(mm_ptr_final_read.add(i)) };
    }
    info!("Read back first 16 bytes: {=[u8]:x}", read_sample);

    let all_errors = total_errors + errors;
    info!("=== SUMMARY ===");
    info!("Total errors across all tests: {}", all_errors);

    if all_errors == 0 {
        info!("PSRAM test PASSED!");
        // Blink LED slowly to indicate success
        loop {
            led.toggle();
            Timer::after_millis(500).await;
        }
    } else {
        error!("PSRAM test FAILED with {} total errors!", all_errors);
        // Blink LED fast to indicate error
        loop {
            led.toggle();
            Timer::after_millis(100).await;
        }
    }
}

/// APS256XX PSRAM driver
pub struct Aps256Psram<I: Instance> {
    xspi: Xspi<'static, I, Blocking>,
}

impl<I: Instance> Aps256Psram<I> {
    pub fn new(xspi: Xspi<'static, I, Blocking>) -> Self {
        Self { xspi }
    }

    /// Reset the PSRAM
    pub fn reset(&mut self) {
        // Reset command: 8-line instruction STR, 8-line address STR (24-bit), no data
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_8bit,
            idtr: false,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_24bit,
            addtr: false,
            dwidth: XspiWidth::NONE,
            instruction: Some(aps256_cmd::RESET),
            address: Some(0),
            dummy: DummyCycles::_0,
            dqse: false,
            ..Default::default()
        };
        self.xspi.blocking_command(&transaction).unwrap();
    }

    /// Configure PSRAM mode registers for optimal operation
    /// Following C HAL pattern: write configuration first, then read to verify
    pub fn configure_mode_registers(&mut self) {
        // C HAL regW_MR0[2]={0x30,0x8D} - LT=Fixed, RLC=4 (latency 6), Full drive
        // DTR mode writes to MRn and MRn+1 simultaneously
        info!("Writing MR0/MR1...");
        self.write_register(aps256_mr::MR0, 0x30, 0x8D);
        info!("MR0/MR1 written");

        // C HAL regW_MR4[2]={0x20,0xF0} - WLC=7 for up to 200MHz
        info!("Writing MR4/MR5...");
        self.write_register(aps256_mr::MR4, 0x20, 0xF0);
        info!("MR4/MR5 written");

        // C HAL regW_MR8[2]={0x4B,0x08} - X16 mode, RBX, 2K burst
        info!("Writing MR8/MR9...");
        self.write_register(aps256_mr::MR8, 0x4B, 0x08);
        info!("MR8/MR9 written");

        // Now verify by reading back
        info!("Reading MR0...");
        let mr0 = self.read_register(aps256_mr::MR0);
        info!("MR0 read: 0x{:02x}", mr0);

        info!("Reading MR4...");
        let mr4 = self.read_register(aps256_mr::MR4);
        info!("MR4 read: 0x{:02x}", mr4);

        info!("Reading MR8...");
        let mr8 = self.read_register(aps256_mr::MR8);
        info!("MR8 read: 0x{:02x}", mr8);

        info!(
            "All registers configured: MR0=0x{:02x}, MR4=0x{:02x}, MR8=0x{:02x}",
            mr0, mr4, mr8
        );
    }

    /// Switch to full speed (200MHz) after PSRAM configuration
    /// This mirrors the C HAL BSP approach: init at 100MHz, then switch to 200MHz
    pub fn set_full_speed(&mut self) {
        self.xspi.set_clock_prescaler(0);
    }

    /// Read a mode register
    pub fn read_register(&mut self, address: u32) -> u8 {
        let mut buffer = [0u8; 2]; // DTR mode returns 2 bytes

        // Register read: OCTO instruction (8-bit, STR), OCTO address (32-bit, DTR),
        // OCTO data (DTR), dummy cycles = latency - 1, DQS enabled
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_8bit,
            idtr: false, // Instruction is STR
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            addtr: true, // Address is DTR
            dwidth: XspiWidth::OCTO,
            ddtr: true, // Data is DTR
            instruction: Some(aps256_cmd::READ_REG),
            address: Some(address),
            dummy: DummyCycles::_5, // C HAL uses (latency-1) = 5 for register access
            dqse: true,
            ..Default::default()
        };
        self.xspi.blocking_read(&mut buffer, transaction).unwrap();
        buffer[0]
    }

    /// Write a mode register pair (MRn and MRn+1)
    /// In DTR mode, 2 bytes are sent to consecutive register addresses
    pub fn write_register(&mut self, address: u32, value0: u8, value1: u8) {
        let buffer = [value0, value1]; // DTR mode sends 2 bytes to MRn and MRn+1

        // Register write: OCTO instruction (8-bit, STR), OCTO address (32-bit, DTR),
        // OCTO data (DTR), no dummy cycles, DQS disabled
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_8bit,
            idtr: false, // Instruction is STR
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            addtr: true, // Address is DTR
            dwidth: XspiWidth::OCTO,
            ddtr: true, // Data is DTR
            instruction: Some(aps256_cmd::WRITE_REG),
            address: Some(address),
            dummy: DummyCycles::_0,
            dqse: false,
            ..Default::default()
        };
        self.xspi.blocking_write(&buffer, transaction).unwrap();
    }

    /// Enable memory-mapped mode
    pub fn enable_memory_mapped_mode(&mut self) {
        // Read configuration for memory-mapped mode
        let read_config = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_8bit,
            idtr: false,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            addtr: true,
            dwidth: XspiWidth::HEXA,
            ddtr: true,
            instruction: Some(aps256_cmd::READ), // C HAL uses 0x00 (Synchronous Read)
            dummy: DummyCycles::_6,              // Match C HAL: 6 dummy cycles for reads
            dqse: true,
            ..Default::default()
        };

        // Write configuration for memory-mapped mode
        let write_config = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_8bit,
            idtr: false,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            addtr: true,
            dwidth: XspiWidth::HEXA,
            ddtr: true,
            instruction: Some(aps256_cmd::WRITE), // C HAL uses 0x80 (Synchronous Write)
            dummy: DummyCycles::_6,               // Match C HAL: 6 dummy cycles for writes
            dqse: true,
            ..Default::default()
        };

        self.xspi.enable_memory_mapped_mode(read_config, write_config).unwrap();
    }
}
