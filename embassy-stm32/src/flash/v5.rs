//! Flash controller v5: STM32H7, H7A/B, H7R/S, H5 and C5.
//!
//! Programming fills a write buffer of one flash word (256 bits on H7, 128 bits on the
//! others), which is programmed when full. Sectors are erased by number. The status flags
//! are read-only and are cleared through a separate clear register.
//!
//! H7 and H7A/B have one register set per bank; with the banks swapped, the register sets
//! are swapped too, so the set is picked by address. H5 and C5 have a single register set and
//! select the physical bank with `BKSEL`. H5 is driven through its non-secure registers.

use core::ptr::write_volatile;

#[allow(unused_imports)]
use super::{Config, Error, FlashBank, FlashSector, WRITE_SIZE};
use crate::pac;
#[cfg(any(flash_v5_h5, flash_v5_h50, flash_v5_c5))]
use crate::pac::flash::vals::Bksel;
#[cfg(flash_v5_c5)]
use crate::pac::flash::vals::Edatasel;

const KEY1: u32 = 0x4567_0123;
const KEY2: u32 = 0xCDEF_89AB;

/// Register sets: one per bank on dual-bank H7, a single one otherwise.
#[cfg(all(any(flash_v5_h7, flash_v5_h7ab), flash_dual_bank))]
const SETS: usize = 2;
#[cfg(not(all(any(flash_v5_h7, flash_v5_h7ab), flash_dual_bank)))]
const SETS: usize = 1;

/// Register `$r` of register set `$set`.
#[cfg(any(flash_v5_h7, flash_v5_h7ab))]
macro_rules! reg {
    ($set:expr, $r:ident) => {
        pac::FLASH.bank($set).$r()
    };
}
#[cfg(any(flash_v5_h7rs, flash_v5_c5))]
macro_rules! reg {
    ($set:expr, $r:ident) => {{
        let _ = $set;
        pac::FLASH.$r()
    }};
}
#[cfg(any(flash_v5_h5, flash_v5_h50))]
macro_rules! reg {
    ($set:expr, keyr) => {
        reg!(@ $set, nskeyr)
    };
    ($set:expr, cr) => {
        reg!(@ $set, nscr)
    };
    ($set:expr, sr) => {
        reg!(@ $set, nssr)
    };
    ($set:expr, ccr) => {
        reg!(@ $set, nsccr)
    };
    (@ $set:expr, $r:ident) => {{
        let _ = $set;
        pac::FLASH.$r()
    }};
}

/// Register set controlling the bank mapped at `address`.
fn set_of(address: u32) -> usize {
    #[cfg(all(any(flash_v5_h7, flash_v5_h7ab), flash_dual_bank))]
    if address >= super::FLASH_BANK2_BASE {
        return 1;
    }
    let _ = address;
    0
}

pub(super) unsafe fn unlock() {
    for set in 0..SETS {
        // Unlocking an already unlocked register set locks it until reset.
        if reg!(set, cr).read().lock() {
            reg!(set, keyr).write_value(KEY1);
            reg!(set, keyr).write_value(KEY2);
        }
    }
}

pub(super) unsafe fn lock() {
    for set in 0..SETS {
        reg!(set, cr).modify(|w| w.set_lock(true));
    }
}

fn set_interrupts(set: usize, enable: bool) {
    // H7R/S has the interrupt enables in a separate register.
    #[cfg(flash_v5_h7rs)]
    let r = reg!(set, ier);
    #[cfg(not(flash_v5_h7rs))]
    let r = reg!(set, cr);
    r.modify(|w| {
        w.set_eopie(enable);
        w.set_wrperrie(enable);
        w.set_pgserrie(enable);
        w.set_strberrie(enable);
        w.set_incerrie(enable);
        #[cfg(flash_v5_h7)]
        w.set_operrie(enable);
    });
}

fn start_program(set: usize, interrupt: bool) {
    reg!(set, cr).modify(|w| {
        w.set_pg(true);
        #[cfg(flash_v5_h7)]
        w.set_psize(2); // x32
    });
    set_interrupts(set, interrupt);
}

pub(super) unsafe fn start_write(address: u32, data: &[u8; WRITE_SIZE], interrupt: bool) {
    start_program(set_of(address), interrupt);
    // The flash word is programmed once the write buffer is full.
    let mut address = address;
    for word in data.chunks_exact(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(word.try_into())));
        address += 4;
    }
    // Make sure the stores reached the flash interface before polling its status.
    cortex_m::asm::dsb();
}

pub(super) unsafe fn start_erase(sector: &FlashSector, config: &Config, interrupt: bool) {
    let _ = config;
    let set = set_of(sector.start);
    reg!(set, cr).modify(|w| {
        w.set_per(true);
        w.set_pnb(sector.index_in_bank as u8);
        #[cfg(flash_v5_h7)]
        w.set_psize(match config.erase_parallelism {
            crate::flash::EraseParallelism::X8 => 0,
            crate::flash::EraseParallelism::X16 => 1,
            crate::flash::EraseParallelism::X32 => 2,
            crate::flash::EraseParallelism::X64 => 3,
        });
        #[cfg(any(flash_v5_h5, flash_v5_h50, flash_v5_c5))]
        w.set_bksel(bksel(sector.bank));
    });
    set_interrupts(set, interrupt);
    reg!(set, cr).modify(|w| w.set_strt(true));
    cortex_m::asm::dsb();
}

