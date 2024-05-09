use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use cortex_m::interrupt;

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
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}
pub(crate) unsafe fn unlock() {
    // Wait, while the memory interface is busy.
    while pac::FLASH.sr().read().bsy1() {}

    // Unlock flash
    if pac::FLASH.cr().read().lock() {
        pac::FLASH.keyr().write(|w| w.set_key(0x4567_0123));
        pac::FLASH.keyr().write(|w| w.set_key(0xCDEF_89AB));
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);
    pac::FLASH.cr().write(|w| w.set_pg(true));
}

pub(crate) unsafe fn disable_blocking_write() {
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

    wait_ready_blocking()
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    let idx = (sector.start - super::FLASH_BASE as u32) / super::BANK1_REGION.erase_size as u32;
    while pac::FLASH.sr().read().bsy1() {}
    clear_all_err();

    interrupt::free(|_| {
        pac::FLASH.cr().modify(|w| {
            w.set_per(true);
            w.set_pnb(idx as u8);
            w.set_strt(true);
        });
    });

    let ret: Result<(), Error> = wait_ready_blocking();
    pac::FLASH.cr().modify(|w| w.set_per(false));
    ret
}

pub(crate) unsafe fn wait_ready_blocking() -> Result<(), Error> {
    while pac::FLASH.sr().read().bsy1() {}

    let sr = pac::FLASH.sr().read();

    if sr.progerr() {
        return Err(Error::Prog);
    }

    if sr.wrperr() {
        return Err(Error::Protected);
    }

    if sr.pgaerr() {
        return Err(Error::Unaligned);
    }

    Ok(())
}

pub(crate) unsafe fn clear_all_err() {
    // read and write back the same value.
    // This clears all "write 1 to clear" bits.
    pac::FLASH.sr().modify(|_| {});
}
