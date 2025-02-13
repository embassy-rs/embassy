/// STM32H50 series flash impl. See RM0492
use core::{
    ptr::write_volatile,
    sync::atomic::{fence, Ordering},
};

use cortex_m::interrupt;
use pac::flash::regs::Nssr;
use pac::flash::vals::Bksel;

use super::{Error, FlashBank, FlashRegion, FlashSector, FLASH_REGIONS, WRITE_SIZE};
use crate::pac;

pub(crate) const fn is_default_layout() -> bool {
    true
}

pub(crate) const fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
}

pub(crate) unsafe fn lock() {
    pac::FLASH.nscr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    while busy() {}

    if pac::FLASH.nscr().read().lock() {
        pac::FLASH.nskeyr().write_value(0x4567_0123);
        pac::FLASH.nskeyr().write_value(0xCDEF_89AB);
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);
    pac::FLASH.nscr().write(|w| w.set_pg(true));
}

pub(crate) unsafe fn disable_blocking_write() {
    pac::FLASH.nscr().write(|w| w.set_pg(false));
}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;

        // prevents parallelism errors
        fence(Ordering::SeqCst);
    }

    wait_ready_blocking()
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    assert!(sector.bank != FlashBank::Otp);
    assert!(sector.index_in_bank < 8);

    while busy() {}

    interrupt::free(|_| {
        pac::FLASH.nscr().modify(|w| {
            // BKSEL ignores SWAP_BANK, so we must take it into account here
            w.set_bksel(match (sector.bank, banks_swapped()) {
                (FlashBank::Bank1, false) => Bksel::BANK1,
                (FlashBank::Bank2, true) => Bksel::BANK1,
                (FlashBank::Bank2, false) => Bksel::BANK2,
                (FlashBank::Bank1, true) => Bksel::BANK2,
                _ => unreachable!(),
            });
            w.set_snb(sector.index_in_bank);
            w.set_ser(true);
            w.set_strt(true);
        })
    });

    let ret = wait_ready_blocking();
    pac::FLASH.nscr().modify(|w| w.set_ser(false));
    ret
}

pub(crate) unsafe fn wait_ready_blocking() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.nssr().read();

        if !sr_busy(sr) {
            if sr.wrperr() {
                return Err(Error::Protected);
            }
            if sr.pgserr() {
                return Err(Error::Seq);
            }
            if sr.strberr() {
                // writing several times to the same byte in the write buffer
                return Err(Error::Prog);
            }
            if sr.incerr() {
                // attempting write operation before completion of previous
                // write operation
                return Err(Error::Seq);
            }

            return Ok(());
        }
    }
}

pub(crate) unsafe fn clear_all_err() {
    pac::FLASH.nsccr().modify(|w| {
        w.set_clr_wrperr(true);
        w.set_clr_pgserr(true);
        w.set_clr_strberr(true);
        w.set_clr_incerr(true);
    })
}

/// Get the current SWAP_BANK option.
///
/// This value is only loaded on system or power-on reset. `perform_bank_swap()`
/// will not reflect here.
pub fn banks_swapped() -> bool {
    pac::FLASH.optcr().read().swap_bank()
}

/// Logical, persistent swap of flash banks 1 and 2.
///
/// This allows the application to write a new firmware blob into bank 2, then
/// swap the banks and perform a reset, loading the new firmware.
///
/// Swap does not take effect until system or power-on reset.
///
/// PLEASE READ THE REFERENCE MANUAL - there are nuances to this feature. For
/// instance, erase commands and interrupt enables which take a flash bank as a
/// parameter ignore the swap!
pub fn perform_bank_swap() {
    while busy() {}

    unsafe {
        clear_all_err();
    }

    // unlock OPTLOCK
    pac::FLASH.optkeyr().write(|w| *w = 0x0819_2A3B);
    pac::FLASH.optkeyr().write(|w| *w = 0x4C5D_6E7F);
    while pac::FLASH.optcr().read().optlock() {}

    // toggle SWAP_BANK option
    pac::FLASH.optsr_prg().modify(|w| w.set_swap_bank(!banks_swapped()));

    // load option bytes
    pac::FLASH.optcr().modify(|w| w.set_optstrt(true));
    while pac::FLASH.optcr().read().optstrt() {}

    // re-lock OPTLOCK
    pac::FLASH.optcr().modify(|w| w.set_optlock(true));
}

fn sr_busy(sr: Nssr) -> bool {
    // Note: RM0492 sometimes incorrectly refers to WBNE as NSWBNE
    sr.bsy() || sr.dbne() || sr.wbne()
}

fn busy() -> bool {
    let sr = pac::FLASH.nssr().read();
    sr_busy(sr)
}
