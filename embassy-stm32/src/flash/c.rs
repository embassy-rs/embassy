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
    let idx = (sector.start - super::FLASH_BASE as u32) / super::BANK1_REGION.erase_size as u32;

    #[cfg(feature = "defmt")]
    defmt::trace!(
        "STM32C0 Erase: addr=0x{:08x}, idx={}, erase_size={}",
        sector.start,
        idx,
        super::BANK1_REGION.erase_size
    );

    wait_busy();
    clear_all_err();

    // Explicitly unlock before erase
    unlock();

    interrupt::free(|_| {
        #[cfg(feature = "defmt")]
        {
            let cr_before = pac::FLASH.cr().read();
            defmt::trace!("FLASH_CR before: 0x{:08x}", cr_before.0);
        }

        pac::FLASH.cr().modify(|w| {
            w.set_per(true);
            w.set_pnb(idx as u8);
            w.set_strt(true);
        });

        #[cfg(feature = "defmt")]
        {
            let cr_after = pac::FLASH.cr().read();
            defmt::trace!(
                "FLASH_CR after: 0x{:08x}, PER={}, PNB={}, STRT={}",
                cr_after.0,
                cr_after.per(),
                cr_after.pnb(),
                cr_after.strt()
            );
        }
    });

    let ret: Result<(), Error> = wait_ready_blocking();

    // Clear erase bit
    pac::FLASH.cr().modify(|w| w.set_per(false));

    // Explicitly lock after erase
    lock();

    // Extra wait to ensure operation completes
    wait_busy();

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

fn wait_busy() {
    while pac::FLASH.sr().read().bsy() {}
}
