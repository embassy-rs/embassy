use core::convert::TryInto;
use core::ptr::write_volatile;

use crate::flash::Error;
use crate::pac;

pub(crate) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    pac::FLASH.keyr().write(|w| w.set_fkeyr(0x4567_0123));
    pac::FLASH.keyr().write(|w| w.set_fkeyr(0xCDEF_89AB));
}

pub(crate) unsafe fn blocking_write(offset: u32, buf: &[u8]) -> Result<(), Error> {
    pac::FLASH.cr().write(|w| w.set_pg(true));

    let ret = {
        let mut ret: Result<(), Error> = Ok(());
        let mut offset = offset;
        for chunk in buf.chunks(2) {
            write_volatile(offset as *mut u16, u16::from_le_bytes(chunk[0..2].try_into().unwrap()));
            offset += chunk.len() as u32;

            ret = blocking_wait_ready();
            if ret.is_err() {
                break;
            }
        }
        ret
    };

    pac::FLASH.cr().write(|w| w.set_pg(false));

    ret
}

pub(crate) unsafe fn blocking_erase(from: u32, to: u32) -> Result<(), Error> {
    for page in (from..to).step_by(super::ERASE_SIZE) {
        pac::FLASH.cr().modify(|w| {
            w.set_per(true);
        });

        pac::FLASH.ar().write(|w| w.set_far(page));

        pac::FLASH.cr().modify(|w| {
            w.set_strt(true);
        });

        let mut ret: Result<(), Error> = blocking_wait_ready();

        if !pac::FLASH.sr().read().eop() {
            trace!("FLASH: EOP not set");
            ret = Err(Error::Prog);
        } else {
            pac::FLASH.sr().write(|w| w.set_eop(true));
        }

        pac::FLASH.cr().modify(|w| w.set_per(false));

        clear_all_err();
        if ret.is_err() {
            return ret;
        }
    }

    Ok(())
}

pub(crate) unsafe fn clear_all_err() {
    pac::FLASH.sr().modify(|w| {
        if w.pgerr() {
            w.set_pgerr(true);
        }
        if w.wrprterr() {
            w.set_wrprterr(true);
        }
        if w.eop() {
            w.set_eop(true);
        }
    });
}

pub(crate) unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.sr().read();

        if !sr.bsy() {
            if sr.wrprterr() {
                return Err(Error::Protected);
            }

            if sr.pgerr() {
                return Err(Error::Seq);
            }

            return Ok(());
        }
    }
}
