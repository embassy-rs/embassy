use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use super::{FlashRegion, FlashSector, FLASH_REGIONS, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

pub const fn is_default_layout() -> bool {
    true
}

pub const fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
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

    wait_ready_blocking()
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

    let ret: Result<(), Error> = wait_ready_blocking();

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
    // read and write back the same value.
    // This clears all "write 0 to clear" bits.
    pac::FLASH.sr().modify(|_| {});
}

unsafe fn wait_ready_blocking() -> Result<(), Error> {
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
