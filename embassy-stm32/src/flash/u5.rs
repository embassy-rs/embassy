use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use super::{FlashRegion, FlashSector, FLASH_REGIONS, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

pub(crate) const fn is_default_layout() -> bool {
    true
}

pub(crate) const fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
}

pub(crate) unsafe fn lock() {
    pac::FLASH.seccr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    if pac::FLASH.seccr().read().lock() {
        pac::FLASH.seckeyr().write_value(0x4567_0123);
        pac::FLASH.seckeyr().write_value(0xCDEF_89AB);
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);

    pac::FLASH.seccr().write(|w| {
        w.set_pg(pac::flash::vals::SeccrPg::B_0X1);
    });
}

pub(crate) unsafe fn disable_blocking_write() {
    pac::FLASH.seccr().write(|w| w.set_pg(pac::flash::vals::SeccrPg::B_0X0));
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

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    pac::FLASH.seccr().modify(|w| {
        w.set_per(pac::flash::vals::SeccrPer::B_0X1);
        w.set_pnb(sector.index_in_bank)
    });

    pac::FLASH.seccr().modify(|w| {
        w.set_strt(true);
    });

    let ret: Result<(), Error> = blocking_wait_ready();
    pac::FLASH
        .seccr()
        .modify(|w| w.set_per(pac::flash::vals::SeccrPer::B_0X0));
    clear_all_err();
    ret
}

pub(crate) unsafe fn clear_all_err() {
    // read and write back the same value.
    // This clears all "write 1 to clear" bits.
    pac::FLASH.secsr().modify(|_| {});
}

unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.secsr().read();

        if !sr.bsy() {
            if sr.pgserr() {
                return Err(Error::Seq);
            }

            if sr.sizerr() {
                return Err(Error::Size);
            }

            if sr.pgaerr() {
                return Err(Error::Unaligned);
            }

            if sr.wrperr() {
                return Err(Error::Protected);
            }

            if sr.progerr() {
                return Err(Error::Prog);
            }

            return Ok(());
        }
    }
}
