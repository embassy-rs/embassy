use core::ptr::write_volatile;
#[cfg(any(flash_g4c2, flash_g4c3, flash_g4c4))]
use core::sync::atomic::AtomicBool;
use core::sync::atomic::{Ordering, fence};

use cortex_m::interrupt;
use embassy_sync::waitqueue::AtomicWaker;
use pac::flash::regs::Sr;

use super::{FlashSector, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

static WAKER: AtomicWaker = AtomicWaker::new();

// G4 has data cache that needs to be handled during flash operations
#[cfg(any(flash_g4c2, flash_g4c3, flash_g4c4))]
static DATA_CACHE_WAS_ENABLED: AtomicBool = AtomicBool::new(false);

pub(crate) unsafe fn on_interrupt() {
    // Clear IRQ flags (EOP and error flags are write-1-to-clear)
    pac::FLASH.sr().write(|w| {
        w.set_eop(true);
        w.set_operr(true);
        w.set_progerr(true);
        w.set_wrperr(true);
        w.set_pgaerr(true);
        w.set_sizerr(true);
        w.set_pgserr(true);
    });

    WAKER.wake();
}

pub(crate) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    // Wait, while the memory interface is busy.
    wait_busy();

    // Unlock flash
    if pac::FLASH.cr().read().lock() {
        pac::FLASH.keyr().write_value(0x4567_0123);
        pac::FLASH.keyr().write_value(0xCDEF_89AB);
    }
}

pub(crate) unsafe fn enable_write() {
    assert_eq!(0, WRITE_SIZE % 4);
    save_data_cache_state();

    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_eopie(true);
        w.set_errie(true);
    });
}

pub(crate) unsafe fn disable_write() {
    pac::FLASH.cr().write(|w| {
        w.set_pg(false);
        w.set_eopie(false);
        w.set_errie(false);
    });
    restore_data_cache_state();
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);
    save_data_cache_state();
    pac::FLASH.cr().write(|w| w.set_pg(true));
}

pub(crate) unsafe fn disable_blocking_write() {
    pac::FLASH.cr().write(|w| w.set_pg(false));
    restore_data_cache_state();
}

pub(crate) async unsafe fn write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    write_start(start_address, buf);
    wait_ready().await
}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    write_start(start_address, buf);
    wait_ready_blocking()
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

pub(crate) async unsafe fn erase_sector(sector: &FlashSector) -> Result<(), Error> {
    wait_busy();
    clear_all_err();
    save_data_cache_state();

    interrupt::free(|_| {
        pac::FLASH.cr().modify(|w| {
            w.set_per(true);
            #[cfg(any(flash_g0x0, flash_g0x1, flash_g4c3))]
            w.set_bker(sector.bank == crate::flash::FlashBank::Bank2);
            #[cfg(flash_g0x0)]
            w.set_pnb(sector.index_in_bank as u16);
            #[cfg(not(flash_g0x0))]
            w.set_pnb(sector.index_in_bank as u8);
            w.set_eopie(true);
            w.set_errie(true);
            w.set_strt(true);
        });
    });

    let ret: Result<(), Error> = wait_ready().await;

    pac::FLASH.cr().modify(|w| {
        w.set_per(false);
        w.set_eopie(false);
        w.set_errie(false);
    });
    clear_all_err();
    restore_data_cache_state();

    ret
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    wait_busy();
    clear_all_err();
    save_data_cache_state();

    interrupt::free(|_| {
        pac::FLASH.cr().modify(|w| {
            w.set_per(true);
            #[cfg(any(flash_g0x0, flash_g0x1, flash_g4c3))]
            w.set_bker(sector.bank == crate::flash::FlashBank::Bank2);
            #[cfg(flash_g0x0)]
            w.set_pnb(sector.index_in_bank as u16);
            #[cfg(not(flash_g0x0))]
            w.set_pnb(sector.index_in_bank as u8);
            w.set_strt(true);
        });
    });

    let ret: Result<(), Error> = wait_ready_blocking();
    pac::FLASH.cr().modify(|w| w.set_per(false));
    clear_all_err();
    restore_data_cache_state();
    ret
}

pub(crate) async fn wait_ready() -> Result<(), Error> {
    use core::future::poll_fn;
    use core::task::Poll;

    poll_fn(|cx| {
        WAKER.register(cx.waker());

        let sr = pac::FLASH.sr().read();
        if !sr.bsy() {
            Poll::Ready(get_result(sr))
        } else {
            Poll::Pending
        }
    })
    .await
}

pub(crate) unsafe fn wait_ready_blocking() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.sr().read();
        if !sr.bsy() {
            return get_result(sr);
        }
    }
}

fn get_result(sr: Sr) -> Result<(), Error> {
    if sr.progerr() {
        Err(Error::Prog)
    } else if sr.wrperr() {
        Err(Error::Protected)
    } else if sr.pgaerr() {
        Err(Error::Unaligned)
    } else if sr.sizerr() {
        Err(Error::Size)
    } else if sr.pgserr() {
        Err(Error::Seq)
    } else {
        Ok(())
    }
}

pub(crate) unsafe fn clear_all_err() {
    // read and write back the same value.
    // This clears all "write 1 to clear" bits.
    pac::FLASH.sr().modify(|_| {});
}

#[cfg(any(flash_g0x0, flash_g0x1))]
fn wait_busy() {
    while pac::FLASH.sr().read().bsy() | pac::FLASH.sr().read().bsy2() {}
}

#[cfg(not(any(flash_g0x0, flash_g0x1)))]
fn wait_busy() {
    while pac::FLASH.sr().read().bsy() {}
}

// G4 data cache handling - must disable during flash operations
#[cfg(any(flash_g4c2, flash_g4c3, flash_g4c4))]
fn save_data_cache_state() {
    let dcen = pac::FLASH.acr().read().dcen();
    DATA_CACHE_WAS_ENABLED.store(dcen, Ordering::Relaxed);
    if dcen {
        pac::FLASH.acr().modify(|w| w.set_dcen(false));
    }
}

#[cfg(any(flash_g4c2, flash_g4c3, flash_g4c4))]
fn restore_data_cache_state() {
    // Restore data cache if it was enabled
    if DATA_CACHE_WAS_ENABLED.load(Ordering::Relaxed) {
        // Reset data cache before re-enabling
        pac::FLASH.acr().modify(|w| w.set_dcrst(true));
        pac::FLASH.acr().modify(|w| w.set_dcrst(false));
        pac::FLASH.acr().modify(|w| w.set_dcen(true));
    }
}

// G0 doesn't have data cache, use no-op functions
#[cfg(any(flash_g0x0, flash_g0x1))]
fn save_data_cache_state() {}

#[cfg(any(flash_g0x0, flash_g0x1))]
fn restore_data_cache_state() {}

#[cfg(all(bank_setup_configurable, any(flash_g4c2, flash_g4c3, flash_g4c4)))]
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

#[cfg(all(bank_setup_configurable, flash_g0x1))]
pub(crate) fn check_bank_setup() {
    if cfg!(feature = "single-bank") && pac::FLASH.optr().read().dual_bank() {
        panic!(
            "Embassy is configured as single-bank, but the hardware is running in dual-bank mode. Change the hardware by changing the dual_bank value in the user option bytes or configure embassy to use dual-bank config"
        );
    }
    if cfg!(feature = "dual-bank") && !pac::FLASH.optr().read().dual_bank() {
        panic!(
            "Embassy is configured as dual-bank, but the hardware is running in single-bank mode. Change the hardware by changing the dual_bank value in the user option bytes or configure embassy to use single-bank config"
        );
    }
}
