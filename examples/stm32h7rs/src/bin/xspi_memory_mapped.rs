#![no_main]
#![no_std]

//! This example is intended for the Nucleo STM32H7S3L8 MB1737, which has a
//! Macronix MX25UW25645GXDI00 flash chip installed on XSPI2, powered at 1.8 V.
//!
//! IMPORTANT: this example use HSLV (high-speed, low-voltage) mode for XSPI2,
//!            to which the flash is connected, to achieve the maximum flash
//!            bus speeds in STR communication mode. This is safe on the Nucleo
//!            board, which is factory-wired for a fixed 1.8 V on VDD_XSPI2.
//!            For other boards, this may not be so, and may be destructive!
//!            See the notes in the RCC config below for more details.

use core::cmp::min;

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::rcc::mux::Xspisel;
use embassy_stm32::rcc::{get_corrected_comp_vals, set_and_enable_comp_vals};
use embassy_stm32::time::Hertz;
use embassy_stm32::xspi::{
    AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, Instance, MemorySize, MemoryType, TransferConfig,
    WrapSize, Xspi, XspiWidth,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // RCC config
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;

        // CSI is required for HSLV and the compensation cells.
        config.rcc.csi = true;

        // Run from the external, 24 MHz crystal oscillator.
        config.rcc.hse = Some(Hse {
            freq: Hertz(24_000_000),
            mode: HseMode::Oscillator,
        });

        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div3,
            mul: PllMul::Mul150,      // 24 MHz / 3 * 150 = 1200 MHz.
            divp: Some(PllDiv::Div2), // 600 MHz PLLCLK for SYSCLK.
            divq: None,
            divr: None,
            divs: None,
            divt: None,
        });

        // PLL2_S will be used as the clock source for XSPI2.
        // A signal with a 50 % duty-cycle is required. The PLL always has a
        // 50% duty-cycle output on post-dividers divs and divt when VCOH is
        // selected (PLLxVCOSEL = 0). This should be so, for the given
        // configuration. However, the uncertainty can be readily avoided by
        // generating double the required frequency, and then post-dividing
        // by 2 in the XSPI peripheral.
        // For bus clk of 133 MHZ: 24 MHz / 4 * 133 / 3 = 266 MHz (and then /2 in XSPI)
        //                200 MHz: 24 MHz / 6 * 200 / 2 = 400 MHz (and then /2 in XSPI)
        config.rcc.pll2 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div6,
            mul: PllMul::Mul200, // 24 MHz / 6 * 200 = 800 MHz
            divp: None,
            divq: None,
            divr: None,
            divs: Some(PllDivSt::Div2), // 400 MHz for XSPI2.
            divt: None,
        });

        // XSPI2 uses PLL2S as its clock source. This allows for more flexible
        // clock rates.
        config.rcc.mux.xspi2sel = Xspisel::Pll2S;

        // Run the core and internal busses at full speed.
        config.rcc.sys = Sysclk::Pll1P; // 600 Mhz
        config.rcc.ahb_pre = AHBPrescaler::Div2; // 300 Mhz
        config.rcc.apb1_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.apb2_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.apb4_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.apb5_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.voltage_scale = VoltageScale::High;

        // Enable HSLV mode to allow high-speed operation at low supply voltage.
        // The STM32H7RS has three power domains where HSLV mode is applicable:
        // - XSPI1 I/Os supplied from VDDXSPI1
        // - XSPI2 I/Os supplied from VDDXSPI2.
        // - General I/Os supplied from VDD.
        //
        // IMPORTANT: Only enable HSLV mode when the specific power domain VDD is
        //            below 2.7 V. Enabling when VDD is > 2.7 V may be destructive!
        //            (RM0477, p.551, Ch. 8.5.10 "SBS I/O compensation cell control
        //            and status register (SBS_CCCSR)")
        //
        // IMPORTANT: JP5 selects all VDD domains EXCEPT XSPI. (Assuming the
        //            factory-default solder bridges are installed.)
        //            Make sure the jumper is in the correct position BEFORE
        //            enabling hslv_io!
        config.rcc.hslv_xspi1 = false; // NUCLEO-H7S3L8: SB20 to VDD (default).
        config.rcc.hslv_xspi2 = true; // NUCLEO-H7S3L8: SB34 to 1.8 V (default).
        config.rcc.hslv_io = false; // NUCLEO-H7S3L8: JP5 (user VDD selection) to 1.8 V or 3.3 V.

        // Selectively enable the compensation cell for the XSPI peripherals.
        // - The XSPI compensation cell auto-tune will fail (and time-out) if no
        //   active device is connected to the selected XSPI port.
        // - The GPIO compensation cell is always enabled, as it can auto-tune
        //   irregardless of connected devices.
        config.rcc.comp_xspi1 = false; // NUCLEO-H7S3L8: not connected.
        config.rcc.comp_xspi2 = true; // NUCLEO-H7S3L8: MX25UW25645GXDI00 flash.
    }

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    let spi_config = embassy_stm32::xspi::Config {
        fifo_threshold: FIFOThresholdLevel::_4Bytes,
        memory_type: MemoryType::Macronix,
        delay_hold_quarter_cycle: true,
        device_size: MemorySize::_32MiB,
        chip_select_high_time: ChipSelectHighTime::_2Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        // 400 MHz clock / (3 + 1) = 100 MHz. This is above the maximum for
        // READ instructions so the FAST READ must be used. The nucleo board's
        // flash can run at up to 133 MHz in SPI mode and 200 MHz in OPI mode.
        // This clock prescaler must be even, otherwise the clock will not have
        // symmetric high and low times. (Note: one is internally added to the
        // clock_prescaler value defined here.)
        clock_prescaler: 3,
        sample_shifting: false,
        chip_select_boundary: 0,
        max_transfer: 0,
        refresh: 0,
    };

    let mut cor = cortex_m::Peripherals::take().unwrap();

    // Not necessary, but recommended if using XIP
    cor.SCB.enable_icache();
    // Note: Enabling data cache can cause issues with DMA transfers.
    cor.SCB.enable_dcache(&mut cor.CPUID);

    // Work-around the errata isse with automatic compensation cell tuning.
    // - Read the corrected, auto-tune values at around 30°C ambient and store
    //   in non-volatile memory.
    // - Apply these values at boot, before attempting high-speed operations.
    //
    // ES0596, p. 12, Ch 2.2.15 "I/O compensation could alter duty-cycle of high-frequency output signal"
    // <https://community.st.com/t5/stm32-mcus-products/stm32h7s7l8h6h-xspi-instability/td-p/749315>
    //
    // Note: applying the errata to the GPIO compensation cell, as is done here,
    //       seems to improve this as well, judging by a signal on the MCO output
    //       pin, although it is not explicitly stated in the errata.
    //
    // Note: these steps are already done in the RCC initialization code.
    //       They are made explicit here, to show how to use the convenience
    //       functions, and to remind the user that this must be implemented
    //       in real-world projects that operate at high I/O speeds, over wider
    //       temperature ranges.
    let cv = get_corrected_comp_vals();
    info!("Compensation Cells - Corrected Tuning Values:");
    info!("- XSPI1: NMOS {:#04X}   PMOS {:#04X}", cv.octo1_nsrc, cv.octo1_psrc);
    info!("- XSPI2: NNOS {:#04X}   PMOS {:#04X}", cv.octo2_nsrc, cv.octo2_psrc);
    info!("- GPIO:  NMOS {:#04X}   PMOS {:#04X}", cv.io_nsrc, cv.io_psrc);
    set_and_enable_comp_vals(&cv);

    let xspi = embassy_stm32::xspi::Xspi::new_blocking_xspi_dqs(
        p.XSPI2, p.PN6, p.PN2, p.PN3, p.PN4, p.PN5, p.PN8, p.PN9, p.PN10, p.PN11, p.PN1, p.PN0, spi_config,
    );

    let mut flash = SpiFlashMemory::new(xspi);
    info!("Starting in SPI mode with 100 MHz XSPI bus clock.");

    // With higher flash clock speeds, the first read_id() returns all zeros,
    // or a hardfault occurs. A short wait (>15µs) resolves this. (This may be
    // due to the flash initialization not having completed yet.)
    Timer::after_micros(50).await;

    let flash_id = flash.read_id();
    info!("FLASH ID: {=[u8]:x}", flash_id);

    // Erase the first sector
    flash.erase_sector(0);

    // Write some data into the flash. This writes more than one page to test
    // that functionality.
    let mut wr_buf = [0u8; 512];
    let base_number: u8 = 0x90;
    for i in 0..512 {
        wr_buf[i] = base_number.wrapping_add(i as u8);
    }
    flash.write_memory(0, &wr_buf);

    // Read the data back and verify it.
    let mut rd_buf = [0u8; 512];
    let start_time = embassy_time::Instant::now();
    flash.read_memory(0, &mut rd_buf);
    let elapsed = start_time.elapsed();
    info!("Read 512 bytes in {} us in SPI mode", elapsed.as_micros());
    info!("WRITE BUF: {=[u8]:#X}", wr_buf[0..32]);
    info!("READ BUF: {=[u8]:#X}", rd_buf[0..32]);

    assert_eq!(wr_buf, rd_buf, "Read buffer does not match write buffer");

    flash.enable_mm();
    info!("Enabled memory mapped mode");

    let first_u32 = unsafe { *(0x70000000 as *const u32) };
    assert_eq!(first_u32, 0x93929190);
    info!("first_u32 {:08x}", first_u32);

    let second_u32 = unsafe { *(0x70000004 as *const u32) };
    assert_eq!(second_u32, 0x97969594);
    info!("second_u32 {:08x}", first_u32);

    flash.disable_mm();
    info!("Disabled memory mapped mode");

    let flash_id = flash.read_id();
    info!("FLASH ID: {=[u8]:x}", flash_id);

    // Change to OPI mode. This allows for higher XSPI bus clock speeds.
    let mut flash = flash.into_octo();
    info!("Entered OPI mode.");

    // Change the bus clock to 200 MHz.
    // 400 MHz / (1 + 1) = 200 MHz (Even ratio: 1+1 = 2)
    flash.xspi.set_clock_prescaler(1);
    info!("XSPI clock prescaler set to /2 -> 200 MHz bus clock");

    Timer::after_millis(100).await;

    let flash_id = flash.read_id();
    info!("FLASH ID in OPI mode: {=[u8]:x}", flash_id);

    flash.erase_sector(0);

    let mut rd_buf = [0u8; 512];
    flash.read_memory(0, &mut rd_buf);
    info!("READ BUF after erase: {=[u8]:#X}", rd_buf[0..32]);

    assert_eq!(rd_buf, [0xFF; 512], "Read buffer is not all 0xFF after erase");

    flash.write_memory(0, &wr_buf);
    let start = embassy_time::Instant::now();
    flash.read_memory(0, &mut rd_buf);
    let elapsed = start.elapsed();
    info!("Read 512 bytes in {} us in OPI mode", elapsed.as_micros());
    info!("READ BUF after write: {=[u8]:#X}", rd_buf[0..32]);
    assert_eq!(wr_buf, rd_buf, "Read buffer does not match write buffer in OPI mode");

    flash.enable_mm();
    info!("Enabled memory mapped mode in OPI mode");
    let first_u32 = unsafe { *(0x70000000 as *const u32) };
    assert_eq!(first_u32, 0x93929190);
    info!("first_u32 {:08x}", first_u32);
    let second_u32 = unsafe { *(0x70000004 as *const u32) };
    assert_eq!(second_u32, 0x97969594);
    info!("second_u32 {:08x}", first_u32);
    flash.disable_mm();
    info!("Disabled memory mapped mode in OPI mode");

    // Change the bus clock to 100 MHz in preparation for re-entering SPI mode.
    // 400 MHz / (3 + 1) = 100 MHz (Even ratio: 3+1 = 4)
    flash.xspi.set_clock_prescaler(3);
    info!("XSPI clock prescaler set to /4 -> 100 MHz bus clock");

    // Reset back to SPI mode.
    let mut flash = flash.into_spi();
    info!("Re-entered SPI mode");

    let flash_id = flash.read_id();
    info!("FLASH ID back in SPI mode: {=[u8]:x}", flash_id);

    info!("DONE");

    // Output pin PD10 = LD1 (green).
    let mut led = Output::new(p.PD10, Level::Low, Speed::Low);

    loop {
        led.toggle();
        Timer::after_millis(1000).await;
    }
}

