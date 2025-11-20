use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use cortex_m::interrupt;

use super::{FlashSector, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

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

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    wait_busy();
    clear_all_err();

    interrupt::free(|_| {
        pac::FLASH.cr().modify(|w| {
            w.set_per(true);
            #[cfg(any(flash_g0x0, flash_g0x1, flash_g4c3))]
            w.set_bker(sector.bank == crate::flash::FlashBank::Bank2);
            #[cfg(flash_g0x0)]
            w.set_pnb(sector.index_in_bank as u16);
            #[cfg(not(flash_g0x0))]
            w.set_pnb(sector.index_in_bank as u8);
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

#[cfg(all(bank_setup_configurable, any(flash_g4c2, flash_g4c3, flash_g4c4)))]
pub(crate) fn check_bank_setup() {
    if cfg!(feature = "single-bank") && pac::FLASH.optr().read().dbank() {
        panic!(
            "Embassy is configured as single-bank, but the hardware is running in dual-bank mode. Change the hardware by changing the dbank value in the user option bytes or configure embassy to use dual-bank config"
        );
    }
    if cfg!(feature = "dual-bank") && !pac::FLASH.optr().read().dbank() {
        panic!(
            "Embassy is configured as dual-bank, but the hardware is running in single-bank mode. Change the hardware by changing the dbank value in the user option bytes or configure embassy to use single-bank config"
        );
    }
}

#[cfg(all(bank_setup_configurable, flash_g0x1))]
pub(crate) fn check_bank_setup() {
    if cfg!(feature = "single-bank") && pac::FLASH.optr().read().dual_bank() {
        panic!(
            "Embassy is configured as single-bank, but the hardware is running in dual-bank mode. Change the hardware by changing the dual_bank value in the user option bytes or configure embassy to use dual-bank config"
        );
    }
    if cfg!(feature = "dual-bank") && !pac::FLASH.optr().read().dual_bank() {
        panic!(
            "Embassy is configured as dual-bank, but the hardware is running in single-bank mode. Change the hardware by changing the dual_bank value in the user option bytes or configure embassy to use single-bank config"
        );
    }
}
