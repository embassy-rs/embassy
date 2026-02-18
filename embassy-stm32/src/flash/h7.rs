use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use embassy_sync::waitqueue::AtomicWaker;
use pac::flash::regs::Sr;

use super::{BANK1_REGION, FLASH_REGIONS, FlashSector, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) unsafe fn on_interrupt() {
    // Clear IRQ flags
    pac::FLASH.bank(0).ccr().write(|w| {
        w.set_clr_eop(true);
        w.set_clr_operr(true);
    });
    if is_dual_bank() {
        pac::FLASH.bank(1).ccr().write(|w| {
            w.set_clr_eop(true);
            w.set_clr_operr(true);
        });
    }

    WAKER.wake();
}

const fn is_dual_bank() -> bool {
    FLASH_REGIONS.len() >= 2
}

pub(crate) unsafe fn lock() {
    pac::FLASH.bank(0).cr().modify(|w| w.set_lock(true));
    if is_dual_bank() {
        pac::FLASH.bank(1).cr().modify(|w| w.set_lock(true));
    }
}

pub(crate) unsafe fn unlock() {
    if pac::FLASH.bank(0).cr().read().lock() {
        pac::FLASH.bank(0).keyr().write_value(0x4567_0123);
        pac::FLASH.bank(0).keyr().write_value(0xCDEF_89AB);
    }
    if is_dual_bank() {
        if pac::FLASH.bank(1).cr().read().lock() {
            pac::FLASH.bank(1).keyr().write_value(0x4567_0123);
            pac::FLASH.bank(1).keyr().write_value(0xCDEF_89AB);
        }
    }
}

pub(crate) unsafe fn enable_write() {
    enable_blocking_write();
}

pub(crate) unsafe fn disable_write() {
    disable_blocking_write();
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);
}

pub(crate) unsafe fn disable_blocking_write() {}

pub(crate) async unsafe fn write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    // We cannot have the write setup sequence in begin_write as it depends on the address
    let bank = if start_address >= BANK1_REGION.base() && start_address < BANK1_REGION.end() {
        pac::FLASH.bank(0)
    } else {
        pac::FLASH.bank(1)
    };
    bank.cr().write(|w| {
        w.set_pg(true);
        #[cfg(flash_h7)]
        w.set_psize(2); // 32 bits at once
        w.set_eopie(true);
        w.set_operrie(true);
    });
    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let mut res = None;
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;

        res = Some(wait_ready(bank).await);
        bank.sr().modify(|w| {
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

    bank.cr().write(|w| {
        w.set_pg(false);
        w.set_eopie(false);
        w.set_operrie(false);
    });

    unwrap!(res)
}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    // We cannot have the write setup sequence in begin_write as it depends on the address
    let bank = if start_address >= BANK1_REGION.base() && start_address < BANK1_REGION.end() {
        pac::FLASH.bank(0)
    } else {
        pac::FLASH.bank(1)
    };
    bank.cr().write(|w| {
        w.set_pg(true);
        #[cfg(flash_h7)]
        w.set_psize(2); // 32 bits at once
    });
    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let mut res = None;
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;

        res = Some(blocking_wait_ready(bank));
        bank.sr().modify(|w| {
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

    bank.cr().write(|w| w.set_pg(false));

    unwrap!(res)
}

pub(crate) async unsafe fn erase_sector(sector: &FlashSector) -> Result<(), Error> {
    let bank = pac::FLASH.bank(sector.bank as usize);
    bank.cr().modify(|w| {
        w.set_ser(true);
        #[cfg(flash_h7)]
        w.set_snb(sector.index_in_bank);
        #[cfg(flash_h7ab)]
        w.set_ssn(sector.index_in_bank);
        w.set_eopie(true);
        w.set_operrie(true);
    });

    bank.cr().modify(|w| {
        w.set_start(true);
    });

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let ret: Result<(), Error> = wait_ready(bank).await;
    bank.cr().modify(|w| {
        w.set_ser(false);
        w.set_eopie(false);
        w.set_operrie(false);
    });
    bank_clear_all_err(bank);
    ret
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    let bank = pac::FLASH.bank(sector.bank as usize);
    bank.cr().modify(|w| {
        w.set_ser(true);
        #[cfg(flash_h7)]
        w.set_snb(sector.index_in_bank);
        #[cfg(flash_h7ab)]
        w.set_ssn(sector.index_in_bank);
    });

    bank.cr().modify(|w| {
        w.set_start(true);
    });

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let ret: Result<(), Error> = blocking_wait_ready(bank);
    bank.cr().modify(|w| w.set_ser(false));
    bank_clear_all_err(bank);
    ret
}

pub(crate) unsafe fn clear_all_err() {
    bank_clear_all_err(pac::FLASH.bank(0));
    bank_clear_all_err(pac::FLASH.bank(1));
}

unsafe fn bank_clear_all_err(bank: pac::flash::Bank) {
    // read and write back the same value.
    // This clears all "write 1 to clear" bits.
    bank.sr().modify(|_| {});
}

async fn wait_ready(bank: pac::flash::Bank) -> Result<(), Error> {
    use core::future::poll_fn;
    use core::task::Poll;

    poll_fn(|cx| {
        WAKER.register(cx.waker());

        let sr = bank.sr().read();
        if !sr.bsy() && !sr.qw() {
            Poll::Ready(get_result(sr))
        } else {
            return Poll::Pending;
        }
    })
    .await
}

unsafe fn blocking_wait_ready(bank: pac::flash::Bank) -> Result<(), Error> {
    loop {
        let sr = bank.sr().read();

        if !sr.bsy() && !sr.qw() {
            return get_result(sr);
        }
    }
}

fn get_result(sr: Sr) -> Result<(), Error> {
    if sr.wrperr() {
        Err(Error::Protected)
    } else if sr.pgserr() {
        error!("pgserr");
        Err(Error::Seq)
    } else if sr.incerr() {
        // writing to a different address when programming 256 bit word was not finished
        error!("incerr");
        Err(Error::Seq)
    } else if sr.crcrderr() {
        error!("crcrderr");
        Err(Error::Seq)
    } else if sr.operr() {
        Err(Error::Prog)
    } else if sr.sneccerr1() {
        // single ECC error
        Err(Error::Prog)
    } else if sr.dbeccerr() {
        // double ECC error
        Err(Error::Prog)
    } else if sr.rdperr() {
        Err(Error::Protected)
    } else if sr.rdserr() {
        Err(Error::Protected)
    } else {
        Ok(())
    }
}
