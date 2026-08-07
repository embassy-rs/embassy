//! FlexSPI configuration block (FCB) for the iMXRT1060-EVK
//!
//! This uses IS25WP QuadSPI flash.

#![no_std]

use imxrt_boot_gen::flexspi::opcodes::sdr::*;
use imxrt_boot_gen::flexspi::{self, FlashPadType, ReadSampleClockSource, SerialClockFrequency, SerialFlashRegion, *};
use imxrt_boot_gen::serial_flash::*;
pub use nor::ConfigurationBlock;

const SEQ_READ: Sequence = SequenceBuilder::new()
    .instr(Instr::new(CMD, Pads::One, 0xEB))
    .instr(Instr::new(RADDR, Pads::Four, 0x18))
    .instr(Instr::new(DUMMY, Pads::Four, 0x06))
    .instr(Instr::new(READ, Pads::Four, 0x04))
    .build();
const SEQ_READ_STATUS: Sequence = SequenceBuilder::new()
    .instr(Instr::new(CMD, Pads::One, 0x05))
    .instr(Instr::new(READ, Pads::One, 0x04))
    .build();
const SEQ_WRITE_ENABLE: Sequence = SequenceBuilder::new().instr(Instr::new(CMD, Pads::One, 0x06)).build();
const SEQ_ERASE_SECTOR: Sequence = SequenceBuilder::new()
    .instr(Instr::new(CMD, Pads::One, 0x20))
    .instr(Instr::new(RADDR, Pads::One, 0x18))
    .build();
const SEQ_PAGE_PROGRAM: Sequence = SequenceBuilder::new()
    .instr(Instr::new(CMD, Pads::One, 0x02))
    .instr(Instr::new(RADDR, Pads::One, 0x18))
    .instr(Instr::new(WRITE, Pads::One, 0x04))
    .build();
const SEQ_CHIP_ERASE: Sequence = SequenceBuilder::new().instr(Instr::new(CMD, Pads::One, 0x60)).build();

const LUT: LookupTable = LookupTable::new()
    .command(Command::Read, SEQ_READ)
    .command(Command::ReadStatus, SEQ_READ_STATUS)
    .command(Command::WriteEnable, SEQ_WRITE_ENABLE)
    .command(Command::EraseSector, SEQ_ERASE_SECTOR)
    .command(Command::PageProgram, SEQ_PAGE_PROGRAM)
    .command(Command::ChipErase, SEQ_CHIP_ERASE);

const COMMON_CONFIGURATION_BLOCK: flexspi::ConfigurationBlock = flexspi::ConfigurationBlock::new(LUT)
    .version(Version::new(1, 4, 0))
    .read_sample_clk_src(ReadSampleClockSource::LoopbackFromDQSPad)
    .cs_hold_time(3)
    .cs_setup_time(3)
    .controller_misc_options(0x10)
    .serial_flash_pad_type(FlashPadType::Quad)
    .serial_clk_freq(SerialClockFrequency::MHz133)
    .flash_size(SerialFlashRegion::A1, 8 * 1024 * 1024);

pub const SERIAL_NOR_CONFIGURATION_BLOCK: nor::ConfigurationBlock =
    nor::ConfigurationBlock::new(COMMON_CONFIGURATION_BLOCK)
        .page_size(256)
        .sector_size(4096)
        .ip_cmd_serial_clk_freq(nor::SerialClockFrequency::MHz30);

#[unsafe(no_mangle)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), unsafe(link_section = ".fcb"))]
pub static FLEXSPI_CONFIGURATION_BLOCK: nor::ConfigurationBlock = SERIAL_NOR_CONFIGURATION_BLOCK;