const MEMORY_PAGE_SIZE: usize = 256;

/// Implementation of access to flash chip using SPI.
///
/// Chip commands are hardcoded as it depends on used chip.
/// This targets a MX25UW25645GXDI00.
pub struct SpiFlashMemory<I: Instance> {
    xspi: Xspi<'static, I, Blocking>,
}

/// Implementation of access to flash chip using Octo SPI.
///
/// Chip commands are hardcoded as it depends on used chip.
/// This targets a MX25UW25645GXDI00.
pub struct OpiFlashMemory<I: Instance> {
    xspi: Xspi<'static, I, Blocking>,
}

/// SPI mode commands for MX25UW25645G flash memory
#[allow(dead_code)]
#[repr(u8)]
enum SpiCommand {
    // Array access commands
    /// Read data bytes using 3-byte address (up to 50 MHz)
    Read3B = 0x03,
    /// Fast read data bytes using 3-byte address with 8 dummy cycles (up to 133 MHz)
    FastRead3B = 0x0B,
    /// Program 1-256 bytes of data using 3-byte address
    PageProgram3B = 0x02,
    /// Erase 4KB sector using 3-byte address
    SectorErase3B = 0x20,
    /// Erase 64KB block using 3-byte address
    BlockErase3B = 0xD8,
    /// Read data bytes using 4-byte address (up to 50 MHz)
    Read4B = 0x13,
    /// Fast read data bytes using 4-byte address with 8 dummy cycles (up to 133 MHz)
    FastRead4B = 0x0C,
    /// Program 1-256 bytes of data using 4-byte address
    PageProgram4B = 0x12,
    /// Erase 4KB sector using 4-byte address
    SectorErase4B = 0x21,
    /// Erase 64KB block using 4-byte address
    BlockErase4B = 0xDC,
    /// Erase entire chip (only if no blocks are protected)
    ChipErase = 0x60,

