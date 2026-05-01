use embassy_mcxa::flexspi::lookup::opcodes::sdr::{CMD, DUMMY, MODE8, RADDR, READ, WRITE};
use embassy_mcxa::flexspi::lookup::{Command, Instr, LookupTable, Pads, SequenceBuilder};
use embassy_mcxa::flexspi::{DeviceCommand, FlashConfig};

pub const FLASH_PAGE_SIZE: usize = 256;
pub const FLASH_SECTOR_SIZE: usize = 4096;

/// Number of contiguous sectors the self-test will erase, program, and verify.
pub const SELF_TEST_SECTORS: u32 = 4;
/// Total byte span exercised by the self-test.
pub const SELF_TEST_BYTES: u32 = SELF_TEST_SECTORS * FLASH_SECTOR_SIZE as u32;
/// Number of pages in the self-test span.
pub const SELF_TEST_PAGES: u32 = SELF_TEST_BYTES / FLASH_PAGE_SIZE as u32;

/// Lengths used to exercise the read path with a variety of (mostly) odd sizes.
/// They include sub-byte-bus, sub-FIFO, full page, and across-page values.
pub const READ_LEN_PROBES: &[usize] = &[1, 3, 7, 17, 33, 63, 127, 256, 257, 511, 1024];

/// Deterministic pseudo-random byte for a given flash address.
///
/// Picks bytes with a Knuth multiplicative hash so that adjacent addresses
/// look uncorrelated; this catches off-by-one address arithmetic that an
/// `i as u8` ramp would silently mask (since 256 = page size).
#[inline]
pub fn pattern_byte(address: u32) -> u8 {
    let h = address.wrapping_mul(2_654_435_761);
    (h ^ (h >> 16)) as u8
}

pub fn fill_pattern(base: u32, buffer: &mut [u8]) {
    for (i, b) in buffer.iter_mut().enumerate() {
        *b = pattern_byte(base + i as u32);
    }
}

/// Returns the absolute address of the first mismatch, or `None` on match.
pub fn check_pattern(base: u32, buffer: &[u8]) -> Option<u32> {
    for (i, b) in buffer.iter().enumerate() {
        if *b != pattern_byte(base + i as u32) {
            return Some(base + i as u32);
        }
    }
    None
}

/// Returns the absolute address of the first non-`0xFF` byte, or `None`.
pub fn check_erased(base: u32, buffer: &[u8]) -> Option<u32> {
    for (i, b) in buffer.iter().enumerate() {
        if *b != 0xff {
            return Some(base + i as u32);
        }
    }
    None
}

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
