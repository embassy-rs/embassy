use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use super::{FlashBank, FlashSector, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

pub(crate) unsafe fn lock() {
    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.scr().modify(|w| w.set_lock(true));
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    #[cfg(feature = "trustzone-secure")]
    if pac::FLASH.scr().read().lock() {
        pac::FLASH.skeyr().write(|w| w.set_key(0x4567_0123));
        pac::FLASH.skeyr().write(|w| w.set_key(0xCDEF_89AB));
    }
    #[cfg(not(feature = "trustzone-secure"))]
    if pac::FLASH.cr().read().lock() {
        pac::FLASH.keyr().write(|w| w.set_key(0x4567_0123));
        pac::FLASH.keyr().write(|w| w.set_key(0xCDEF_89AB));
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);

    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.scr().write(|w| {
        w.set_pg(true);
    });
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
    });
}

pub(crate) unsafe fn disable_blocking_write() {
    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.scr().write(|w| w.set_pg(false));
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.cr().write(|w| w.set_pg(false));
}

/// Wait until write buffer is empty (WDW) and no operation is in progress (BSY).
/// Per RM: check before starting a new program; after writing, wait WDW then BSY.
unsafe fn blocking_wait_wdw_and_bsy() {
    loop {
        #[cfg(feature = "trustzone-secure")]
        let sr = pac::FLASH.ssr().read();
        #[cfg(not(feature = "trustzone-secure"))]
        let sr = pac::FLASH.sr().read();

        if !sr.wdw() && !sr.bsy() {
            return;
        }
    }
}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    // Per RM: before programming, ensure WDW (write buffer empty) and BSY (no op ongoing).
    blocking_wait_wdw_and_bsy();

    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;

        // prevents parallelism errors
        fence(Ordering::SeqCst);
    }

    blocking_wait_ready()
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.scr().modify(|w| {
        w.set_per(true);
        w.set_pnb(sector.index_in_bank);
        // TODO: add check for bank swap
        w.set_bker(match sector.bank {
            FlashBank::Bank1 => false,
            FlashBank::Bank2 => true,
            _ => unreachable!(),
        });
    });
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.cr().modify(|w| {
        w.set_per(true);
        w.set_pnb(sector.index_in_bank);
        // TODO: add check for bank swap
        w.set_bker(match sector.bank {
            FlashBank::Bank1 => false,
            FlashBank::Bank2 => true,
            _ => unreachable!(),
        });
    });

    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.scr().modify(|w| {
        w.set_strt(true);
    });
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.cr().modify(|w| {
        w.set_strt(true);
    });

    cortex_m::asm::dsb();
    cortex_m::asm::isb();
    fence(Ordering::SeqCst);

    let ret: Result<(), Error> = blocking_wait_ready();

    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.scr().modify(|w| w.set_per(false));
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.cr().modify(|w| w.set_per(false));
    clear_all_err();

    // Invalidate ICACHE after erase to prevent stale cached reads.
    if pac::ICACHE.cr().read().en() {
        pac::ICACHE.fcr().write(|w| w.set_cbsyendf(true));
        pac::ICACHE.cr().modify(|w| w.set_cacheinv(true));
        while !pac::ICACHE.sr().read().bsyendf() {}
    }

    ret
}

pub(crate) unsafe fn clear_all_err() {
    // Explicit clear of all "write 1 to clear" bits in SR/SSR (per RM).
    // Note: SSR on U3 does not have optwerr field (only present in SR).
    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.ssr().modify(|w| {
        w.set_eop(true);
        w.set_operr(true);
        w.set_progerr(true);
        w.set_wrperr(true);
        w.set_pgaerr(true);
        w.set_sizerr(true);
        w.set_pgserr(true);
    });
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.sr().modify(|w| {
        w.set_eop(true);
        w.set_operr(true);
        w.set_progerr(true);
        w.set_wrperr(true);
        w.set_pgaerr(true);
        w.set_sizerr(true);
        w.set_pgserr(true);
        w.set_optwerr(true);
    });
}

unsafe fn blocking_wait_ready() -> Result<(), Error> {
    // Per RM: wait until WDW is cleared, then until BSY is cleared.
    loop {
        #[cfg(feature = "trustzone-secure")]
        let sr = pac::FLASH.ssr().read();
        #[cfg(not(feature = "trustzone-secure"))]
        let sr = pac::FLASH.sr().read();

        if !sr.wdw() && !sr.bsy() {
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

            if sr.operr() {
                return Err(Error::Prog);
            }

            return Ok(());
        }
    }
}
