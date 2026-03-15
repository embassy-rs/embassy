//! RAM-resident flash operations for mark_booted().
//!
//! Since the app runs from memory-mapped XSPI flash, we cannot use the normal
//! embassy XSPI driver to write to flash (it would conflict with code execution).
//! Instead, this module provides a self-contained function placed in RAM that
//! uses raw XSPI2 register writes to:
//! 1. Exit memory-mapped mode
//! 2. Erase and write the bootloader state partition
//! 3. Re-enable memory-mapped mode
//!
//! All code in this module must be in RAM (via #[link_section = ".data"]) and
//! must not call any functions that reside in flash.

// XSPI2 base address
const XSPI2_BASE: u32 = 0x4802_a000;

// XSPI register offsets
const CR: u32 = 0x00; // Control Register
const SR: u32 = 0x20; // Status Register
const FCR: u32 = 0x24; // Flag Clear Register
const DLR: u32 = 0x40; // Data Length Register
const AR: u32 = 0x48; // Address Register
const DR: u32 = 0x50; // Data Register
const CCR: u32 = 0x100; // Communication Configuration Register
const TCR: u32 = 0x108; // Timing Configuration Register
const IR: u32 = 0x110; // Instruction Register
const WPCCR: u32 = 0x140; // Wrap Communication Configuration Register
const WPTCR: u32 = 0x148; // Wrap Timing Configuration Register
const WPIR: u32 = 0x150; // Wrap Instruction Register

// CR register bits
const CR_ABORT: u32 = 1 << 1;
const CR_FMODE_SHIFT: u32 = 28;
const CR_FMODE_MASK: u32 = 0x3 << CR_FMODE_SHIFT;
const CR_FMODE_INDIRECT_WRITE: u32 = 0x0 << CR_FMODE_SHIFT;
const CR_FMODE_INDIRECT_READ: u32 = 0x1 << CR_FMODE_SHIFT;
const CR_FMODE_MEMORY_MAPPED: u32 = 0x3 << CR_FMODE_SHIFT;

// SR register bits
const SR_BUSY: u32 = 1 << 5;
const SR_TCF: u32 = 1 << 1; // Transfer Complete Flag
const SR_FTF: u32 = 1 << 2; // FIFO Threshold Flag

// FCR register bits
const FCR_CTCF: u32 = 1 << 1; // Clear Transfer Complete Flag

// SPI flash commands
const CMD_WRITE_ENABLE: u32 = 0x06;
const CMD_READ_STATUS: u32 = 0x05;
const CMD_SECTOR_ERASE_4B: u32 = 0x21;
const CMD_PAGE_PROGRAM_4B: u32 = 0x12;
const CMD_FAST_READ_4B: u32 = 0x0C;
const CMD_RESET_ENABLE: u32 = 0x66;
const CMD_RESET_MEMORY: u32 = 0x99;

// CCR field values for single-SPI mode
// IMODE=1 (single line), ADMODE=0/1, DMODE=0/1, ADSIZE, ISIZE
const CCR_IMODE_SINGLE: u32 = 0x1; // bits [2:0]
const CCR_ADMODE_SINGLE: u32 = 0x1 << 8; // bits [10:8]
const CCR_ADMODE_NONE: u32 = 0x0 << 8;
const CCR_ADSIZE_32BIT: u32 = 0x3 << 12; // bits [13:12]
const CCR_DMODE_SINGLE: u32 = 0x1 << 24; // bits [26:24]
const CCR_DMODE_NONE: u32 = 0x0 << 24;

// TCR field: 8 dummy cycles for fast read
const TCR_DCYC_8: u32 = 8; // bits [4:0]

// Bootloader state
const STATE_OFFSET: u32 = 0x500000; // STATE partition in external flash
const STATE_SIZE: u32 = 0x3000; // 12K = 3 sectors
const SECTOR_SIZE: u32 = 4096;
const BOOT_MAGIC: u8 = 0xD0;

#[inline(always)]
fn reg_write(offset: u32, val: u32) {
    unsafe { core::ptr::write_volatile((XSPI2_BASE + offset) as *mut u32, val) };
}

#[inline(always)]
fn reg_read(offset: u32) -> u32 {
    unsafe { core::ptr::read_volatile((XSPI2_BASE + offset) as *const u32) }
}

#[inline(always)]
fn reg_modify(offset: u32, clear: u32, set: u32) {
    let val = reg_read(offset);
    reg_write(offset, (val & !clear) | set);
}

#[inline(always)]
fn wait_not_busy() {
    while (reg_read(SR) & SR_BUSY) != 0 {}
}

#[inline(always)]
fn wait_tcf() {
    while (reg_read(SR) & SR_TCF) == 0 {}
    reg_write(FCR, FCR_CTCF);
}

/// Send a command-only SPI transaction (no address, no data)
#[inline(always)]
fn send_command(cmd: u32) {
    wait_not_busy();

    // Set functional mode to indirect-write
    reg_modify(CR, CR_FMODE_MASK, CR_FMODE_INDIRECT_WRITE);

    // CCR: IMODE=single, no address, no data
    reg_write(CCR, CCR_IMODE_SINGLE | CCR_ADMODE_NONE | CCR_DMODE_NONE);
    // TCR: no dummy cycles
    reg_write(TCR, 0);
    // DLR: no data
    reg_write(DLR, 0);
    // IR: instruction triggers the transfer
    reg_write(IR, cmd);

    wait_tcf();
}

