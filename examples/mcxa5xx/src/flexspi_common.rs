use embassy_mcxa::flexspi::lookup::opcodes::sdr::{CMD, DUMMY, MODE8, RADDR, READ, WRITE};
use embassy_mcxa::flexspi::lookup::{Command, Instr, LookupTable, Pads, SequenceBuilder};
use embassy_mcxa::flexspi::{DeviceCommand, FlashConfig};

pub const FLASH_PAGE_SIZE: usize = 256;

const ENTER_OPI_SEQ: u8 = Command::WriteStatus as u8;

pub const FLASH_CONFIG: FlashConfig = FlashConfig {
    flash_size_kbytes: 0x10000,
    page_size: FLASH_PAGE_SIZE,
    busy_status_polarity: true,
    busy_status_offset: 0,
    lookup_table: LookupTable::new()
        .command(
            Command::Read,
            SequenceBuilder::new()
                .instr(Instr::new(CMD, Pads::One, 0xEB))
                .instr(Instr::new(RADDR, Pads::Four, 0x18))
                .instr(Instr::new(MODE8, Pads::Four, 0xF0))
                .instr(Instr::new(DUMMY, Pads::Four, 0x04))
                .instr(Instr::new(READ, Pads::Four, 0x00))
                .build(),
        )
        .command(
            Command::ReadStatus,
            SequenceBuilder::new()
                .instr(Instr::new(CMD, Pads::One, 0x05))
                .instr(Instr::new(READ, Pads::One, 0x00))
                .build(),
        )
        .command(
            Command::WriteEnable,
            SequenceBuilder::new().instr(Instr::new(CMD, Pads::One, 0x06)).build(),
        )
        .command(
            Command::ReadId,
            SequenceBuilder::new()
                .instr(Instr::new(CMD, Pads::One, 0x9F))
                .instr(Instr::new(READ, Pads::One, 0x00))
                .build(),
        )
        .command(
            Command::EraseSector,
            SequenceBuilder::new()
                .instr(Instr::new(CMD, Pads::One, 0x20))
                .instr(Instr::new(RADDR, Pads::One, 0x18))
                .build(),
        )
        .command(
            Command::PageProgram,
            SequenceBuilder::new()
                .instr(Instr::new(CMD, Pads::One, 0x02))
                .instr(Instr::new(RADDR, Pads::One, 0x18))
                .instr(Instr::new(WRITE, Pads::One, 0x00))
                .build(),
        )
        .command(
            Command::WriteStatus,
            SequenceBuilder::new()
                .instr(Instr::new(CMD, Pads::One, 0x05))
                .instr(Instr::new(READ, Pads::One, 0x00))
                .build(),
        ),
    read_seq: Command::Read as u8,
    read_status_seq: Command::ReadStatus as u8,
    write_enable_seq: Command::WriteEnable as u8,
    read_id_seq: Command::ReadId as u8,
    erase_sector_seq: Command::EraseSector as u8,
    page_program_seq: Command::PageProgram as u8,
    reset_sequence: Some(
        SequenceBuilder::new()
            .instr(Instr::new(CMD, Pads::One, 0x66))
            .instr(Instr::new(CMD, Pads::One, 0x99))
            .build(),
    ),
    device_mode_command: Some(DeviceCommand::new(ENTER_OPI_SEQ, [0xE7, 0, 0, 0], 1, true)),
};
