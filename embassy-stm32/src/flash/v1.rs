//! Flash controller v1: STM32F0, F1 and F3.
//!
//! Programs one half-word at a time, erases pages by address (`AR`). The F1 XL-density parts
//! have a separate register set per bank.

use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use super::{Config, Error, FlashBank, FlashSector, WRITE_SIZE};
use crate::pac;

const KEY1: u32 = 0x4567_0123;
const KEY2: u32 = 0xCDEF_89AB;

#[cfg(flash_v1_f1xl)]
const BANKS: usize = 2;
#[cfg(not(flash_v1_f1xl))]
const BANKS: usize = 1;

#[cfg(flash_v1_f1xl)]
fn regs(bank: usize) -> pac::flash::Bank {
    pac::FLASH.bank(bank)
}
#[cfg(not(flash_v1_f1xl))]
fn regs(_bank: usize) -> pac::flash::Flash {
    pac::FLASH
}

/// Index of the register set of the bank containing `address`.
fn bank_of(address: u32) -> usize {
    #[cfg(flash_dual_bank)]
    if address >= super::FLASH_BANK2_BASE {
        return 1;
    }
    let _ = address;
    0
}

pub(super) unsafe fn unlock() {
    for bank in 0..BANKS {
        let r = regs(bank);
        if r.cr().read().lock() {
            r.keyr().write_value(KEY1);
            r.keyr().write_value(KEY2);
        }
    }
}

pub(super) unsafe fn lock() {
    for bank in 0..BANKS {
        regs(bank).cr().modify(|w| w.set_lock(true));
    }
}

pub(super) unsafe fn start_write(address: u32, data: &[u8; WRITE_SIZE], interrupt: bool) {
    let r = regs(bank_of(address));
    r.cr().write(|w| {
        w.set_pg(true);
        w.set_eopie(interrupt);
        w.set_errie(interrupt);
    });
    write_volatile(address as *mut u16, u16::from_le_bytes(*data));
    fence(Ordering::SeqCst);
}

pub(super) unsafe fn start_erase(sector: &FlashSector, _config: &Config, interrupt: bool) {
    let r = regs(match sector.bank {
        FlashBank::Bank2 => 1,
        _ => 0,
    });
    r.cr().write(|w| {
        w.set_per(true);
        w.set_eopie(interrupt);
        w.set_errie(interrupt);
    });
    r.ar().write(|w| w.set_far(sector.start));
    r.cr().modify(|w| w.set_strt(true));
    // BSY is set one cycle after STRT (RM0091 §3.2.2, F105/F107 errata 2.2.8).
    r.cr().read();
}

pub(super) fn busy() -> bool {
    (0..BANKS).any(|bank| regs(bank).sr().read().bsy())
}

pub(super) fn wait_idle() {
    while busy() {}
}

pub(super) fn finish() -> Result<(), Error> {
    let mut res = Ok(());
    for bank in 0..BANKS {
        let r = regs(bank);
        let sr = r.sr().read();
        // Clear PG/PER and the interrupt enables. LOCK is write-1-only.
        r.cr().write(|_| {});
        r.sr().write(|w| {
            w.set_eop(true);
            w.set_pgerr(true);
            w.set_wrprterr(true);
        });
        if sr.wrprterr() {
            res = Err(Error::Protected);
        } else if sr.pgerr() {
            res = Err(Error::NotErased);
        }
    }
    res
}

/// F1 XL-density parts can boot from bank 2 (BFB2), but can't remap the banks.
#[cfg(flash_dual_bank)]
pub(super) fn banks_swapped() -> bool {
    false
}