    // Write Buffer Access commands
    /// Read data from the 256-byte page buffer
    ReadBuffer = 0x25,
    /// Initialize write-to-buffer sequence, clears buffer and writes initial data
    WriteBufferInitial = 0x22,
    /// Continue writing data to buffer (used between WRBI and WRCF)
    WriteBufferContinue = 0x24,
    /// Confirm write operation, programs buffer contents to flash array
    WriteBufferConfirm = 0x31,

    // Device operation commands
    /// Set Write Enable Latch (WEL) bit, required before write/program/erase operations
    WriteEnable = 0x06,
    /// Clear Write Enable Latch (WEL) bit
    WriteDisable = 0x04,
    /// Select write protection mode (BP mode or Advanced Sector Protection)
    WriteProtectSelection = 0x68,
    /// Suspend ongoing program or erase operation to allow read access
    ProgramEraseSuspend = 0xB0,
    /// Resume suspended program or erase operation
    ProgramEraseResume = 0x30,
    /// Enter deep power-down mode for minimum power consumption
    DeepPowerDown = 0xB9,
    /// Exit deep power-down mode and return to standby
    ReleaseFromDeepPowerDown = 0xAB,
    /// No operation, can terminate Reset Enable command
    NoOperation = 0x00,
    /// Enable reset operation (must precede Reset Memory command)
    ResetEnable = 0x66,
    /// Reset device to power-on state (requires prior Reset Enable)
    ResetMemory = 0x99,
    /// Protect all sectors using Dynamic Protection Bits (DPB)
    GangBlockLock = 0x7E,
    /// Unprotect all sectors by clearing Dynamic Protection Bits (DPB)
    GangBlockUnlock = 0x98,

