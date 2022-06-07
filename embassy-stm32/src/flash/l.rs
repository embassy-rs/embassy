use core::convert::TryInto;
use core::ptr::write_volatile;

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

pub(crate) unsafe fn blocking_write(offset: u32, buf: &[u8]) -> Result<(), Error> {
    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    pac::FLASH.cr().write(|w| w.set_pg(true));

    let ret = {
        let mut ret: Result<(), Error> = Ok(());
        let mut offset = offset;
        for chunk in buf.chunks(super::WRITE_SIZE) {
            for val in chunk.chunks(4) {
                write_volatile(
                    offset as *mut u32,
                    u32::from_le_bytes(val[0..4].try_into().unwrap()),
                );
                offset += val.len() as u32;
            }

            ret = blocking_wait_ready();
            if ret.is_err() {
                break;
            }
        }
        ret
    };

    #[cfg(any(flash_wl, flash_wb, flash_l4))]
    pac::FLASH.cr().write(|w| w.set_pg(false));

    ret
}

pub(crate) unsafe fn blocking_erase(from: u32, to: u32) -> Result<(), Error> {
    for page in (from..to).step_by(super::ERASE_SIZE) {
        #[cfg(any(flash_l0, flash_l1))]
        {
            pac::FLASH.pecr().modify(|w| {
                w.set_erase(true);
                w.set_prog(true);
            });

            write_volatile(page as *mut u32, 0xFFFFFFFF);
        }

        #[cfg(any(flash_wl, flash_wb, flash_l4))]
        {
            let idx = (page - super::FLASH_BASE as u32) / super::ERASE_SIZE as u32;

            #[cfg(flash_l4)]
            let (idx, bank) = if idx > 255 {
                (idx - 256, true)
            } else {
                (idx, false)
            };

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
        if ret.is_err() {
            return ret;
        }
    }

    Ok(())
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

pub(crate) unsafe fn blocking_wait_ready() -> Result<(), Error> {
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
