#![no_std]
#![no_main]

//! XSPI External Flash Example for STM32N6570-DK
//!
//! This example demonstrates how to use the XSPI peripheral to communicate with
//! the external NOR flash (MX66UW1G45G) on the STM32N6570-DK board.
//!
//! The flash is connected via XSPI2 using Octo-SPI interface:
//! - Memory-mapped address: 0x70000000
//! - Size: 1 Gbit (128 MB)
//! - Interface: Octo-SPI (8-bit data width)
//!
//! Pin mapping (XSPIM Port 2):
//! - PN2: IO0, PN3: IO1, PN4: IO2, PN5: IO3
//! - PN8: IO4, PN9: IO5, PN10: IO6, PN11: IO7
//! - PN6: CLK, PN7: NCLK (optional)
//! - PN1: NCS1, PN0: DQS0 (optional)

use core::cmp::min;

use defmt::{error, info, warn};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::rcc::{IcConfig, Icint, Icsel, XspiClkSrc};
use embassy_stm32::xspi::{
    AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, Instance, MemorySize, MemoryType, TransferConfig,
    WrapSize, Xspi, XspiWidth,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const MEMORY_PAGE_SIZE: usize = 256;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Configure RCC with IC4 for XSPI2 kernel clock
    let mut config = embassy_stm32::Config::default();
    config.rcc.ic4 = Some(IcConfig {
        source: Icsel::PLL2,          // PLL2 in bypass mode = HSI 64 MHz
        divider: Icint::from_bits(0), // DIV1 = no division
    });
    config.rcc.mux.xspi2sel = XspiClkSrc::IC4; // Select IC4 for XSPI2 kernel clock
    config.rcc.vddio3_1v8 = true;
    let p = embassy_stm32::init(config);
    info!("XSPI Flash Example for STM32N6570-DK");

    // Configure XSPI for the external NOR flash
    let spi_config = embassy_stm32::xspi::Config {
        fifo_threshold: FIFOThresholdLevel::_4Bytes,
        memory_type: MemoryType::Macronix, // MX66UW1G45G is Macronix
        delay_hold_quarter_cycle: true,
        device_size: MemorySize::_128MiB, // 1 Gbit = 128 MiB
        chip_select_high_time: ChipSelectHighTime::_2Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        clock_prescaler: 8,    // Conservative prescaler for debugging (64MHz / 8 = 8MHz)
        sample_shifting: true, // Enable sample shifting for better signal capture
        chip_select_boundary: 0,
        max_transfer: 0,
        refresh: 0,
    };

    // Create XSPI driver for flash (XSPI2, Port P2 pins)
    // Pin mapping: CLK=PN6, D0=PN2, D1=PN3, D2=PN4, D3=PN5, D4=PN8, D5=PN9, D6=PN10, D7=PN11, NCS=PN1
    let xspi = Xspi::new_blocking_xspi(
        p.XSPI2, p.PN6, p.PN2, p.PN3, p.PN4, p.PN5, p.PN8, p.PN9, p.PN10, p.PN11, p.PN1, spi_config,
    );

    // Wait for flash to be ready after power-on
    info!("Waiting for flash to initialize...");
    Timer::after_millis(100).await;

    let mut flash = SpiFlashMemory::new(xspi);

    // Read flash ID multiple times to check consistency
    let flash_id = flash.read_id();
    info!("FLASH ID: {=[u8]:x}", flash_id);

    // Read status register to check if flash is responding
    let sr = flash.read_sr();
    info!("Status Register: 0x{:02x}", sr);

    // If ID is all zeros, the flash is not responding
    if flash_id == [0, 0, 0] {
        warn!("Flash not responding! Check pin connections and clock configuration.");
        warn!("Expected ID for MX66UW1G45G: [0xC2, 0x85, 0x3B]");

        // Try reading ID again after delay
        Timer::after_millis(100).await;
        let flash_id2 = flash.read_id();
        info!("FLASH ID (retry): {=[u8]:x}", flash_id2);

        if flash_id2 == [0, 0, 0] {
            error!("Flash still not responding. Aborting to prevent crash.");
            // Blink LED fast to indicate error
            let mut led = Output::new(p.PG10, Level::High, Speed::Low);
            loop {
                led.toggle();
                Timer::after_millis(100).await;
            }
        }
    }

    // Flash size: 128 MiB = 0x08000000 bytes
    // Sector size: 4 KiB = 0x1000 bytes
    // Test at last sector: 0x08000000 - 0x1000 = 0x07FFF000
    const TEST_ADDR: u32 = 0x07FFF000;

    // 1. Read original data and save to RAM
    let mut original_data = [0u8; 256];
    info!("Reading original data at address 0x{:08x}...", TEST_ADDR);
    flash.read_memory(TEST_ADDR, &mut original_data);
    info!("Original data: {=[u8]:x}", &original_data[0..16]);

    // 2. Erase the sector at test address
    info!("Erasing sector at 0x{:08x}...", TEST_ADDR);
    flash.erase_sector(TEST_ADDR);

    // 3. Write test pattern
    let mut test_data = [0u8; 256];
    for i in 0..256 {
        test_data[i] = i as u8;
    }
    info!("Writing test pattern...");
    flash.write_memory(TEST_ADDR, &test_data);

    // 4. Read back test data
    let mut read_data = [0u8; 256];
    flash.read_memory(TEST_ADDR, &mut read_data);
    info!("Read back: {=[u8]:x}", &read_data[0..16]);

    // 5. Verify test data
    if test_data == read_data {
        info!("Verification PASSED!");
    } else {
        error!("Verification FAILED!");
    }

    // 6. Restore original data
    info!("Restoring original data...");
    flash.erase_sector(TEST_ADDR);
    flash.write_memory(TEST_ADDR, &original_data);

    // Verify restoration
    let mut verify_restore = [0u8; 256];
    flash.read_memory(TEST_ADDR, &mut verify_restore);
    if original_data == verify_restore {
        info!("Original data restored successfully!");
    } else {
        error!("Failed to restore original data!");
    }

    // =========================================================================
    // Memory-Mapped Mode Demonstration
    // =========================================================================
    // NOR flash in memory-mapped mode has limitations:
    // - Reads: Work seamlessly - just dereference pointers to the memory-mapped address space
    // - Writes: NOR flash requires erase-before-write and page programming. Memory-mapped
    //   writes don't bypass this - you still need indirect mode for erase/program cycles.

    info!("Enabling memory-mapped mode...");
    flash.enable_mm();

    const MM_BASE: u32 = 0x70000000;
    let mm_base_ptr = MM_BASE as *const u8;

    // --- Memory-Mapped READS ---

    // 1. Single u32 read (basic demonstration)
    let test_u32 = unsafe { *((MM_BASE + TEST_ADDR) as *const u32) };
    info!("MM read u32 at 0x{:08x}: 0x{:08x}", MM_BASE + TEST_ADDR, test_u32);

    // 2. Read multiple sequential u32s
    info!("Reading multiple u32s in memory-mapped mode...");
    let mut mm_values = [0u32; 4];
    for i in 0..4 {
        mm_values[i] = unsafe { core::ptr::read_volatile((MM_BASE + TEST_ADDR + i as u32 * 4) as *const u32) };
    }
    info!(
        "MM read 4 x u32: {:08x} {:08x} {:08x} {:08x}",
        mm_values[0], mm_values[1], mm_values[2], mm_values[3]
    );

    // 3. Read a buffer using volatile reads
    let mut mm_buffer = [0u8; 16];
    for i in 0..16 {
        mm_buffer[i] = unsafe { core::ptr::read_volatile(mm_base_ptr.add(TEST_ADDR as usize + i)) };
    }
    info!("MM read 16 bytes: {=[u8]:x}", mm_buffer);

    // 4. Compare with previously read indirect mode data
    if mm_buffer == original_data[0..16] {
        info!("Memory-mapped read matches indirect mode read!");
    } else {
        error!("Memory-mapped read differs from indirect mode!");
    }

    flash.disable_mm();
    info!("Memory-mapped mode disabled");

    // --- Memory-Mapped WRITE demonstration ---
    // NOR flash requires erase+program cycle. Demonstrate the pattern:
    // 1. Write new data in indirect mode (requires erase first)
    // 2. Re-enable memory-mapped mode
    // 3. Verify via memory-mapped reads
    info!("Demonstrating memory-mapped write pattern for NOR flash...");

    // Write new test data (requires leaving MM mode for erase/program)
    let mut new_test_data = [0u8; 256];
    for i in 0..256 {
        new_test_data[i] = (255 - i) as u8; // Inverse pattern
    }

    // Erase and write in indirect mode
    flash.erase_sector(TEST_ADDR);
    flash.write_memory(TEST_ADDR, &new_test_data);
    info!("Wrote new pattern in indirect mode");

    // Re-enable memory-mapped mode and verify
    flash.enable_mm();
    info!("Re-enabled memory-mapped mode");

    // Read back via memory-mapped mode
    let mut mm_verify = [0u8; 16];
    for i in 0..16 {
        mm_verify[i] = unsafe { core::ptr::read_volatile(mm_base_ptr.add(TEST_ADDR as usize + i)) };
    }
    info!("MM verify read: {=[u8]:x}", mm_verify);

    if mm_verify == new_test_data[0..16] {
        info!("Memory-mapped verify PASSED!");
    } else {
        error!("Memory-mapped verify FAILED!");
    }

    flash.disable_mm();

    // Restore original data
    flash.erase_sector(TEST_ADDR);
    flash.write_memory(TEST_ADDR, &original_data);
    info!("Original data restored");

    info!("Example complete!");

    // Blink LED to indicate success
    let mut led = Output::new(p.PG10, Level::High, Speed::Low);
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}

