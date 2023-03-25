use core::convert::TryInto;
use core::mem::size_of;
use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use embassy_hal_common::stm32::flash::f7::get_sector;

use super::FlashRegion;
use crate::flash::Error;
use crate::pac;

pub(crate) const MAX_WRITE_SIZE: usize = super::BANK1_REGION3::WRITE_SIZE;
pub(crate) const MAX_ERASE_SIZE: usize = super::BANK1_REGION3::ERASE_SIZE;

pub(crate) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    pac::FLASH.keyr().write(|w| w.set_key(0x4567_0123));
    pac::FLASH.keyr().write(|w| w.set_key(0xCDEF_89AB));
}

pub(crate) unsafe fn blocking_write(first_address: u32, buf: &[u8]) -> Result<(), Error> {
    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(pac::flash::vals::Psize::PSIZE32);
    });

    let ret = {
        let mut ret: Result<(), Error> = Ok(());
        let mut address = first_address;
        for chunk in buf.chunks(MAX_WRITE_SIZE) {
            let vals = chunk.chunks_exact(size_of::<u32>());
            assert!(vals.remainder().is_empty());
            for val in vals {
                write_volatile(address as *mut u32, u32::from_le_bytes(val.try_into().unwrap()));
                address += val.len() as u32;

                // prevents parallelism errors
                fence(Ordering::SeqCst);
            }

            ret = blocking_wait_ready();
            if ret.is_err() {
                break;
            }
        }
        ret
    };

    pac::FLASH.cr().write(|w| w.set_pg(false));

    ret
}

pub(crate) unsafe fn blocking_erase(from_address: u32, to_address: u32) -> Result<(), Error> {
    let start_sector = get_sector(from_address);
    let end_sector = get_sector(to_address);
    for sector in start_sector.index..end_sector.index {
        let ret = erase_sector(sector as u8);
        if ret.is_err() {
            return ret;
        }
    }

    Ok(())
}

unsafe fn erase_sector(sector: u8) -> Result<(), Error> {
    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(sector)
    });

    pac::FLASH.cr().modify(|w| {
        w.set_strt(true);
    });

    let ret: Result<(), Error> = blocking_wait_ready();

    pac::FLASH.cr().modify(|w| w.set_ser(false));

    clear_all_err();

    ret
}

pub(crate) unsafe fn clear_all_err() {
    pac::FLASH.sr().modify(|w| {
        if w.erserr() {
            w.set_erserr(true);
        }
        if w.pgperr() {
            w.set_pgperr(true);
        }
        if w.pgaerr() {
            w.set_pgaerr(true);
        }
        if w.wrperr() {
            w.set_wrperr(true);
        }
        if w.eop() {
            w.set_eop(true);
        }
    });
}

pub(crate) unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.sr().read();

        if !sr.bsy() {
            if sr.erserr() {
                return Err(Error::Seq);
            }

            if sr.pgperr() {
                return Err(Error::Parallelism);
            }

            if sr.pgaerr() {
                return Err(Error::Unaligned);
            }

            if sr.wrperr() {
                return Err(Error::Protected);
            }

            return Ok(());
        }
    }
}
