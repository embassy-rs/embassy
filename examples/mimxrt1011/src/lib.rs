//! FlexSPI configuration block (FCB) for iMXRT1011 boards.
//!
//! This is a generic FCB that should work with most QSPI flash.

#![no_std]

use imxrt_boot_gen::flexspi;
use imxrt_boot_gen::flexspi::opcodes::sdr::*;
use imxrt_boot_gen::flexspi::{
    ColumnAddressWidth, Command, DeviceModeConfiguration, FlashPadType, Instr, LookupTable, Pads,
    ReadSampleClockSource, Sequence, SequenceBuilder, SerialClockFrequency, SerialFlashRegion,
    WaitTimeConfigurationCommands,
};
use imxrt_boot_gen::serial_flash::nor;

/// While the IMXRT1010-EVK and Makerdiary iMX RT1011 Nano Kit have 128MBit of flash we limit to 64Mbit
/// to allow the Metro M7 boards to use the same FCB configuration.
const DENSITY_BITS: u32 = 64 * 1024 * 1024;
const DENSITY_BYTES: u32 = DENSITY_BITS / 8;

const SEQ_READ: Sequence = SequenceBuilder::new()
    .instr(Instr::new(CMD, Pads::One, 0xEB))
    .instr(Instr::new(RADDR, Pads::Four, 0x18))
    .instr(Instr::new(DUMMY, Pads::Four, 0x06))
    .instr(Instr::new(READ, Pads::Four, 0x04))
    .build();

const SEQ_READ_STATUS: Sequence = SequenceBuilder::new()
    .instr(Instr::new(CMD, Pads::One, 0x05))
    .instr(Instr::new(READ, Pads::One, 0x01))
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
    .read_sample_clk_src(ReadSampleClockSource::LoopbackFromDQSPad)
    .cs_hold_time(0x03)
    .cs_setup_time(0x03)
    .column_address_width(ColumnAddressWidth::OtherDevices)
    .device_mode_configuration(DeviceModeConfiguration::Disabled)
    .wait_time_cfg_commands(WaitTimeConfigurationCommands::disable())
    .flash_size(SerialFlashRegion::A1, DENSITY_BYTES)
    .serial_clk_freq(SerialClockFrequency::MHz120)
    .serial_flash_pad_type(FlashPadType::Quad);

pub const SERIAL_NOR_CONFIGURATION_BLOCK: nor::ConfigurationBlock =
    nor::ConfigurationBlock::new(COMMON_CONFIGURATION_BLOCK)
        .page_size(256)
        .sector_size(4096)
        .ip_cmd_serial_clk_freq(nor::SerialClockFrequency::MHz30);

#[unsafe(no_mangle)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), unsafe(link_section = ".fcb"))]
pub static FLEXSPI_CONFIGURATION_BLOCK: nor::ConfigurationBlock = SERIAL_NOR_CONFIGURATION_BLOCK;
