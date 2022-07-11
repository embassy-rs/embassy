use core::convert::TryInto;
use core::ptr::write_volatile;

use atomic_polyfill::{fence, Ordering};

use super::{ERASE_SIZE, FLASH_BASE, FLASH_SIZE};
use crate::flash::Error;
use crate::pac;

// Only available on some devices
const SECOND_BANK_OFFSET: usize = FLASH_SIZE / 2;
const SECOND_BANK_SECTOR_START: u32 = 12;

unsafe fn is_dual_bank() -> bool {
    match FLASH_SIZE / 1024 {
        // 1 MB devices depend on configuration
        1024 => {
            if cfg!(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479)) {
                pac::FLASH.optcr().read().db1m()
            } else {
                false
            }
        }
        // 2 MB devices are always dual bank
        2048 => true,
        // All other devices are single bank
        _ => false,
    }
}

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

unsafe fn get_sector(addr: u32) -> u8 {
    let offset = addr - FLASH_BASE as u32;

    let sector = if is_dual_bank() {
        let bank = offset / SECOND_BANK_OFFSET as u32;
        let offset_in_bank = offset % SECOND_BANK_OFFSET as u32;

        let sector_in_bank = if offset_in_bank >= ERASE_SIZE as u32 / 2 {
            4 + offset_in_bank / ERASE_SIZE as u32
        } else {
            offset_in_bank / (ERASE_SIZE as u32 / 8)
        };

        if bank == 1 {
            SECOND_BANK_SECTOR_START + sector_in_bank
        } else {
            sector_in_bank
        }
    } else {
        if offset >= ERASE_SIZE as u32 / 2 {
            4 + offset / ERASE_SIZE as u32
        } else {
            offset / (ERASE_SIZE as u32 / 8)
        }
    };

    sector as u8
}

pub(crate) unsafe fn blocking_erase(from: u32, to: u32) -> Result<(), Error> {
    let start_sector = get_sector(from);
    let end_sector = get_sector(to);

    for sector in start_sector..end_sector {
        let ret = erase_sector(sector as u8);
        if ret.is_err() {
            return ret;
        }
    }

    Ok(())
}

unsafe fn erase_sector(sector: u8) -> Result<(), Error> {
    let bank = sector / SECOND_BANK_SECTOR_START as u8;
    let snb = (bank << 4) + (sector % SECOND_BANK_SECTOR_START as u8);

    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(snb)
    });

    pac::FLASH.cr().modify(|w| {
        w.set_strt(true);
    });

    let ret: Result<(), Error> = blocking_wait_ready();

    clear_all_err();

    ret
}

pub(crate) unsafe fn clear_all_err() {
    pac::FLASH.sr().write(|w| {
        w.set_pgserr(true);
        w.set_pgperr(true);
        w.set_pgaerr(true);
        w.set_wrperr(true);
        w.set_eop(true);
    });
}

pub(crate) unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.sr().read();

        if !sr.bsy() {
            if sr.pgserr() {
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