/// SPI Flash Memory driver for MX66UW1G45G
pub struct SpiFlashMemory<I: Instance> {
    xspi: Xspi<'static, I, Blocking>,
}

/// SPI mode commands for MX66UW1G45G flash memory
#[allow(dead_code)]
#[repr(u8)]
enum SpiCommand {
    Read3B = 0x03,
    FastRead3B = 0x0B,
    PageProgram3B = 0x02,
    SectorErase3B = 0x20,
    Read4B = 0x13,
    FastRead4B = 0x0C,
    PageProgram4B = 0x12,
    SectorErase4B = 0x21,
    BlockErase4B = 0xDC,
    ChipErase = 0x60,
    WriteEnable = 0x06,
    WriteDisable = 0x04,
    ResetEnable = 0x66,
    ResetMemory = 0x99,
    ReadIdentification = 0x9F,
    ReadStatusRegister = 0x05,
}

impl<I: Instance> SpiFlashMemory<I> {
    pub fn new(xspi: Xspi<'static, I, Blocking>) -> Self {
        let mut memory = Self { xspi };
        memory.reset_memory();
        memory
    }

    pub fn reset_memory(&mut self) {
        self.exec_command(SpiCommand::ResetEnable as u8);
        self.exec_command(SpiCommand::ResetMemory as u8);
        self.wait_write_finish();
    }