    // Register Access commands
    /// Read 3-byte device identification (manufacturer ID + device ID)
    ReadIdentification = 0x9F,
    /// Read Serial Flash Discoverable Parameters (SFDP) table
    ReadSFDP = 0x5A,
    /// Read 8-bit Status Register (WIP, WEL, BP bits, etc.)
    ReadStatusRegister = 0x05,
    /// Read 8-bit Configuration Register (ODS, TB, PBE bits)
    ReadConfigurationRegister = 0x15,
    /// Write Status and/or Configuration Register (1-2 bytes)
    WriteStatusConfigurationRegister = 0x01,
    /// Read Configuration Register 2 from specified 4-byte address
    ReadConfigurationRegister2 = 0x71,
    /// Write Configuration Register 2 to specified 4-byte address
    WriteConfigurationRegister2 = 0x72,
    /// Read 8-bit Security Register (protection status, suspend bits)
    ReadSecurityRegister = 0x2B,
    /// Write Security Register to set customer lock-down bit
    WriteSecurityRegister = 0x2F,
    /// Read 32-bit Fast Boot Register (boot address and configuration)
    ReadFastBootRegister = 0x16,
    /// Write 32-bit Fast Boot Register
    WriteFastBootRegister = 0x17,
    /// Erase Fast Boot Register (disable fast boot feature)
    EraseFastBootRegister = 0x18,
    /// Set burst/wrap length for read operations (16/32/64 bytes)
    SetBurstLength = 0xC0,
    /// Enter 8K-bit secured OTP mode for programming unique identifiers
    EnterSecuredOTP = 0xB1,
    /// Exit secured OTP mode and return to main array access
    ExitSecuredOTP = 0xC1,
    /// Write Lock Register to control SPB protection mode
    WriteLockRegister = 0x2C,
    /// Read Lock Register status
    ReadLockRegister = 0x2D,
    /// Program Solid Protection Bit (SPB) for specified sector/block
    WriteSPB = 0xE3,
    /// Erase all Solid Protection Bits (SPB)
    EraseSPB = 0xE4,
    /// Read Solid Protection Bit (SPB) status for specified sector/block
    ReadSPB = 0xE2,
    /// Write Dynamic Protection Bit (DPB) for specified sector
    WriteDPB = 0xE1,
    /// Read Dynamic Protection Bit (DPB) status for specified sector
    ReadDPB = 0xE0,
    /// Read 64-bit password register (only in Solid Protection mode)
    ReadPassword = 0x27,
    /// Write 64-bit password register
    WritePassword = 0x28,
    /// Unlock SPB operations using 64-bit password
    PasswordUnlock = 0x29,
}

