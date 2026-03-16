//! XSPI NOR Flash driver for Macronix NOR flash on STM32N6 boards
//! - MX66UW1G45G (128MB) on STM32N6570-DK
//! - MX25UM51245G (64MB) on NUCLEO-N657X0-Q
//!
//! Adapted from examples/stm32n6/src/bin/xspi_flash.rs

use core::cmp::min;

use embassy_stm32::mode::Blocking;
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
        // (observed on Nucleo-N657). Commands in the wrong mode are silently ignored.

        // 1. OPI-DTR reset
        self.exec_command_opi_dtr(SpiCommand::ResetEnable as u8);
        self.exec_command_opi_dtr(SpiCommand::ResetMemory as u8);

        // 2. OPI-STR reset
        self.exec_command_opi_str(SpiCommand::ResetEnable as u8);
        self.exec_command_opi_str(SpiCommand::ResetMemory as u8);

        // 3. SPI single-line reset
        self.exec_command(SpiCommand::ResetEnable as u8);
        self.exec_command(SpiCommand::ResetMemory as u8);

        // Wait for reset recovery (tRST ~30μs typical, use ~200μs margin at 64MHz)
        cortex_m::asm::delay(12_800);

        self.wait_write_finish();
    }

    fn exec_command_opi_dtr(&mut self, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            idtr: true,
            adwidth: XspiWidth::NONE,
            dwidth: XspiWidth::NONE,
            instruction: Some(cmd as u32),
            address: None,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        let _ = self.xspi.blocking_command(&transaction);
    }

    fn exec_command_opi_str(&mut self, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::OCTO,
            idtr: false,
            adwidth: XspiWidth::NONE,
            dwidth: XspiWidth::NONE,
            instruction: Some(cmd as u32),
            address: None,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        let _ = self.xspi.blocking_command(&transaction);
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
