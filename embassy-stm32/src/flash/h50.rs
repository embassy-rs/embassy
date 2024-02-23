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
        write_volatile(address as *mut u32, u32::from_le_bytes(val.try_into().unwrap()));
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
            w.set_bksel(match sector.bank {
                FlashBank::Bank1 => Bksel::B_0X0,
                FlashBank::Bank2 => Bksel::B_0X1,
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

fn sr_busy(sr: Nssr) -> bool {
    // Note: RM0492 sometimes incorrectly refers to WBNE as NSWBNE
    sr.bsy() || sr.dbne() || sr.wbne()
}

fn busy() -> bool {
    let sr = pac::FLASH.nssr().read();
    sr_busy(sr)
}
