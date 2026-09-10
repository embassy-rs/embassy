use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use embassy_hal_internal::drop::OnDrop;

use super::{Error, FlashBank, FlashSector, WRITE_SIZE, get_flash_regions};
use crate::pac;

pub(crate) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
    pac::FLASH.cr2().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    // PM0068 Rev 2, Sections 2.3.1-2.3.2 and 3.9: each bank has its own key register.
    if pac::FLASH.cr().read().lock() {
        pac::FLASH.keyr().write_value(0x4567_0123);
        pac::FLASH.keyr().write_value(0xcdef_89ab);
    }
    if pac::FLASH.cr2().read().lock() {
        pac::FLASH.keyr2().write_value(0x4567_0123);
        pac::FLASH.keyr2().write_value(0xcdef_89ab);
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(WRITE_SIZE % 2, 0);
    pac::FLASH.cr().modify(|w| w.set_pg(true));
    pac::FLASH.cr2().modify(|w| w.set_pg(true));
}

pub(crate) unsafe fn disable_blocking_write() {
    pac::FLASH.cr().modify(|w| w.set_pg(false));
    pac::FLASH.cr2().modify(|w| w.set_pg(false));
}

#[inline]
fn status(bank: FlashBank) -> pac::common::Reg<pac::flash::regs::Sr, pac::common::RW> {
    match bank {
        FlashBank::Bank1 => pac::FLASH.sr(),
        FlashBank::Bank2 => pac::FLASH.sr2(),
        _ => panic!("F1 main Flash transactions do not support OTP"),
    }
}

fn bank_for_range(address: u32, size: usize) -> Result<FlashBank, Error> {
    let end = address
        .checked_add(size.try_into().map_err(|_| Error::Size)?)
        .ok_or(Error::Size)?;
    get_flash_regions()
        .iter()
        .find(|r| address >= r.base() && address < r.end() && end <= r.end())
        .map(|r| r.bank)
        .filter(|b| matches!(b, FlashBank::Bank1 | FlashBank::Bank2))
        .ok_or(Error::Size)
}

#[inline]
fn clear_status(bank: FlashBank) {
    // Write only documented W1C flags, without echoing busy or reserved bits.
    status(bank).write(|w| {
        w.set_eop(true);
        w.set_pgerr(true);
        w.set_wrprterr(true);
    });
}

fn wait_ready(bank: FlashBank) -> Result<(), Error> {
    loop {
        let sr = status(bank).read();
        if !sr.bsy() {
            return if sr.wrprterr() {
                Err(Error::Protected)
            } else if sr.pgerr() {
                Err(Error::Seq)
            } else {
                Ok(())
            };
        }
    }
}

pub(crate) unsafe fn blocking_write(address: u32, bytes: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    if bytes.len() % 2 != 0 || address % 2 != 0 {
        return Err(Error::Unaligned);
    }
    let bank = bank_for_range(address, bytes.len())?;
    check_unlocked(bank)?;
    let _cleanup = OnDrop::new(|| clear_status(bank));
    // PM0068 Rev 2, Section 2.3.3: programming uses halfword stores and per-bank status.
    for (index, chunk) in bytes.chunks_exact(2).enumerate() {
        // SAFETY: the range is within the selected Flash bank, each address is halfword aligned,
        // and the caller has exclusive controller access with programming enabled.
        unsafe {
            write_volatile(
                (address + index as u32 * 2) as *mut u16,
                u16::from_le_bytes([chunk[0], chunk[1]]),
            )
        };
        fence(Ordering::SeqCst);
        wait_ready(bank)?;
    }
    Ok(())
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    let bank = bank_for_range(sector.start, sector.size as usize)?;
    if bank != sector.bank {
        return Err(Error::Size);
    }
    check_unlocked(bank)?;
    let _cleanup = OnDrop::new(|| {
        match bank {
            FlashBank::Bank1 => pac::FLASH.cr().modify(|w| w.set_per(false)),
            FlashBank::Bank2 => pac::FLASH.cr2().modify(|w| w.set_per(false)),
            _ => unreachable!(),
        }
        clear_status(bank);
    });
    // PM0068 Rev 2, Sections 2.3.4, 3.11 and 3.12: select PER, page address, then STRT.
    match bank {
        FlashBank::Bank1 => {
            pac::FLASH.cr().modify(|w| w.set_per(true));
            pac::FLASH.ar().write(|w| w.set_far(sector.start));
            pac::FLASH.cr().modify(|w| w.set_strt(true));
            pac::FLASH.cr().read();
        }
        FlashBank::Bank2 => {
            pac::FLASH.cr2().modify(|w| w.set_per(true));
            pac::FLASH.ar2().write(|w| w.set_far(sector.start));
            pac::FLASH.cr2().modify(|w| w.set_strt(true));
            pac::FLASH.cr2().read();
        }
        _ => return Err(Error::Size),
    }
    wait_ready(bank)?;
    if !status(bank).read().eop() {
        return Err(Error::Prog);
    }
    Ok(())
}

#[inline]
fn check_unlocked(bank: FlashBank) -> Result<(), Error> {
    let locked = match bank {
        FlashBank::Bank1 => pac::FLASH.cr().read().lock(),
        FlashBank::Bank2 => pac::FLASH.cr2().read().lock(),
        _ => return Err(Error::Size),
    };
    if locked { Err(Error::Protected) } else { Ok(()) }
}

pub(crate) unsafe fn clear_all_err() {
    for bank in [FlashBank::Bank1, FlashBank::Bank2] {
        while status(bank).read().bsy() {}
        clear_status(bank);
    }
}
