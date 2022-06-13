use core::convert::TryInto;
use core::ptr::write_volatile;

use atomic_polyfill::{fence, Ordering};

use crate::flash::Error;
use crate::pac;

pub(crate) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    pac::FLASH.keyr().write(|w| w.set_key(0x4567_0123));
    pac::FLASH.keyr().write(|w| w.set_key(0xCDEF_89AB));
}

pub(crate) unsafe fn blocking_write(offset: u32, buf: &[u8]) -> Result<(), Error> {
    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(pac::flash::vals::Psize::PSIZE32);
    });

    let ret = {
        let mut ret: Result<(), Error> = Ok(());
        let mut offset = offset;
        for chunk in buf.chunks(super::WRITE_SIZE) {
            for val in chunk.chunks(4) {
                write_volatile(offset as *mut u32, u32::from_le_bytes(val[0..4].try_into().unwrap()));
                offset += val.len() as u32;

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

pub(crate) unsafe fn blocking_erase(from: u32, to: u32) -> Result<(), Error> {
    let start_sector = if from >= (super::FLASH_BASE + super::ERASE_SIZE / 2) as u32 {
        4 + (from - super::FLASH_BASE as u32) / super::ERASE_SIZE as u32
    } else {
        (from - super::FLASH_BASE as u32) / (super::ERASE_SIZE as u32 / 8)
    };

    let end_sector = if to >= (super::FLASH_BASE + super::ERASE_SIZE / 2) as u32 {
        4 + (to - super::FLASH_BASE as u32) / super::ERASE_SIZE as u32
    } else {
        (to - super::FLASH_BASE as u32) / (super::ERASE_SIZE as u32 / 8)
    };

    for sector in start_sector..end_sector {
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