/// Read the flash status register and return the value
#[inline(always)]
fn read_status_register() -> u8 {
    wait_not_busy();

    // Set functional mode to indirect-read
    reg_modify(CR, CR_FMODE_MASK, CR_FMODE_INDIRECT_READ);

    // CCR: IMODE=single, no address, DMODE=single
    reg_write(CCR, CCR_IMODE_SINGLE | CCR_ADMODE_NONE | CCR_DMODE_SINGLE);
    reg_write(TCR, 0);
    reg_write(DLR, 0); // 1 byte (DLR = number_of_bytes - 1)
    // IR: triggers the transfer
    reg_write(IR, CMD_READ_STATUS);

    // Wait for data
    while (reg_read(SR) & SR_FTF) == 0 {}
    let val = reg_read(DR) as u8;

    wait_tcf();
    val
}

/// Wait for flash write/erase to complete (WIP bit in status register)
#[inline(always)]
fn wait_flash_ready() {
    loop {
        let sr = read_status_register();
        if (sr & 0x01) == 0 {
            break;
        }
    }
}

/// Send Write Enable command
#[inline(always)]
fn write_enable() {
    send_command(CMD_WRITE_ENABLE);
}

/// Erase a 4K sector at the given flash address
#[inline(always)]
fn erase_sector(addr: u32) {
    write_enable();
    wait_not_busy();

    reg_modify(CR, CR_FMODE_MASK, CR_FMODE_INDIRECT_WRITE);
    // CCR: IMODE=single, ADMODE=single, ADSIZE=32bit, no data
    reg_write(
        CCR,
        CCR_IMODE_SINGLE | CCR_ADMODE_SINGLE | CCR_ADSIZE_32BIT | CCR_DMODE_NONE,
    );
    reg_write(TCR, 0);
    reg_write(DLR, 0);
    // Write address first, then instruction triggers the command
    reg_write(AR, addr);
    reg_write(IR, CMD_SECTOR_ERASE_4B);

    wait_tcf();
    wait_flash_ready();
}

/// Write up to 4 bytes to flash at the given address
#[inline(always)]
fn write_bytes(addr: u32, data: &[u8]) {
    write_enable();
    wait_not_busy();

    reg_modify(CR, CR_FMODE_MASK, CR_FMODE_INDIRECT_WRITE);
    // CCR: IMODE=single, ADMODE=single, ADSIZE=32bit, DMODE=single
    reg_write(
        CCR,
        CCR_IMODE_SINGLE | CCR_ADMODE_SINGLE | CCR_ADSIZE_32BIT | CCR_DMODE_SINGLE,
    );
    reg_write(TCR, 0);
    reg_write(DLR, (data.len() - 1) as u32);
    reg_write(AR, addr);
    reg_write(IR, CMD_PAGE_PROGRAM_4B);

    // Write data bytes (pack into u32)
    let mut word: u32 = 0;
    for (i, &b) in data.iter().enumerate() {
        word |= (b as u32) << (i * 8);
    }
    reg_write(DR, word);

    wait_tcf();
    wait_flash_ready();
}

/// Exit memory-mapped mode by aborting and disabling XSPI
#[inline(always)]
fn exit_memory_mapped_mode() {
    // Abort any ongoing operation
    reg_modify(CR, 0, CR_ABORT);
    // Wait for abort to complete
    while (reg_read(CR) & CR_ABORT) != 0 {}
    wait_not_busy();
}

/// Re-enable memory-mapped mode with the same config the FSBL uses
#[inline(always)]
fn enter_memory_mapped_mode() {
    wait_not_busy();

    // Configure read command: FastRead4B, single-line, 32-bit address, 8 dummy cycles
    reg_write(
        CCR,
        CCR_IMODE_SINGLE | CCR_ADMODE_SINGLE | CCR_ADSIZE_32BIT | CCR_DMODE_SINGLE,
    );
    reg_write(TCR, TCR_DCYC_8);
    reg_write(IR, CMD_FAST_READ_4B);

    // Configure write command for memory-mapped mode
    reg_write(
        WPCCR,
        CCR_IMODE_SINGLE | CCR_ADMODE_SINGLE | CCR_ADSIZE_32BIT | CCR_DMODE_SINGLE,
    );
    reg_write(WPTCR, 0);
    reg_write(WPIR, CMD_PAGE_PROGRAM_4B);

    // Set functional mode to memory-mapped
    reg_modify(CR, CR_FMODE_MASK, CR_FMODE_MEMORY_MAPPED);
}

/// Mark the current firmware boot as successful.
///
/// This function is placed in RAM and temporarily exits XSPI memory-mapped mode
/// to write BOOT_MAGIC to the bootloader state partition in external flash.
/// All interrupts are disabled during this operation since the vector table
/// is in flash and cannot be accessed while XSPI is in indirect mode.
///
/// # Safety
/// Must only be called when XSPI2 is in memory-mapped mode and the flash
/// is properly initialized.
#[unsafe(link_section = ".data")]
#[inline(never)]
pub fn mark_booted() {
    unsafe {
        // Disable all interrupts — vector table is in flash
        cortex_m::interrupt::disable();

        // Exit memory-mapped mode
        exit_memory_mapped_mode();

        // Reset flash to known state
        send_command(CMD_RESET_ENABLE);
        send_command(CMD_RESET_MEMORY);
        wait_flash_ready();

        // Erase STATE partition (3 sectors × 4K = 12K)
        let mut addr = STATE_OFFSET;
        while addr < STATE_OFFSET + STATE_SIZE {
            erase_sector(addr);
            addr += SECTOR_SIZE;
        }

        // Write BOOT_MAGIC (0xD0) — 4 bytes at start of STATE
        let magic = [BOOT_MAGIC; 4];
        write_bytes(STATE_OFFSET, &magic);

        // Re-enable memory-mapped mode
        enter_memory_mapped_mode();

        // Invalidate instruction cache since flash content may have changed
        let mut p = cortex_m::Peripherals::steal();
        p.SCB.invalidate_icache();

        // Re-enable interrupts
        cortex_m::interrupt::enable();
    }
}