#[cfg(any(flash_v5_h5, flash_v5_h50, flash_v5_c5))]
fn bksel(bank: FlashBank) -> Bksel {
    match bank {
        FlashBank::Bank2 => Bksel::Bank2,
        _ => Bksel::Bank1,
    }
}

pub(super) fn busy() -> bool {
    (0..SETS).any(|set| {
        let sr = reg!(set, sr).read();
        #[cfg(any(flash_v5_h7, flash_v5_h7ab))]
        return sr.bsy() || sr.qw() || sr.wbne();
        // RM0477: poll QW rather than BSY, which can pulse several times per operation.
        #[cfg(flash_v5_h7rs)]
        return sr.qw() || sr.wbne();
        #[cfg(any(flash_v5_h5, flash_v5_h50, flash_v5_c5))]
        return sr.bsy() || sr.wbne() || sr.dbne();
    })
}

pub(super) fn wait_idle() {
    while busy() {}
}

pub(super) fn finish() -> Result<(), Error> {
    let mut res = Ok(());
    for set in 0..SETS {
        // H7R/S has the error flags in ISR/ICR instead of SR/CCR.
        #[cfg(flash_v5_h7rs)]
        let (sr, ccr) = (reg!(set, isr), reg!(set, icr));
        #[cfg(not(flash_v5_h7rs))]
        let (sr, ccr) = (reg!(set, sr), reg!(set, ccr));

        let sr = sr.read();
        ccr.write(|w| {
            w.set_clr_eop(true);
            w.set_clr_wrperr(true);
            w.set_clr_pgserr(true);
            w.set_clr_strberr(true);
            w.set_clr_incerr(true);
            #[cfg(flash_v5_h7)]
            w.set_clr_operr(true);
        });

        reg!(set, cr).modify(|w| {
            w.set_pg(false);
            w.set_per(false);
            #[cfg(flash_v5_c5)]
            w.set_edatasel(Edatasel::Main);
        });
        set_interrupts(set, false);

        #[cfg(flash_v5_h7)]
        let operr = sr.operr();
        #[cfg(not(flash_v5_h7))]
        let operr = false;
        if sr.wrperr() {
            res = Err(Error::Protected);
        } else if sr.pgserr() || sr.strberr() || sr.incerr() || operr {
            warn!("flash error: SR={:08x}", sr.0);
            res = Err(Error::Hardware);
        }
    }
    res
}

#[cfg(flash_dual_bank)]
pub(super) fn banks_swapped() -> bool {
    pac::FLASH.optcr().read().swap_bank()
}

#[cfg(bank_setup_configurable)]
pub(super) fn dual_bank_mode() -> bool {
    #[cfg(flash_v5_c5)]
    return !pac::FLASH.optsr_cur().read().single_bank();
}

/// Runs one EDATA operation to completion. The flash must be unlocked.
#[cfg(flash_v5_c5)]
unsafe fn edata_op(start: impl FnOnce()) -> Result<(), Error> {
    wait_idle();
    let _ = finish();
    start();
    cortex_m::asm::dsb();
    wait_idle();
    finish()
}

/// Programs one half-word of the EDATA area. The flash must be unlocked.
#[cfg(flash_v5_c5)]
pub(super) unsafe fn edata_write(address: u32, value: u16) -> Result<(), Error> {
    edata_op(|| {
        start_program(0, false);
        write_volatile(address as *mut u16, value);
    })
}

/// Erases one EDATA page of a physical bank. The flash must be unlocked.
#[cfg(flash_v5_c5)]
pub(super) unsafe fn edata_erase(bank: FlashBank, page: u8) -> Result<(), Error> {
    edata_op(|| {
        reg!(0, cr).modify(|w| {
            w.set_per(true);
            w.set_pnb(page);
            w.set_edatasel(Edatasel::Edata);
            w.set_bksel(bksel(bank));
        });
        reg!(0, cr).modify(|w| w.set_strt(true));
    })
}

/// Swaps flash banks 1 and 2, persistently.
///
/// This allows the application to write a new firmware image into bank 2, then swap the
/// banks and reset, booting the new firmware. The swap takes effect at the next reset;
/// [`banks_swapped()`](super::banks_swapped) keeps returning the current state until then.
///
/// PLEASE READ THE REFERENCE MANUAL - there are nuances to this feature.
#[cfg(all(flash_dual_bank, any(flash_v5_h5, flash_v5_h50, flash_v5_c5)))]
pub fn perform_bank_swap() {
    wait_idle();
    let _ = finish();

    // Unlock OPTLOCK.
    pac::FLASH.optkeyr().write_value(0x0819_2A3B);
    pac::FLASH.optkeyr().write_value(0x4C5D_6E7F);
    while pac::FLASH.optcr().read().optlock() {}

    // Toggle the SWAP_BANK option and program the option bytes.
    let swapped = banks_swapped();
    pac::FLASH.optsr_prg().modify(|w| w.set_swap_bank(!swapped));
    pac::FLASH.optcr().modify(|w| w.set_optstrt(true));
    while pac::FLASH.optcr().read().optstrt() {}

    pac::FLASH.optcr().modify(|w| w.set_optlock(true));
}
