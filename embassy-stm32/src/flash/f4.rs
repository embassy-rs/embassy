use core::convert::TryInto;
use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use embassy_hal_common::stm32::flash::f4::{get_sector, SECOND_BANK_SECTOR_OFFSET};

use super::{FLASH_SIZE, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

fn is_dual_bank() -> bool {
    match FLASH_SIZE / 1024 {
        // 1 MB devices depend on configuration
        1024 => {
            if cfg!(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479)) {
                unsafe { pac::FLASH.optcr().read().db1m() }
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

pub(crate) unsafe fn begin_write() {
    assert_eq!(0, WRITE_SIZE % 4);

    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(pac::flash::vals::Psize::PSIZE32);
    });
}

pub(crate) unsafe fn end_write() {
    pac::FLASH.cr().write(|w| w.set_pg(false));
}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(val.try_into().unwrap()));
        address += val.len() as u32;

        // prevents parallelism errors
        fence(Ordering::SeqCst);
    }

    blocking_wait_ready()
}

pub(crate) fn is_eraseable_range(start_address: u32, end_address: u32) -> bool {
    let dual_bank = is_dual_bank();
    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address, dual_bank, FLASH_SIZE as u32);
        if sector.start != address {
            return false;
        }
        address += sector.size;
    }
    address == end_address
}

pub(crate) unsafe fn blocking_erase(start_address: u32, end_address: u32) -> Result<(), Error> {
    let dual_bank = is_dual_bank();
    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address, dual_bank, FLASH_SIZE as u32);
        erase_sector(sector.index)?;
        address += sector.size;
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

unsafe fn blocking_wait_ready() -> Result<(), Error> {
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
