//! XSPI NOR Flash driver for Macronix NOR flash on STM32N6 boards
//! - MX66UW1G45G (128MB) on STM32N6570-DK
//! - MX25UM51245G (64MB) on NUCLEO-N657X0-Q
//!
//! Adapted from examples/stm32n6/src/bin/xspi_flash.rs

use core::cmp::min;

use defmt::info;
use embassy_stm32::mode::Blocking;
use embassy_stm32::pac::XSPI2;
use embassy_stm32::pac::xspi::vals::{CcrAbmode, CcrAdmode, CcrDmode, CcrImode, CcrIsize};
use embassy_stm32::xspi::{AddressSize, DummyCycles, Instance, TransferConfig, Xspi, XspiWidth};

const MEMORY_PAGE_SIZE: usize = 256;

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

/// Abort any in-progress XSPI transfer and recover the peripheral.
/// Matches the embassy driver pattern (mod.rs:266-270) plus clears CCR
/// to prevent stale mode settings from affecting the next command.
fn abort_and_recover() {
    XSPI2.cr().modify(|w| {
        w.set_abort(true);
        w.set_en(false);
    });
    XSPI2.fcr().write(|w| {
        w.set_ctcf(true);
        w.set_ctef(true);
    });
    // Clear CCR to prevent stale mode settings from affecting next command
    XSPI2.ccr().write(|_| {});
    XSPI2.cr().modify(|w| w.set_en(true));
}

/// Send an OPI reset command via PAC directly with a timeout.
///
/// The embassy XSPI driver's `blocking_command()` polls TCF in an infinite loop.
/// When `delay_hold_quarter_cycle` is enabled and the flash isn't in OPI mode,
/// the peripheral hangs forever. This function bypasses the driver, configures
/// CCR manually (DDTR=false, DQSE=false), and aborts on timeout — matching
/// how ST's HAL handles reset failures with a 500ms timeout.
///
/// Returns `true` if the command completed, `false` on timeout (expected when
/// the flash is not in the targeted OPI mode).
fn try_opi_reset(cmd: u8, dtr: bool) -> bool {
    // Macronix OPI: 16-bit instruction = command byte + bitwise complement
    let instruction = ((cmd as u32) << 8) | ((!cmd) as u32 & 0xFF);

    // Wait for peripheral not busy (with timeout in case it's already wedged)
    for _ in 0..100_000u32 {
        if !XSPI2.sr().read().busy() {
            break;
        }
    }

    // Clear any pending flags
    XSPI2.fcr().write(|w| {
        w.set_ctcf(true);
        w.set_ctef(true);
    });

    // Configure CCR: instruction-only, OCTO mode, 16-bit, no DDTR/DQSE
    // This avoids the driver's auto-enable of DDTR which causes the hang.
    XSPI2.ccr().write(|w| {
        w.set_imode(CcrImode::B_0X4); // OCTO
        w.set_isize(CcrIsize::B_0X1); // 16-bit
        w.set_idtr(dtr);
        w.set_admode(CcrAdmode::B_0X0); // NONE
        w.set_abmode(CcrAbmode::B_0X0); // NONE
        w.set_dmode(CcrDmode::B_0X0); // NONE
        w.set_ddtr(false);
        w.set_dqse(false);
    });

    // Write instruction — triggers the transfer (no address/data phase)
    XSPI2.ir().write(|v| v.set_instruction(instruction));

    // Poll TCF with timeout (~100k iterations ≈ several ms at 64MHz)
    for _ in 0..100_000u32 {
        let sr = XSPI2.sr().read();
        if sr.tcf() {
            XSPI2.fcr().write(|w| w.set_ctcf(true));
            return true;
        }
        if sr.tef() {
            XSPI2.fcr().write(|w| w.set_ctef(true));
            return false;
        }
    }

    abort_and_recover();
    false
}

/// Send a SPI single-line command via PAC with timeout.
/// Used for SPI reset commands after OPI abort cycles may have left the
/// peripheral in an inconsistent state. Avoids the embassy driver's
/// infinite TCF poll loop.
fn try_spi_command(cmd: u8) -> bool {
    for _ in 0..100_000u32 {
        if !XSPI2.sr().read().busy() {
            break;
        }
    }

    XSPI2.fcr().write(|w| {
        w.set_ctcf(true);
        w.set_ctef(true);
    });

    XSPI2.ccr().write(|w| {
        w.set_imode(CcrImode::B_0X1); // SING
        w.set_isize(CcrIsize::B_0X0); // 8-bit
        w.set_idtr(false);
        w.set_admode(CcrAdmode::B_0X0);
        w.set_abmode(CcrAbmode::B_0X0);
        w.set_dmode(CcrDmode::B_0X0);
        w.set_ddtr(false);
        w.set_dqse(false);
    });

    XSPI2.ir().write(|v| v.set_instruction(cmd as u32));

    for _ in 0..100_000u32 {
        let sr = XSPI2.sr().read();
        if sr.tcf() {
            XSPI2.fcr().write(|w| w.set_ctcf(true));
            return true;
        }
        if sr.tef() {
            XSPI2.fcr().write(|w| w.set_ctef(true));
            return false;
        }
    }

    abort_and_recover();
    false
}

/// SPI Flash Memory driver for MX66UW1G45G
pub struct SpiFlashMemory<I: Instance> {
    xspi: Xspi<'static, I, Blocking>,
}

impl<I: Instance> SpiFlashMemory<I> {
    pub fn new(xspi: Xspi<'static, I, Blocking>) -> Self {
        let mut memory = Self { xspi };
        memory.reset_memory();
        memory
    }

    pub fn reset_memory(&mut self) {
        // Send reset in all three modes — the boot ROM may leave the flash in OPI mode
        // (observed on Nucleo-N657). Commands in the wrong mode time out and are skipped.
        //
        // OPI resets use PAC directly with a timeout to avoid the embassy XSPI driver's
        // infinite TCF poll loop, which hangs when DHQC is enabled and flash isn't in OPI mode.

        // 1. OPI-DTR reset
        info!("reset_memory: sending OPI-DTR reset");
        let ok1 = try_opi_reset(SpiCommand::ResetEnable as u8, true);
        let ok2 = try_opi_reset(SpiCommand::ResetMemory as u8, true);
        if ok1 && ok2 {
            info!("  OPI-DTR: ok");
        } else {
            info!("  OPI-DTR: timeout (normal if flash not in this mode)");
        }

        // 2. OPI-STR reset
        info!("reset_memory: sending OPI-STR reset");
        let ok1 = try_opi_reset(SpiCommand::ResetEnable as u8, false);
        let ok2 = try_opi_reset(SpiCommand::ResetMemory as u8, false);
        if ok1 && ok2 {
            info!("  OPI-STR: ok");
        } else {
            info!("  OPI-STR: timeout (normal if flash not in this mode)");
        }

        // 3. SPI single-line reset (also via PAC with timeout — after OPI abort cycles
        // the peripheral may be in an inconsistent state causing the driver to hang)
        info!("reset_memory: sending SPI reset");
        let ok1 = try_spi_command(SpiCommand::ResetEnable as u8);
        let ok2 = try_spi_command(SpiCommand::ResetMemory as u8);
        if ok1 && ok2 {
            info!("  SPI: ok");
        } else {
            info!("  SPI: timeout");
        }

        // Wait for reset recovery (tRST ~30μs typical, use ~200μs margin at 64MHz)
        info!("reset_memory: waiting for reset recovery");
        cortex_m::asm::delay(12_800);

        self.wait_write_finish();
        info!("reset_memory: done");
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
}
