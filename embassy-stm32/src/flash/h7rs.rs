use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use embassy_sync::waitqueue::AtomicWaker;
use pac::flash::regs::Isr;

use super::{FlashSector, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) unsafe fn on_interrupt() {
    // Deassert the end-of-program flag so the level interrupt does not re-fire.
    // The error flags are intentionally left in FLASH_ISR for the waiting task
    // to decode, so mask the interrupt sources instead of clearing them. They
    // are re-enabled by the next `enable_write`/`erase_sector`.
    pac::FLASH.icr().write(|w| w.set_eopf(true));
    disable_interrupts();

    WAKER.wake();
}

pub(crate) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    if pac::FLASH.cr().read().lock() {
        pac::FLASH.keyr().write(|w| w.set_cukey(0x4567_0123));
        pac::FLASH.keyr().write(|w| w.set_cukey(0xCDEF_89AB));
    }
}

pub(crate) unsafe fn enable_write() {
    enable_blocking_write();
    enable_interrupts();
}

pub(crate) unsafe fn disable_write() {
    disable_blocking_write();
    disable_interrupts();
}

unsafe fn enable_interrupts() {
    pac::FLASH.ier().modify(|w| {
        w.set_eopie(true);
        w.set_wrperrie(true);
        w.set_pgserrie(true);
        w.set_strberrie(true);
        w.set_incerrie(true);
    });
}

unsafe fn disable_interrupts() {
    pac::FLASH.ier().modify(|w| {
        w.set_eopie(false);
        w.set_wrperrie(false);
        w.set_pgserrie(false);
        w.set_strberrie(false);
        w.set_incerrie(false);
    });
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);
    pac::FLASH.cr().modify(|w| w.set_pg(true));
}

pub(crate) unsafe fn disable_blocking_write() {
    pac::FLASH.cr().modify(|w| w.set_pg(false));
}

pub(crate) async unsafe fn write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    write_raw(start_address, buf);
    wait_ready().await
}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    write_raw(start_address, buf);
    blocking_wait_ready()
}

/// Fill the 128-bit write buffer. The embedded flash starts programming once
/// the buffer is complete, so the four 32-bit write accesses must be issued in
/// order within the same 16-byte aligned flash word.
unsafe fn write_raw(start_address: u32, buf: &[u8; WRITE_SIZE]) {
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;
    }

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);
}

pub(crate) async unsafe fn erase_sector(sector: &FlashSector) -> Result<(), Error> {
    enable_interrupts();

    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_ssn(sector.index_in_bank);
    });
    pac::FLASH.cr().modify(|w| w.set_start(true));

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let ret = wait_ready().await;

    pac::FLASH.cr().modify(|w| w.set_ser(false));
    disable_interrupts();
    ret
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_ssn(sector.index_in_bank);
    });
    pac::FLASH.cr().modify(|w| w.set_start(true));

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let ret = blocking_wait_ready();

    pac::FLASH.cr().modify(|w| w.set_ser(false));
    ret
}

pub(crate) unsafe fn clear_all_err() {
    // ICR is not key-protected: write 1 to clear every flag.
    pac::FLASH.icr().write(|w| {
        w.set_eopf(true);
        w.set_wrperrf(true);
        w.set_pgserrf(true);
        w.set_strberrf(true);
        w.set_oblerrf(true);
        w.set_incerrf(true);
        w.set_rdserrf(true);
        w.set_sneccerrf(true);
        w.set_dbeccerrf(true);
        w.set_crcendf(true);
        w.set_crcrderrf(true);
    });
}

async fn wait_ready() -> Result<(), Error> {
    use core::future::poll_fn;
    use core::task::Poll;

    poll_fn(|cx| {
        WAKER.register(cx.waker());

        let sr = pac::FLASH.sr().read();
        if !sr.qw() && !sr.busy() {
            // Decode the flags before clearing them: `on_interrupt` only masks
            // the interrupt sources, so any error flag is still present here.
            let res = get_result(pac::FLASH.isr().read());
            unsafe { clear_all_err() };
            Poll::Ready(res)
        } else {
            Poll::Pending
        }
    })
    .await
}

unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.sr().read();
        if !sr.qw() && !sr.busy() {
            let res = get_result(pac::FLASH.isr().read());
            clear_all_err();
            return res;
        }
    }
}

fn get_result(isr: Isr) -> Result<(), Error> {
    if isr.wrperrf() {
        Err(Error::Protected)
    } else if isr.pgserrf() {
        error!("pgserrf");
        Err(Error::Seq)
    } else if isr.strberrf() {
        error!("strberrf");
        Err(Error::Seq)
    } else if isr.incerrf() {
        // writing to a different address when programming a 128-bit flash word
        error!("incerrf");
        Err(Error::Seq)
    } else if isr.crcrderrf() {
        error!("crcrderrf");
        Err(Error::Seq)
    } else if isr.rdserrf() {
        Err(Error::Protected)
    } else if isr.sneccerrf() {
        // single ECC error
        Err(Error::Prog)
    } else if isr.dbeccerrf() {
        // double ECC error
        Err(Error::Prog)
    } else if isr.oblerrf() {
        Err(Error::Seq)
    } else {
        Ok(())
    }
}
