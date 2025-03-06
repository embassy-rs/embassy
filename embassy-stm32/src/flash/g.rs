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
    wait_busy();

    // Unlock flash
    if pac::FLASH.cr().read().lock() {
        pac::FLASH.keyr().write_value(0x4567_0123);
        pac::FLASH.keyr().write_value(0xCDEF_89AB);
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
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;

        // prevents parallelism errors
        fence(Ordering::SeqCst);
    }

    wait_ready_blocking()
}

/// Fix for incorrect register field width in STM32G4 category metadata
#[cfg(flash_g4c4)]
#[inline(always)]
pub fn set_pnb_c4(cr: &mut stm32_metapac::flash::regs::Cr, val: u8) {
    cr.0 = (cr.0 & !(0xff << 3usize)) | (((val as u32) & 0xff) << 3usize);
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    let idx = (sector.start - super::FLASH_BASE as u32) / super::BANK1_REGION.erase_size as u32;
    wait_busy();
    clear_all_err();

    interrupt::free(|_| {
        pac::FLASH.cr().modify(|w| {
            w.set_per(true);
            #[cfg(any(flash_g0x0, flash_g0x1, flash_g4c3))]
            w.set_bker(sector.bank == crate::flash::FlashBank::Bank2);
            #[cfg(flash_g0x0)]
            w.set_pnb(idx as u16);
            #[cfg(flash_g4c4)]
            set_pnb_c4(w, idx as u8);
            // otherwise
            #[cfg(not(any(flash_g0x0, flash_g4c4)))]
            w.set_pnb(idx as u8);
            w.set_strt(true);
        });
    });

    let ret: Result<(), Error> = wait_ready_blocking();
    pac::FLASH.cr().modify(|w| w.set_per(false));
    ret
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

    Ok(())
}

pub(crate) unsafe fn clear_all_err() {
    // read and write back the same value.
    // This clears all "write 1 to clear" bits.
    pac::FLASH.sr().modify(|_| {});
}

#[cfg(any(flash_g0x0, flash_g0x1))]
fn wait_busy() {
    while pac::FLASH.sr().read().bsy() | pac::FLASH.sr().read().bsy2() {}
}

#[cfg(not(any(flash_g0x0, flash_g0x1)))]
fn wait_busy() {
    while pac::FLASH.sr().read().bsy() {}
}
