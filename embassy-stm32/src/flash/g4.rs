use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use cortex_m::interrupt;

use super::{Error, FlashRegion, FlashSector, FLASH_REGIONS, WRITE_SIZE};
use crate::pac;

pub const fn is_default_layout() -> bool {
    true
}

pub const fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
}

pub(crate) unsafe fn lock() {
    // trace!("locking");
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    // trace!("unlocking");
    // Wait, while the memory interface is busy.
    while pac::FLASH.sr().read().bsy() {}

    // Unlock flash
    if pac::FLASH.cr().read().lock() {
        pac::FLASH.keyr().write(|w| w.set_keyr(0x4567_0123));
        pac::FLASH.keyr().write(|w| w.set_keyr(0xCDEF_89AB));
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    // trace!("enable_blocking_write");
    assert_eq!(0, WRITE_SIZE % 4);

    // workaround for errata 2.2.2
    // TODO: do this conditionally on whether it's already disabled?
    pac::FLASH.acr().modify(|w| w.set_dcen(false));
    pac::FLASH.cr().write(|w| w.set_pg(true));
}

pub(crate) unsafe fn disable_blocking_write() {
    // trace!("disable_blocking_write");
    pac::FLASH.cr().write(|w| w.set_pg(false));
    pac::FLASH.acr().modify(|w| w.set_dcrst(true));
    pac::FLASH.acr().modify(|w| w.set_dcen(true));
}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    // trace!("blocking_write start_address={:08x} buf={}", start_address, buf.len());
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(val.try_into().unwrap()));
        address += val.len() as u32;

        // prevents parallelism errors
        fence(Ordering::SeqCst);
    }

    wait_ready_blocking()
}

const DUAL_BANK_ERASE_SIZE: u32 = 2048;

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    let mut start = sector.start;

    // embassy currently incorrectly assumes 4k sectors for the G4
    // split each sector into two 2k sectors
    // TODO: remove after embassy uses the correct size
    while start < sector.size + sector.start {
        let idx = (start - super::FLASH_BASE as u32) / DUAL_BANK_ERASE_SIZE;
        let (idx, bank) = if idx < 128 { (idx, 0) } else { (idx - 128, 1) };
        // trace!("Erasing idx {} in bank {}", idx, bank);

        while pac::FLASH.sr().read().bsy() {}
        clear_all_err();

        interrupt::free(|_| {
            pac::FLASH.cr().modify(|w| {
                w.set_per(true);
                w.set_pnb(idx as u8);
                w.set_strt(true);

                // BKER bit 11
                // TODO: add to stm32 metapac
                w.0 = (w.0 & !(0x01 << 11usize)) | ((bank & 0x01) << 11usize);
            });
        });

        let ret: Result<(), Error> = wait_ready_blocking();
        pac::FLASH.cr().modify(|w| {
            w.set_per(false);
            w.0 = w.0 & !(0x01 << 11usize);
        });
        ret?;

        start += DUAL_BANK_ERASE_SIZE;
    }

    assert_eq!(start, sector.size + sector.start);

    Ok(())
}

pub(crate) unsafe fn wait_ready_blocking() -> Result<(), Error> {
    while pac::FLASH.sr().read().bsy() {}

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

    // TODO: should check some of the other errors? Also the should check for the operation finished flag?

    Ok(())
}

pub(crate) unsafe fn clear_all_err() {
    // read and write back the same value.
    // This clears all "write 1 to clear" bits.
    pac::FLASH.sr().modify(|_| {});
}