/// OPI mode commands for MX25UW25645G flash memory
#[allow(dead_code)]
#[repr(u16)]
enum OpiCommand {
    // Array access commands
    /// Read data using 8 I/O lines in STR mode with configurable dummy cycles (up to 200 MHz)
    OctaRead = 0xEC13,
    /// Read data using 8 I/O lines in DTR mode with configurable dummy cycles (up to 200 MHz)
    OctaDTRRead = 0xEE11,
    /// Program 1-256 bytes using 4-byte address and 8 I/O lines
    PageProgram4B = 0x12ED,
    /// Erase 4KB sector using 4-byte address
    SectorErase4B = 0x21DE,
    /// Erase 64KB block using 4-byte address
    BlockErase4B = 0xDC23,
    /// Erase entire chip (only if no blocks are protected)
    ChipErase = 0x609F,

    // Write Buffer Access commands
    /// Read data from the 256-byte page buffer using 4-byte address
    ReadBuffer = 0x25DA,
    /// Initialize interruptible write-to-buffer sequence with 4-byte address
    WriteBufferInitial = 0x22DD,
    /// Continue writing data to buffer during interruptible sequence
    WriteBufferContinue = 0x24DB,
    /// Confirm and execute write operation from buffer to flash array
    WriteBufferConfirm = 0x31CE,

    // Device operation commands
    /// Set Write Enable Latch (WEL) bit, required before write/program/erase operations
    WriteEnable = 0x06F9,
    /// Clear Write Enable Latch (WEL) bit, aborts write-to-buffer sequence
    WriteDisable = 0x04FB,
    /// Select write protection mode (BP mode or Advanced Sector Protection) - OTP bit
    WriteProtectSelection = 0x6897,
    /// Suspend ongoing program or erase operation to allow read from other banks
    ProgramEraseSuspend = 0xB04F,
    /// Resume suspended program or erase operation
    ProgramEraseResume = 0x30CF,
    /// Enter deep power-down mode for minimum power consumption
    DeepPowerDown = 0xB946,
    /// Exit deep power-down mode and return to standby
    ReleaseFromDeepPowerDown = 0xAB54,
    /// No operation, can terminate Reset Enable command
    NoOperation = 0x00FF,
    /// Enable reset operation (must precede Reset Memory command)
    ResetEnable = 0x6699,
    /// Reset device to power-on state, clears volatile settings
    ResetMemory = 0x9966,
    /// Protect all sectors using Dynamic Protection Bits (DPB)
    GangBlockLock = 0x7E81,
    /// Unprotect all sectors by clearing Dynamic Protection Bits (DPB)
    GangBlockUnlock = 0x9867,

