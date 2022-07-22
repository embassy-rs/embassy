use core::convert::TryInto;
use core::ptr::write_volatile;
use core::task::Poll;

use atomic_polyfill::{fence, Ordering};
use embassy::waitqueue::AtomicWaker;
use futures::future::poll_fn;

use super::{ERASE_SIZE, FLASH_BASE, FLASH_SIZE};
use crate::flash::Error;
use crate::{interrupt, pac};

const SECOND_BANK_SECTOR_START: u32 = 12;

unsafe fn is_dual_bank() -> bool {
    match FLASH_SIZE / 1024 {
        // 1 MB devices depend on configuration
        1024 => {
            if cfg!(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479)) {
                pac::FLASH.optcr().read().db1m()
            } else {
                false
            }
        }
        // 2 MB devices are always dual bank
        2048 => true,
        // All other devices are single bank
        _ => false,
    }
}

pub(crate) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    pac::FLASH.keyr().write(|w| w.set_key(0x4567_0123));
    pac::FLASH.keyr().write(|w| w.set_key(0xCDEF_89AB));
}

pub(crate) unsafe fn blocking_write(offset: u32, buf: &[u8]) -> Result<(), Error> {
    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(pac::flash::vals::Psize::PSIZE32);
    });

    let ret = {
        let mut ret: Result<(), Error> = Ok(());
        let mut offset = offset;
        for chunk in buf.chunks(super::WRITE_SIZE) {
            for val in chunk.chunks(4) {
                write_volatile(offset as *mut u32, u32::from_le_bytes(val[0..4].try_into().unwrap()));
                offset += val.len() as u32;

                // prevents parallelism errors
                fence(Ordering::SeqCst);
            }

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

struct FlashSector {
    index: u8,
    size: u32,
}

fn get_sector(addr: u32, dual_bank: bool) -> FlashSector {
    let offset = addr - FLASH_BASE as u32;

    let bank_size = match dual_bank {
        true => FLASH_SIZE / 2,
        false => FLASH_SIZE,
    } as u32;

    let bank = offset / bank_size;
    let offset_in_bank = offset % bank_size;

    let index_in_bank = if offset_in_bank >= ERASE_SIZE as u32 / 2 {
        4 + offset_in_bank / ERASE_SIZE as u32
    } else {
        offset_in_bank / (ERASE_SIZE as u32 / 8)
    };

    // First 4 sectors are 16KB, then one 64KB, and rest are 128KB
    let size = match index_in_bank {
        0..=3 => 16 * 1024,
        4 => 64 * 1024,
        _ => 128 * 1024,
    };

    let index = if bank == 1 {
        SECOND_BANK_SECTOR_START + index_in_bank
    } else {
        index_in_bank
    } as u8;

    FlashSector { index, size }
}

pub(crate) unsafe fn blocking_erase(from: u32, to: u32) -> Result<(), Error> {
    let mut addr = from;
    let dual_bank = is_dual_bank();

    while addr < to {
        let sector = get_sector(addr, dual_bank);
        blocking_erase_sector(sector.index)?;
        addr += sector.size;
    }

    Ok(())
}

unsafe fn blocking_erase_sector(sector: u8) -> Result<(), Error> {
    let bank = sector / SECOND_BANK_SECTOR_START as u8;
    let snb = (bank << 4) + (sector % SECOND_BANK_SECTOR_START as u8);

    trace!("Erasing sector: {}", sector);

    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(snb)
    });

    pac::FLASH.cr().modify(|w| {
        w.set_strt(true);
    });

    let ret: Result<(), Error> = blocking_wait_ready();

    clear_all_err();

    ret
}

pub(crate) unsafe fn clear_all_err() {
    pac::FLASH.sr().write(|w| {
        w.set_pgserr(true);
        w.set_pgperr(true);
        w.set_pgaerr(true);
        w.set_wrperr(true);
    });
}

pub(crate) unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.sr().read();

        if !sr.bsy() {
            if sr.pgserr() {
                return Err(Error::Seq);
            }

            if sr.pgperr() {
                return Err(Error::Parallelism);
            }

            if sr.pgaerr() {
                return Err(Error::Unaligned);
            }

            if sr.wrperr() {
                return Err(Error::Protected);
            }

            return Ok(());
        }
    }
}

pub(crate) async unsafe fn write(offset: u32, buf: &[u8]) -> Result<(), Error> {
    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(pac::flash::vals::Psize::PSIZE32);
        w.set_eopie(true);
        w.set_errie(true);
    });

    let ret = {
        let mut ret: Result<(), Error> = Ok(());
        let mut offset = offset;
        for chunk in buf.chunks(super::WRITE_SIZE) {
            for val in chunk.chunks(4) {
                write_volatile(offset as *mut u32, u32::from_le_bytes(val[0..4].try_into().unwrap()));
                offset += val.len() as u32;

                // prevents parallelism errors
                fence(Ordering::SeqCst);
            }

            ret = wait_ready().await;

            if ret.is_err() {
                break;
            }
        }
        ret
    };

    pac::FLASH.cr().write(|w| {
        w.set_pg(false);
        w.set_eopie(false);
        w.set_errie(false);
    });

    ret
}

pub(crate) async unsafe fn erase(from: u32, to: u32) -> Result<(), Error> {
    let mut addr = from;
    let dual_bank = is_dual_bank();

    while addr < to {
        let sector = get_sector(addr, dual_bank);
        erase_sector(sector.index).await?;
        addr += sector.size;
    }

    Ok(())
}

async unsafe fn erase_sector(sector: u8) -> Result<(), Error> {
    let bank = sector / SECOND_BANK_SECTOR_START as u8;
    let snb = (bank << 4) + (sector % SECOND_BANK_SECTOR_START as u8);

    trace!("Erasing sector: {}", sector);

    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(snb);
        w.set_eopie(true);
        w.set_errie(true);
    });

    pac::FLASH.cr().modify(|w| {
        w.set_strt(true);
    });

    let ret: Result<(), Error> = wait_ready().await;

    pac::FLASH.cr().modify(|w| {
        w.set_eopie(false);
        w.set_errie(false);
    });

    clear_all_err();

    ret
}

static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) async unsafe fn wait_ready() -> Result<(), Error> {
    poll_fn(|cx| {
        WAKER.register(cx.waker());

        let sr = pac::FLASH.sr().read();

        if !sr.bsy() {
            Poll::Ready(if sr.pgserr() {
                Err(Error::Seq)
            } else if sr.pgperr() {
                Err(Error::Parallelism)
            } else if sr.pgaerr() {
                Err(Error::Unaligned)
            } else if sr.wrperr() {
                Err(Error::Protected)
            } else {
                Ok(())
            })
        } else {
            return Poll::Pending;
        }
    })
    .await
}

pub(crate) unsafe fn init() {
    use embassy_cortex_m::interrupt::{Interrupt, InterruptExt};
    crate::interrupt::FLASH::steal().enable();
}

#[interrupt]
unsafe fn FLASH() {
    // Clear IRQ flags
    pac::FLASH.sr().write(|w| {
        w.set_operr(true);
        w.set_eop(true);
    });

    WAKER.wake();
}
