use core::ptr::write_volatile;

use atomic_polyfill::{fence, Ordering};

use super::{FlashRegion, FlashSector, FLASH_REGIONS, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

pub const fn set_default_layout() {}

pub const fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
}

pub(crate) unsafe fn on_interrupt() {
    unimplemented!();
}

pub(crate) unsafe fn lock() {
    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    pac::FLASH.cr().modify(|w| w.set_lock(true));

    #[cfg(any(flash_l0))]
    pac::FLASH.pecr().modify(|w| {
        w.set_optlock(true);
        w.set_prglock(true);
        w.set_pelock(true);
    });
}

pub(crate) unsafe fn unlock() {
    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    {
        pac::FLASH.keyr().write(|w| w.set_keyr(0x4567_0123));
        pac::FLASH.keyr().write(|w| w.set_keyr(0xCDEF_89AB));
    }

    #[cfg(any(flash_l0, flash_l1))]
    {
        pac::FLASH.pekeyr().write(|w| w.set_pekeyr(0x89ABCDEF));
        pac::FLASH.pekeyr().write(|w| w.set_pekeyr(0x02030405));

        pac::FLASH.prgkeyr().write(|w| w.set_prgkeyr(0x8C9DAEBF));
        pac::FLASH.prgkeyr().write(|w| w.set_prgkeyr(0x13141516));
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);

    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    pac::FLASH.cr().write(|w| w.set_pg(true));
}

pub(crate) unsafe fn disable_blocking_write() {
    #[cfg(any(flash_wl, flash_wb, flash_l4))]
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

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    #[cfg(any(flash_l0, flash_l1))]
    {
        pac::FLASH.pecr().modify(|w| {
            w.set_erase(true);
            w.set_prog(true);
        });

        write_volatile(sector.start as *mut u32, 0xFFFFFFFF);
    }

    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    {
        let idx = (sector.start - super::FLASH_BASE as u32) / super::BANK1_REGION.erase_size as u32;

        #[cfg(flash_l4)]
        let (idx, bank) = if idx > 255 { (idx - 256, true) } else { (idx, false) };

        pac::FLASH.cr().modify(|w| {
            w.set_per(true);
            w.set_pnb(idx as u8);
            #[cfg(any(flash_wl, flash_wb))]
            w.set_strt(true);
            #[cfg(any(flash_l4))]
            w.set_start(true);
            #[cfg(any(flash_l4))]
            w.set_bker(bank);
        });
    }

    let ret: Result<(), Error> = blocking_wait_ready();

    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    pac::FLASH.cr().modify(|w| w.set_per(false));

    #[cfg(any(flash_l0, flash_l1))]
    pac::FLASH.pecr().modify(|w| {
        w.set_erase(false);
        w.set_prog(false);
    });

    clear_all_err();
    ret
}

pub(crate) unsafe fn clear_all_err() {
    pac::FLASH.sr().modify(|w| {
        #[cfg(any(flash_wl, flash_wb, flash_l4, flash_l0))]
        if w.rderr() {
            w.set_rderr(true);
        }
        #[cfg(any(flash_wl, flash_wb, flash_l4))]
        if w.fasterr() {
            w.set_fasterr(true);
        }
        #[cfg(any(flash_wl, flash_wb, flash_l4))]
        if w.miserr() {
            w.set_miserr(true);
        }
        #[cfg(any(flash_wl, flash_wb, flash_l4))]
        if w.pgserr() {
            w.set_pgserr(true);
        }
        if w.sizerr() {
            w.set_sizerr(true);
        }
        if w.pgaerr() {
            w.set_pgaerr(true);
        }
        if w.wrperr() {
            w.set_wrperr(true);
        }
        #[cfg(any(flash_wl, flash_wb, flash_l4))]
        if w.progerr() {
            w.set_progerr(true);
        }
        #[cfg(any(flash_wl, flash_wb, flash_l4))]
        if w.operr() {
            w.set_operr(true);
        }
    });
}

unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.sr().read();

        if !sr.bsy() {
            #[cfg(any(flash_wl, flash_wb, flash_l4))]
            if sr.progerr() {
                return Err(Error::Prog);
            }

            if sr.wrperr() {
                return Err(Error::Protected);
            }

            if sr.pgaerr() {
                return Err(Error::Unaligned);
            }

            if sr.sizerr() {
                return Err(Error::Size);
            }

            #[cfg(any(flash_wl, flash_wb, flash_l4))]
            if sr.miserr() {
                return Err(Error::Miss);
            }

            #[cfg(any(flash_wl, flash_wb, flash_l4))]
            if sr.pgserr() {
                return Err(Error::Seq);
            }

            return Ok(());
        }
    }
}
