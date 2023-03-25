use core::convert::TryInto;
use core::mem::size_of;
use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use embassy_hal_common::stm32::flash::f4::{get_sector, SECOND_BANK_SECTOR_OFFSET};

use super::{FlashRegion, FLASH_SIZE, MAINC};
use crate::flash::Error;
use crate::pac;

pub(crate) const MAX_WRITE_SIZE: usize = MAINC::WRITE_SIZE;
pub(crate) const MAX_ERASE_SIZE: usize = MAINC::ERASE_SIZE;

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

pub(crate) unsafe fn blocking_write(first_address: u32, buf: &[u8]) -> Result<(), Error> {
    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(pac::flash::vals::Psize::PSIZE32);
    });

    let ret = {
        let mut ret: Result<(), Error> = Ok(());
        let mut offset = first_address;
        for chunk in buf.chunks(MAX_WRITE_SIZE) {
            let vals = chunk.chunks_exact(size_of::<u32>());
            assert!(vals.remainder().is_empty());
            for val in vals {
                write_volatile(offset as *mut u32, u32::from_le_bytes(val.try_into().unwrap()));
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

pub(crate) unsafe fn blocking_erase(from_address: u32, to_address: u32) -> Result<(), Error> {
    let mut addr = from_address;
    let dual_bank = is_dual_bank();

    while addr < to_address {
        let sector = get_sector(addr, dual_bank, FLASH_SIZE as u32);
        erase_sector(sector.index)?;
        addr += sector.size;
    }

    Ok(())
}

unsafe fn erase_sector(sector: u8) -> Result<(), Error> {
    let bank = sector / SECOND_BANK_SECTOR_OFFSET as u8;
    let snb = (bank << 4) + (sector % SECOND_BANK_SECTOR_OFFSET as u8);

    trace!("Erasing sector: {}", sector);

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