    fn exec_command(&mut self, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            adwidth: XspiWidth::NONE,
            dwidth: XspiWidth::NONE,
            instruction: Some(cmd as u32),
            address: None,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.xspi.blocking_command(&transaction).unwrap();
    }

    pub fn enable_write(&mut self) {
        self.exec_command(SpiCommand::WriteEnable as u8);
    }

    pub fn read_id(&mut self) -> [u8; 3] {
        let mut buffer = [0; 3];
        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            adwidth: XspiWidth::NONE,
            dwidth: XspiWidth::SING,
            instruction: Some(SpiCommand::ReadIdentification as u32),
            ..Default::default()
        };
        self.xspi.blocking_read(&mut buffer, transaction).unwrap();
        buffer
    }

    pub fn read_sr(&mut self) -> u8 {
        let mut buffer = [0; 1];
        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            adwidth: XspiWidth::NONE,
            dwidth: XspiWidth::SING,
            instruction: Some(SpiCommand::ReadStatusRegister as u32),
            ..Default::default()
        };
        self.xspi.blocking_read(&mut buffer, transaction).unwrap();
        buffer[0]
    }

    fn wait_write_finish(&mut self) {
        while (self.read_sr() & 0x01) != 0 {}
    }

    pub fn read_memory(&mut self, addr: u32, buffer: &mut [u8]) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            adwidth: XspiWidth::SING,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::SING,
            instruction: Some(SpiCommand::FastRead4B as u32),
            dummy: DummyCycles::_8,
            address: Some(addr),
            ..Default::default()
        };
        self.xspi.blocking_read(buffer, transaction).unwrap();
    }

    pub fn erase_sector(&mut self, addr: u32) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            adwidth: XspiWidth::SING,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::NONE,
            instruction: Some(SpiCommand::SectorErase4B as u32),
            address: Some(addr),
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.enable_write();
        self.xspi.blocking_command(&transaction).unwrap();
        self.wait_write_finish();
    }

    fn write_page(&mut self, addr: u32, buffer: &[u8], len: usize) {
        assert!(
            (len as u32 + (addr & 0x000000ff)) <= MEMORY_PAGE_SIZE as u32,
            "write_page(): page write length exceeds page boundary"
        );

        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            adsize: AddressSize::_32bit,
            adwidth: XspiWidth::SING,
            dwidth: XspiWidth::SING,
            instruction: Some(SpiCommand::PageProgram4B as u32),
            address: Some(addr),
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.enable_write();
        self.xspi.blocking_write(buffer, transaction).unwrap();
        self.wait_write_finish();
    }

    pub fn write_memory(&mut self, addr: u32, buffer: &[u8]) {
        let mut left = buffer.len();
        let mut place = addr;
        let mut chunk_start = 0;

        while left > 0 {
            let max_chunk_size = MEMORY_PAGE_SIZE - (place & 0x000000ff) as usize;
            let chunk_size = min(max_chunk_size, left);
            let chunk = &buffer[chunk_start..(chunk_start + chunk_size)];
            self.write_page(place, chunk, chunk_size);
            place += chunk_size as u32;
            left -= chunk_size;
            chunk_start += chunk_size;
        }
    }

    pub fn enable_mm(&mut self) {
        let read_config = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            adwidth: XspiWidth::SING,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::SING,
            instruction: Some(SpiCommand::FastRead4B as u32),
            dummy: DummyCycles::_8,
            ..Default::default()
        };

        let write_config = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            adwidth: XspiWidth::SING,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::SING,
            instruction: Some(SpiCommand::PageProgram4B as u32),
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.xspi.enable_memory_mapped_mode(read_config, write_config).unwrap();
    }

    pub fn disable_mm(&mut self) {
        self.xspi.disable_memory_mapped_mode();
    }
}
