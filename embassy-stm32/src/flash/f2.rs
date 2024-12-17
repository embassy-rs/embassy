use core::ptr::write_volatile;
use core::sync::atomic::{fence, AtomicBool, Ordering};

use pac::flash::regs::Sr;

use super::{FlashBank, FlashRegion, FlashSector, FLASH_REGIONS, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

static DATA_CACHE_WAS_ENABLED: AtomicBool = AtomicBool::new(false);

impl FlashSector {
    const fn snb(&self) -> u8 {
        ((self.bank as u8) << 4) + self.index_in_bank
    }
}

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
    if pac::FLASH.cr().read().lock() {
        pac::FLASH.keyr().write_value(0x4567_0123);
        pac::FLASH.keyr().write_value(0xCDEF_89AB);
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);
    save_data_cache_state();

    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(pac::flash::vals::Psize::PSIZE32);
    });
}

pub(crate) unsafe fn disable_blocking_write() {
    pac::FLASH.cr().write(|w| w.set_pg(false));
    restore_data_cache_state();
}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    write_start(start_address, buf);
    blocking_wait_ready()
}

unsafe fn write_start(start_address: u32, buf: &[u8; WRITE_SIZE]) {
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;

        // prevents parallelism errors
        fence(Ordering::SeqCst);
    }
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    save_data_cache_state();

    trace!("Blocking erasing sector number {}", sector.snb());

    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(sector.snb())
    });

    pac::FLASH.cr().modify(|w| {
        w.set_strt(true);
    });

    let ret: Result<(), Error> = blocking_wait_ready();
    clear_all_err();
    restore_data_cache_state();
    ret
}

pub(crate) unsafe fn clear_all_err() {
    // read and write back the same value.
    // This clears all "write 1 to clear" bits.
    pac::FLASH.sr().modify(|_| {});
}

unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.sr().read();

        if !sr.bsy() {
            return get_result(sr);
        }
    }
}

fn get_result(sr: Sr) -> Result<(), Error> {
    if sr.pgserr() {
        Err(Error::Seq)
    } else if sr.pgperr() {
        Err(Error::Parallelism)
    } else if sr.pgaerr() {
        Err(Error::Unaligned)
    } else if sr.wrperr() {
        Err(Error::Protected)
    } else {
        Ok(())
    }
}

fn save_data_cache_state() {
    let dual_bank = unwrap!(get_flash_regions().last()).bank == FlashBank::Bank2;
    if dual_bank {
        // Disable data cache during write/erase if there are two banks, see errata 2.2.12
        let dcen = pac::FLASH.acr().read().dcen();
        DATA_CACHE_WAS_ENABLED.store(dcen, Ordering::Relaxed);
        if dcen {
            pac::FLASH.acr().modify(|w| w.set_dcen(false));
        }
    }
}

fn restore_data_cache_state() {
    let dual_bank = unwrap!(get_flash_regions().last()).bank == FlashBank::Bank2;
    if dual_bank {
        // Restore data cache if it was enabled
        let dcen = DATA_CACHE_WAS_ENABLED.load(Ordering::Relaxed);
        if dcen {
            // Reset data cache before we enable it again
            pac::FLASH.acr().modify(|w| w.set_dcrst(true));
            pac::FLASH.acr().modify(|w| w.set_dcrst(false));
            pac::FLASH.acr().modify(|w| w.set_dcen(true))
        }
    }
}
