//! Flash controller v3: STM32L0 and L1.
//!
//! The program memory erases to 0x00. A plain 32-bit store programs a word, and a page is
//! erased by writing a word into it with `ERASE` and `PROG` set. There is no bank or page
//! number: everything is selected by address. The same controller handles the data EEPROM.

use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use super::{Config, Error, FlashSector, WRITE_SIZE};
use crate::pac;

pub(super) unsafe fn unlock() {
    let pecr = pac::FLASH.pecr().read();
    if pecr.pelock() {
        pac::FLASH.pekeyr().write_value(0x89AB_CDEF);
        pac::FLASH.pekeyr().write_value(0x0203_0405);
    }
    if pecr.prglock() {
        pac::FLASH.prgkeyr().write_value(0x8C9D_AEBF);
        pac::FLASH.prgkeyr().write_value(0x1314_1516);
    }
}

pub(super) unsafe fn lock() {
    // Setting PELOCK also sets PRGLOCK and OPTLOCK.
    pac::FLASH.pecr().modify(|w| w.set_pelock(true));
}

fn set_interrupts(w: &mut pac::flash::regs::Pecr, enable: bool) {
    w.set_eopie(enable);
    w.set_errie(enable);
}

pub(super) unsafe fn start_write(address: u32, data: &[u8; WRITE_SIZE], interrupt: bool) {
    pac::FLASH.pecr().modify(|w| set_interrupts(w, interrupt));
    write_volatile(address as *mut u32, u32::from_le_bytes(*data));
    fence(Ordering::SeqCst);
}

pub(super) unsafe fn start_erase(sector: &FlashSector, _config: &Config, interrupt: bool) {
    pac::FLASH.pecr().modify(|w| {
        w.set_erase(true);
        w.set_prog(true);
        set_interrupts(w, interrupt);
    });
    // RM0038 §3.4.2: write 0x0000_0000 to the first word of the page.
    write_volatile(sector.start as *mut u32, 0);
    fence(Ordering::SeqCst);
}

/// Writes a byte, half-word or word to the data EEPROM. The EEPROM must be unlocked.
pub(super) unsafe fn eeprom_write<T: Copy>(address: u32, value: T) -> Result<(), Error> {
    wait_idle();
    let _ = finish();
    write_volatile(address as *mut T, value);
    fence(Ordering::SeqCst);
    wait_idle();
    finish()
}

pub(super) fn busy() -> bool {
    pac::FLASH.sr().read().bsy()
}

pub(super) fn wait_idle() {
    while busy() {}
}

pub(super) fn finish() -> Result<(), Error> {
    let sr = pac::FLASH.sr().read();
    // PECR can only be written while PELOCK is cleared.
    if !pac::FLASH.pecr().read().pelock() {
        pac::FLASH.pecr().modify(|w| {
            w.set_erase(false);
            w.set_prog(false);
            set_interrupts(w, false);
        });
    }
    pac::FLASH.sr().write(|w| {
        w.set_eop(true);
        w.set_wrperr(true);
        w.set_pgaerr(true);
        w.set_sizerr(true);
        w.set_rderr(true);
        #[cfg(flash_v3_l0)]
        {
            w.set_notzeroerr(true);
            w.set_fwwerr(true);
        }
    });

    #[cfg(flash_v3_l0)]
    let (not_erased, aborted) = (sr.notzeroerr(), sr.fwwerr());
    #[cfg(flash_v3_l1)]
    let (not_erased, aborted) = (false, false);

    if sr.wrperr() {
        Err(Error::Protected)
    } else if not_erased {
        Err(Error::NotErased)
    } else if aborted || sr.pgaerr() || sr.sizerr() {
        warn!("flash error: SR={:08x}", sr.0);
        Err(Error::Hardware)
    } else {
        Ok(())
    }
}

#[cfg(flash_dual_bank)]
pub(super) fn banks_swapped() -> bool {
    // L0 category 5: bank 2 is mapped at 0x0800_0000 with SYSCFG_CFGR1.UFB. L1 can only boot
    // from bank 2 (nBFB2), it can't remap the banks.
    #[cfg(flash_v3_l0)]
    return pac::SYSCFG.cfgr1().read().ufb();
    #[cfg(flash_v3_l1)]
    return false;
}
