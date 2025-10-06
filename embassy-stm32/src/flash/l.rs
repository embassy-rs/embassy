use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use super::{FlashSector, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

pub(crate) unsafe fn lock() {
    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    pac::FLASH.cr().modify(|w| w.set_lock(true));

    #[cfg(any(flash_l0))]
    pac::FLASH.pecr().modify(|w| {
        w.set_optlock(true);
        w.set_prglock(true);
        w.set_pelock(true);
    });

    #[cfg(any(flash_l5))]
    pac::FLASH.nscr().modify(|w| w.set_nslock(true));
}

pub(crate) unsafe fn unlock() {
    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    {
        if pac::FLASH.cr().read().lock() {
            pac::FLASH.keyr().write_value(0x4567_0123);
            pac::FLASH.keyr().write_value(0xCDEF_89AB);
        }
    }

    #[cfg(any(flash_l0, flash_l1))]
    {
        if pac::FLASH.pecr().read().pelock() {
            pac::FLASH.pekeyr().write_value(0x89AB_CDEF);
            pac::FLASH.pekeyr().write_value(0x0203_0405);
        }

        if pac::FLASH.pecr().read().prglock() {
            pac::FLASH.prgkeyr().write_value(0x8C9D_AEBF);
            pac::FLASH.prgkeyr().write_value(0x1314_1516);
        }
    }

    #[cfg(any(flash_l5))]
    {
        if pac::FLASH.nscr().read().nslock() {
            pac::FLASH.nskeyr().write_value(0x4567_0123);
            pac::FLASH.nskeyr().write_value(0xCDEF_89AB);
        }
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);

    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    pac::FLASH.cr().write(|w| w.set_pg(true));

    #[cfg(any(flash_l5))]
    pac::FLASH.nscr().write(|w| w.set_nspg(true));
}

pub(crate) unsafe fn disable_blocking_write() {
    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    pac::FLASH.cr().write(|w| w.set_pg(false));

    #[cfg(any(flash_l5))]
    pac::FLASH.nscr().write(|w| w.set_nspg(false));
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
    #[cfg(any(flash_l0, flash_l1))]
    {
        pac::FLASH.pecr().modify(|w| {
            w.set_erase(true);
            w.set_prog(true);
        });

        write_volatile(sector.start as *mut u32, 0xFFFFFFFF);
    }

    #[cfg(any(flash_wl, flash_wb, flash_l4, flash_l5))]
    {
        let idx = (sector.start - super::FLASH_BASE as u32) / super::BANK1_REGION.erase_size as u32;

        #[cfg(flash_l4)]
        let (idx, bank) = if idx > 255 { (idx - 256, true) } else { (idx, false) };

        #[cfg(flash_l5)]
        let (idx, bank) = if pac::FLASH.optr().read().dbank() {
            if idx > 255 {
                (idx - 256, Some(true))
            } else {
                (idx, Some(false))
            }
        } else {
            (idx, None)
        };

        #[cfg(not(flash_l5))]
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

        #[cfg(flash_l5)]
        pac::FLASH.nscr().modify(|w| {
            w.set_nsper(true);
            w.set_nspnb(idx as u8);
            if let Some(bank) = bank {
                w.set_nsbker(bank);
            }
            w.set_nsstrt(true);
        });
    }

    let ret: Result<(), Error> = wait_ready_blocking();

    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    pac::FLASH.cr().modify(|w| w.set_per(false));

    #[cfg(any(flash_l5))]
    pac::FLASH.nscr().modify(|w| w.set_nsper(false));

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
    // This clears all "write 1 to clear" bits.
    #[cfg(not(flash_l5))]
    pac::FLASH.sr().modify(|_| {});

    #[cfg(flash_l5)]
    pac::FLASH.nssr().modify(|_| {});
}

pub(crate) unsafe fn wait_ready_blocking() -> Result<(), Error> {
    loop {
        #[cfg(not(flash_l5))]
        {
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

        #[cfg(flash_l5)]
        {
            let nssr = pac::FLASH.nssr().read();

            if !nssr.nsbsy() {
                if nssr.nsprogerr() {
                    return Err(Error::Prog);
                }

                if nssr.nswrperr() {
                    return Err(Error::Protected);
                }

                if nssr.nspgaerr() {
                    return Err(Error::Unaligned);
                }

                if nssr.nssizerr() {
                    return Err(Error::Size);
                }

                if nssr.nspgserr() {
                    return Err(Error::Seq);
                }

                return Ok(());
            }
        }
    }
}

#[cfg(all(bank_setup_configurable, flash_l5))]
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

#[cfg(all(bank_setup_configurable, flash_l4))]
pub(crate) fn check_bank_setup() {
    if cfg!(feature = "single-bank") && pac::FLASH.optr().read().dualbank() {
        panic!(
            "Embassy is configured as single-bank, but the hardware is running in dual-bank mode. Change the hardware by changing the dualbank value in the user option bytes or configure embassy to use dual-bank config"
        );
    }
    if cfg!(feature = "dual-bank") && !pac::FLASH.optr().read().dualbank() {
        panic!(
            "Embassy is configured as dual-bank, but the hardware is running in single-bank mode. Change the hardware by changing the dualbank value in the user option bytes or configure embassy to use single-bank config"
        );
    }
}
