use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use super::{FlashRegion, FlashSector, FLASH_REGIONS, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

pub(crate) const fn is_default_layout() -> bool {
    true
}

// const fn is_dual_bank() -> bool {
//     FLASH_REGIONS.len() >= 2
// }

pub(crate) fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
}

pub(crate) unsafe fn lock() {
    if !pac::FLASH.nscr().read().lock() {
        pac::FLASH.nscr().modify(|r| {
            r.set_lock(true);
        });
    }
}

pub(crate) unsafe fn unlock() {
    // TODO: check locked first
    while pac::FLASH.nssr().read().bsy() {
        #[cfg(feature = "defmt")]
        defmt::trace!("busy")
    }

    // only unlock if locked to begin with
    if pac::FLASH.nscr().read().lock() {
        pac::FLASH.nskeyr().write_value(0x4567_0123);
        pac::FLASH.nskeyr().write_value(0xCDEF_89AB);
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);
}

pub(crate) unsafe fn disable_blocking_write() {}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    // // We cannot have the write setup sequence in begin_write as it depends on the address
    // let bank = if start_address < BANK1_REGION.end() {
    //     pac::FLASH.bank(0)
    // } else {
    //     pac::FLASH.bank(1)
    // };

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    clear_all_err();

    pac::FLASH.nscr().write(|w| {
        w.set_pg(true);
        // w.set_psize(2); // 32 bits at once
    });

    let mut res = None;
    let mut address = start_address;
    // TODO: see write size
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;

        res = Some(blocking_wait_ready().map_err(|e| {
            error!("write err");
            e
        }));
        pac::FLASH.nssr().modify(|w| {
            if w.eop() {
                w.set_eop(true);
            }
        });
        if unwrap!(res).is_err() {
            break;
        }
    }

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    pac::FLASH.nscr().write(|w| w.set_pg(false));

    unwrap!(res)
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    // pac::FLASH.wrp2r_cur().read().wrpsg()
    // TODO: write protection check
    if pac::FLASH.nscr().read().lock() == true {
        error!("flash locked");
    }

    loop {
        let sr = pac::FLASH.nssr().read();
        if !sr.bsy() && !sr.dbne() {
            break;
        }
    }
    clear_all_err();

    pac::FLASH.nscr().modify(|r| {
        // TODO: later check bank swap
        r.set_bksel(match sector.bank {
            crate::flash::FlashBank::Bank1 => stm32_metapac::flash::vals::NscrBksel::B_0X0,
            crate::flash::FlashBank::Bank2 => stm32_metapac::flash::vals::NscrBksel::B_0X1,
            _ => unreachable!(),
        });
        r.set_snb(sector.index_in_bank);
        r.set_ser(true);
    });

    pac::FLASH.nscr().modify(|r| {
        r.set_strt(true);
    });

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let ret: Result<(), Error> = blocking_wait_ready().map_err(|e| {
        error!("erase err");
        e
    });

    pac::FLASH.nscr().modify(|w| w.set_ser(false));
    clear_all_err();
    ret
}

pub(crate) unsafe fn clear_all_err() {
    pac::FLASH.nssr().modify(|_| {})
}

unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.nssr().read();

        if !sr.bsy() {
            if sr.optchangeerr() {
                error!("optchangeerr");
                return Err(Error::Prog);
            }
            if sr.obkwerr() {
                error!("obkwerr");
                return Err(Error::Seq);
            }
            if sr.obkerr() {
                error!("obkerr");
                return Err(Error::Seq);
            }
            if sr.incerr() {
                error!("incerr");
                return Err(Error::Unaligned);
            }
            if sr.strberr() {
                error!("strberr");
                return Err(Error::Parallelism);
            }
            if sr.wrperr() {
                error!("protected");
                return Err(Error::Protected);
            }

            return Ok(());
        }
    }
}