    // Register Access commands
    /// Read 3-byte device identification with 4-byte dummy address
    ReadIdentification = 0x9F60,
    /// Read Serial Flash Discoverable Parameters (SFDP) table with 4-byte address
    ReadSFDP = 0x5AA5,
    /// Read 8-bit Status Register with 4-byte dummy address
    ReadStatusRegister = 0x05FA,
    /// Read 8-bit Configuration Register with specific address (00000001h)
    ReadConfigurationRegister = 0x15EA,
    /// Write 8-bit Status Register with specific address (00000000h) or Configuration Register with address (00000001h)
    WriteStatusConfigurationRegister = 0x01FE,
    /// Read Configuration Register 2 from specified 4-byte address
    ReadConfigurationRegister2 = 0x718E,
    /// Write Configuration Register 2 to specified 4-byte address
    WriteConfigurationRegister2 = 0x728D,
    /// Read 8-bit Security Register with 4-byte dummy address
    ReadSecurityRegister = 0x2BD4,
    /// Write Security Register to set customer lock-down bit
    WriteSecurityRegister = 0x2FD0,
    /// Set burst/wrap length for read operations with 4-byte dummy address
    SetBurstLength = 0xC03F,
    /// Read 32-bit Fast Boot Register with 4-byte dummy address
    ReadFastBootRegister = 0x16E9,
    /// Write 32-bit Fast Boot Register with 4-byte dummy address
    WriteFastBootRegister = 0x17E8,
    /// Erase Fast Boot Register (disable fast boot feature)
    EraseFastBootRegister = 0x18E7,
    /// Enter 8K-bit secured OTP mode for programming unique identifiers
    EnterSecuredOTP = 0xB14E,
    /// Exit secured OTP mode and return to main array access
    ExitSecuredOTP = 0xC13E,
    /// Write Lock Register to control SPB protection mode with 4-byte dummy address
    WriteLockRegister = 0x2CD3,
    /// Read Lock Register status with 4-byte dummy address
    ReadLockRegister = 0x2DD2,
    /// Program Solid Protection Bit (SPB) for specified 4-byte address
    WriteSPB = 0xE31C,
    /// Erase all Solid Protection Bits (SPB)
    EraseSPB = 0xE41B,
    /// Read Solid Protection Bit (SPB) status for specified 4-byte address
    ReadSPB = 0xE21D,
    /// Write Dynamic Protection Bit (DPB) for specified 4-byte address
    WriteDPB = 0xE11E,
    /// Read Dynamic Protection Bit (DPB) status for specified 4-byte address
    ReadDPB = 0xE01F,
    /// Read 64-bit password register with 4-byte dummy address and 20 dummy cycles
    ReadPassword = 0x27D8,
    /// Write 64-bit password register with 4-byte dummy address
    WritePassword = 0x28D7,
    /// Unlock SPB operations using 64-bit password with 4-byte dummy address
    PasswordUnlock = 0x29D6,
}

impl<I: Instance> SpiFlashMemory<I> {
    pub fn new(xspi: Xspi<'static, I, Blocking>) -> Self {
        let mut memory = Self { xspi };

        memory.reset_memory();
        memory
    }

