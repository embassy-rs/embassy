use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use super::{FlashBank, FlashSector, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

pub(crate) unsafe fn lock() {
    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.seccr().modify(|w| w.set_lock(true));
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.nscr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    #[cfg(feature = "trustzone-secure")]
    if pac::FLASH.seccr().read().lock() {
        pac::FLASH.seckeyr().write_value(0x4567_0123);
        pac::FLASH.seckeyr().write_value(0xCDEF_89AB);
    }
    #[cfg(not(feature = "trustzone-secure"))]
    if pac::FLASH.nscr().read().lock() {
        pac::FLASH.nskeyr().write_value(0x4567_0123);
        pac::FLASH.nskeyr().write_value(0xCDEF_89AB);
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);

    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.seccr().write(|w| {
        w.set_pg(true);
    });
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.nscr().write(|w| {
        w.set_pg(true);
    });
}

pub(crate) unsafe fn disable_blocking_write() {
    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.seccr().write(|w| w.set_pg(false));
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.nscr().write(|w| w.set_pg(false));
}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
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
    pac::FLASH.seccr().modify(|w| {
        w.set_per(pac::flash::vals::SeccrPer::B_0X1);
        w.set_pnb(sector.index_in_bank);
        // TODO: add check for bank swap
        w.set_bker(match sector.bank {
            FlashBank::Bank1 => false,
            FlashBank::Bank2 => true,
            _ => unreachable!(),
        });
    });
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.nscr().modify(|w| {
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
    pac::FLASH.seccr().modify(|w| {
        w.set_strt(true);
    });
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.nscr().modify(|w| {
        w.set_strt(true);
    });

    let ret: Result<(), Error> = blocking_wait_ready();
    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.seccr().modify(|w| w.set_per(false));
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.nscr().modify(|w| w.set_per(false));
    clear_all_err();
    ret
}

pub(crate) unsafe fn clear_all_err() {
    // read and write back the same value.
    // This clears all "write 1 to clear" bits.
    #[cfg(feature = "trustzone-secure")]
    pac::FLASH.secsr().modify(|_| {});
    #[cfg(not(feature = "trustzone-secure"))]
    pac::FLASH.nssr().modify(|_| {});
}

unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        #[cfg(feature = "trustzone-secure")]
        let sr = pac::FLASH.secsr().read();
        #[cfg(not(feature = "trustzone-secure"))]
        let sr = pac::FLASH.nssr().read();

        if !sr.bsy() {
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

            return Ok(());
        }
    }
}
