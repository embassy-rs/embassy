use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use pac::flash::regs::Nssr;
use stm32_metapac::flash::vals;

use super::{FlashSector, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

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
        r.set_bksel(bank_logical_to_physical(sector.bank));
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

/// Get the current SWAP_BANK option.
///
/// This value is only loaded on system or power-on reset. `perform_bank_swap()`
/// will not reflect here.
pub fn banks_swapped() -> bool {
    pac::FLASH.optcr().read().swap_bank() == vals::OptcrSwapBank::B0x1
}

/// Logical, persistent swap of flash banks 1 and 2.
///
/// This allows the application to write a new firmware blob into bank 2, then
/// swap the banks and perform a reset, loading the new firmware.
///
/// Swap does not take effect until system or power-on reset.
///
/// PLEASE READ THE REFERENCE MANUAL - there are nuances to this feature. For
/// instance, erase commands and interrupt enables which take a flash bank as a
/// parameter ignore the swap!
pub fn perform_bank_swap() {
    while busy() {}

    unsafe {
        clear_all_err();
    }

    // unlock OPTLOCK
    pac::FLASH.optkeyr().write(|w| *w = 0x0819_2A3B);
    pac::FLASH.optkeyr().write(|w| *w = 0x4C5D_6E7F);
    while pac::FLASH.optcr().read().optlock() {}

    let new_state = if banks_swapped() {
        vals::OptsrSwapBank::B0x0
    } else {
        vals::OptsrSwapBank::B0x1
    };

    // toggle SWAP_BANK option
    pac::FLASH.optsr_prg().modify(|w| w.set_swap_bank(new_state));

    // load option bytes
    pac::FLASH.optcr().modify(|w| w.set_optstrt(true));
    while pac::FLASH.optcr().read().optstrt() {}

    // re-lock OPTLOCK
    pac::FLASH.optcr().modify(|w| w.set_optlock(true));
}

fn sr_busy(sr: Nssr) -> bool {
    // Note: RM0492 sometimes incorrectly refers to WBNE as NSWBNE
    sr.bsy() || sr.dbne() || sr.wbne()
}

fn busy() -> bool {
    let sr = pac::FLASH.nssr().read();
    sr_busy(sr)
}

fn bank_logical_to_physical(logical: crate::flash::FlashBank) -> vals::NscrBksel {
    match (logical, banks_swapped()) {
        (crate::flash::FlashBank::Bank1, false) => vals::NscrBksel::B0x0, // Bank1
        (crate::flash::FlashBank::Bank1, true) => vals::NscrBksel::B0x1,  // Bank2
        (crate::flash::FlashBank::Bank2, false) => vals::NscrBksel::B0x1, // Bank2
        (crate::flash::FlashBank::Bank2, true) => vals::NscrBksel::B0x0,  // Bank1
        _ => unreachable!(),
    }
}