    pub fn disable_mm(&mut self) {
        self.xspi.disable_memory_mapped_mode();
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

    fn into_octo(mut self) -> OpiFlashMemory<I> {
        self.enable_opi_mode();
        OpiFlashMemory { xspi: self.xspi }
    }

    fn enable_opi_mode(&mut self) {
        let cr2_0 = self.read_cr2(0);
        info!("Read CR2 at 0x0: {:x}", cr2_0);
        self.enable_write();
        self.write_cr2(0, cr2_0 | 0x01); // Set bit 0 to enable octo SPI in STR
    }

    fn exec_command(&mut self, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            adwidth: XspiWidth::NONE,
            // adsize: AddressSize::_24bit,
            dwidth: XspiWidth::NONE,
            instruction: Some(cmd as u32),
            address: None,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        // info!("Excuting command: {:x}", transaction.instruction);
        self.xspi.blocking_command(&transaction).unwrap();
    }

    pub fn reset_memory(&mut self) {
        self.exec_command(SpiCommand::ResetEnable as u8);
        self.exec_command(SpiCommand::ResetMemory as u8);
        self.wait_write_finish();
    }

    pub fn enable_write(&mut self) {
        self.exec_command(SpiCommand::WriteEnable as u8);
    }

    pub fn read_id(&mut self) -> [u8; 3] {
        let mut buffer = [0; 3];
        let transaction: TransferConfig = TransferConfig {
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

    fn wait_write_finish(&mut self) {
        while (self.read_sr() & 0x01) != 0 {}
    }

    fn perform_erase(&mut self, addr: u32, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            adwidth: XspiWidth::SING,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::NONE,
            instruction: Some(cmd as u32),
            address: Some(addr),
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.enable_write();
        self.xspi.blocking_command(&transaction).unwrap();
        self.wait_write_finish();
    }

    pub fn erase_sector(&mut self, addr: u32) {
        self.perform_erase(addr, SpiCommand::SectorErase4B as u8);
    }

    pub fn erase_block_64k(&mut self, addr: u32) {
        self.perform_erase(addr, SpiCommand::BlockErase4B as u8);
    }

    pub fn erase_chip(&mut self) {
        self.enable_write();
        self.exec_command(SpiCommand::ChipErase as u8);
        self.wait_write_finish();
    }

    fn write_page(&mut self, addr: u32, buffer: &[u8], len: usize) {
        assert!(
            (len as u32 + (addr & 0x000000ff)) <= MEMORY_PAGE_SIZE as u32,
            "write_page(): page write length exceeds page boundary (len = {}, addr = {:X}",
            len,
            addr
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

    // Note: read_register cannot be used to read the configuration register 2 since there is an
    // address required for that read.
    fn read_register(&mut self, cmd: u8) -> u8 {
        let mut buffer = [0; 1];
        let transaction: TransferConfig = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            adwidth: XspiWidth::NONE,
            dwidth: XspiWidth::SING,
            instruction: Some(cmd as u32),
            address: None,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.xspi.blocking_read(&mut buffer, transaction).unwrap();
        buffer[0]
    }

    pub fn read_sr(&mut self) -> u8 {
        self.read_register(SpiCommand::ReadStatusRegister as u8)
    }

    pub fn read_cr(&mut self) -> u8 {
        self.read_register(SpiCommand::ReadConfigurationRegister as u8)
    }

    pub fn write_sr_cr(&mut self, sr: u8, cr: u8) {
        let buffer = [sr, cr];
        let transaction: TransferConfig = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            instruction: Some(SpiCommand::WriteStatusConfigurationRegister as u32),
            adwidth: XspiWidth::NONE,
            dwidth: XspiWidth::SING,
            address: None,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.enable_write();
        self.xspi.blocking_write(&buffer, transaction).unwrap();
        self.wait_write_finish();
    }

    pub fn read_cr2(&mut self, address: u32) -> u8 {
        let mut buffer = [0; 1];
        let transaction: TransferConfig = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            instruction: Some(SpiCommand::ReadConfigurationRegister2 as u32),
            adsize: AddressSize::_32bit,
            adwidth: XspiWidth::SING,
            dwidth: XspiWidth::SING,
            address: Some(address),
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.xspi.blocking_read(&mut buffer, transaction).unwrap();
        buffer[0]
    }

    pub fn write_cr2(&mut self, address: u32, value: u8) {
        let buffer = [value; 1];
        let transaction: TransferConfig = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            instruction: Some(SpiCommand::WriteConfigurationRegister2 as u32),
            adsize: AddressSize::_32bit,
            adwidth: XspiWidth::SING,
            dwidth: XspiWidth::SING,
            address: Some(address),
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.xspi.blocking_write(&buffer, transaction).unwrap();
        self.wait_write_finish();
    }
}

impl<I: Instance> OpiFlashMemory<I> {
    pub fn into_spi(mut self) -> SpiFlashMemory<I> {
        self.disable_opi_mode();
        SpiFlashMemory { xspi: self.xspi }
    }

    /// Disable OPI mode and return to SPI
    pub fn disable_opi_mode(&mut self) {
        // Clear SOPI and DOPI bits in CR2 volatile register
        let cr2_0 = self.read_cr2(0x00000000);
        self.write_cr2(0x00000000, cr2_0 & 0xFC); // Clear bits 0 and 1
    }

    /// Enable memory-mapped mode for OPI
    pub fn enable_mm(&mut self) {
        let read_config = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit, // 2-byte command for OPI
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::OCTO,
            instruction: Some(OpiCommand::OctaRead as u32),
            dummy: DummyCycles::_20, // Default dummy cycles for OPI
            ..Default::default()
        };

        let write_config = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::OCTO,
            instruction: Some(OpiCommand::PageProgram4B as u32),
            dummy: DummyCycles::_0,
            ..Default::default()
        };

        self.xspi.enable_memory_mapped_mode(read_config, write_config).unwrap();
    }

    pub fn disable_mm(&mut self) {
        self.xspi.disable_memory_mapped_mode();
    }

    /// Execute OPI command (2-byte command)
    fn exec_command(&mut self, cmd: OpiCommand) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit, // 2-byte command
            adwidth: XspiWidth::NONE,
            dwidth: XspiWidth::NONE,
            instruction: Some(cmd as u32),
            address: None,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.xspi.blocking_command(&transaction).unwrap();
    }

    /// Reset memory using OPI commands
    pub fn reset_memory(&mut self) {
        self.exec_command(OpiCommand::ResetEnable);
        self.exec_command(OpiCommand::ResetMemory);
        self.wait_write_finish();
    }

    /// Enable write using OPI command
    pub fn enable_write(&mut self) {
        self.exec_command(OpiCommand::WriteEnable);
    }

    /// Read device ID in OPI mode
    pub fn read_id(&mut self) -> [u8; 3] {
        let mut buffer = [0; 3];
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::OCTO,
            instruction: Some(OpiCommand::ReadIdentification as u32),
            address: Some(0x00000000), // Dummy address required
            dummy: DummyCycles::_4,
            ..Default::default()
        };
        self.xspi.blocking_read(&mut buffer, transaction).unwrap();
        buffer
    }

    /// Read memory using OPI mode
    pub fn read_memory(&mut self, addr: u32, buffer: &mut [u8]) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::OCTO,
            instruction: Some(OpiCommand::OctaRead as u32),
            address: Some(addr),
            dummy: DummyCycles::_20, // Default for 200MHz operation
            ..Default::default()
        };
        self.xspi.blocking_read(buffer, transaction).unwrap();
    }

    /// Wait for write completion using OPI status read
    fn wait_write_finish(&mut self) {
        while (self.read_sr() & 0x01) != 0 {}
    }

    /// Perform erase operation using OPI command
    fn perform_erase(&mut self, addr: u32, cmd: OpiCommand) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::NONE,
            instruction: Some(cmd as u32),
            address: Some(addr),
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.enable_write();
        self.xspi.blocking_command(&transaction).unwrap();
        self.wait_write_finish();
    }

    /// Erase 4KB sector using OPI
    pub fn erase_sector(&mut self, addr: u32) {
        self.perform_erase(addr, OpiCommand::SectorErase4B);
    }

    /// Erase 64KB block using OPI
    pub fn erase_block_64k(&mut self, addr: u32) {
        self.perform_erase(addr, OpiCommand::BlockErase4B);
    }

    /// Erase entire chip using OPI
    pub fn erase_chip(&mut self) {
        self.enable_write();
        self.exec_command(OpiCommand::ChipErase);
        self.wait_write_finish();
    }

    /// Write single page using OPI
    fn write_page(&mut self, addr: u32, buffer: &[u8], len: usize) {
        assert!(
            (len as u32 + (addr & 0x000000ff)) <= MEMORY_PAGE_SIZE as u32,
            "write_page(): page write length exceeds page boundary (len = {}, addr = {:X})",
            len,
            addr
        );

        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::OCTO,
            instruction: Some(OpiCommand::PageProgram4B as u32),
            address: Some(addr),
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.enable_write();
        self.xspi.blocking_write(buffer, transaction).unwrap();
        self.wait_write_finish();
    }

    /// Write memory using OPI (handles page boundaries)
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

    /// Read register using OPI mode
    fn read_register(&mut self, cmd: OpiCommand, dummy_addr: u32, dummy_cycles: DummyCycles) -> u8 {
        let mut buffer = [0; 1];
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::OCTO,
            instruction: Some(cmd as u32),
            address: Some(dummy_addr),
            dummy: dummy_cycles,
            ..Default::default()
        };
        self.xspi.blocking_read(&mut buffer, transaction).unwrap();
        buffer[0]
    }

    /// Read Status Register using OPI
    pub fn read_sr(&mut self) -> u8 {
        self.read_register(
            OpiCommand::ReadStatusRegister,
            0x00000000, // Dummy address
            DummyCycles::_4,
        )
    }

    /// Read Configuration Register using OPI
    pub fn read_cr(&mut self) -> u8 {
        self.read_register(
            OpiCommand::ReadConfigurationRegister,
            0x00000001, // Address for CR
            DummyCycles::_4,
        )
    }

    /// Write Status/Configuration Register using OPI
    pub fn write_sr_cr(&mut self, sr: u8, cr: u8) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::OCTO,
            instruction: Some(OpiCommand::WriteStatusConfigurationRegister as u32),
            address: Some(0x00000000),
            dummy: DummyCycles::_0,
            ..Default::default()
        };

        self.enable_write();
        self.xspi.blocking_write(&[sr, cr], transaction).unwrap();
        self.wait_write_finish();
    }

    /// Read Configuration Register 2 using OPI
    pub fn read_cr2(&mut self, address: u32) -> u8 {
        let mut buffer = [0; 1];
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::OCTO,
            instruction: Some(OpiCommand::ReadConfigurationRegister2 as u32),
            address: Some(address),
            dummy: DummyCycles::_4,
            ..Default::default()
        };
        self.xspi.blocking_read(&mut buffer, transaction).unwrap();
        buffer[0]
    }

    /// Write Configuration Register 2 using OPI
    pub fn write_cr2(&mut self, address: u32, value: u8) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit,
            adwidth: XspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::OCTO,
            instruction: Some(OpiCommand::WriteConfigurationRegister2 as u32),
            address: Some(address),
            dummy: DummyCycles::_0,
            ..Default::default()
        };

        self.enable_write();
        self.xspi.blocking_write(&[value], transaction).unwrap();
        self.wait_write_finish();
    }
}
