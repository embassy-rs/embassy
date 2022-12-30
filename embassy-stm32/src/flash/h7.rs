use core::convert::TryInto;
use core::ptr::write_volatile;

use crate::flash::Error;
use crate::pac;

const SECOND_BANK_OFFSET: usize = 0x0010_0000;

const fn is_dual_bank() -> bool {
    super::FLASH_SIZE / 2 > super::ERASE_SIZE
}

pub(crate) unsafe fn lock() {
    pac::FLASH.bank(0).cr().modify(|w| w.set_lock(true));
    if is_dual_bank() {
        pac::FLASH.bank(1).cr().modify(|w| w.set_lock(true));
    }
}

pub(crate) unsafe fn unlock() {
    pac::FLASH.bank(0).keyr().write(|w| w.set_keyr(0x4567_0123));
    pac::FLASH.bank(0).keyr().write(|w| w.set_keyr(0xCDEF_89AB));

    if is_dual_bank() {
        pac::FLASH.bank(1).keyr().write(|w| w.set_keyr(0x4567_0123));
        pac::FLASH.bank(1).keyr().write(|w| w.set_keyr(0xCDEF_89AB));
    }
}

pub(crate) unsafe fn blocking_write(offset: u32, buf: &[u8]) -> Result<(), Error> {
    let bank = if !is_dual_bank() || (offset - super::FLASH_BASE as u32) < SECOND_BANK_OFFSET as u32 {
        pac::FLASH.bank(0)
    } else {
        pac::FLASH.bank(1)
    };

    bank.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(2); // 32 bits at once
    });

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

    let ret = {
        let mut ret: Result<(), Error> = Ok(());
        let mut offset = offset;
        'outer: for chunk in buf.chunks(super::WRITE_SIZE) {
            for val in chunk.chunks(4) {
                trace!("Writing at {:x}", offset);
                write_volatile(offset as *mut u32, u32::from_le_bytes(val[0..4].try_into().unwrap()));
                offset += val.len() as u32;

                ret = blocking_wait_ready(bank);
                bank.sr().modify(|w| {
                    if w.eop() {
                        w.set_eop(true);
                    }
                });
                if ret.is_err() {
                    break 'outer;
                }
            }
        }
        ret
    };

    bank.cr().write(|w| w.set_pg(false));

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

    ret
}

pub(crate) unsafe fn blocking_erase(from: u32, to: u32) -> Result<(), Error> {
    let from = from - super::FLASH_BASE as u32;
    let to = to - super::FLASH_BASE as u32;

    let (start, end) = if to <= super::FLASH_SIZE as u32 {
        let start_sector = from / super::ERASE_SIZE as u32;
        let end_sector = to / super::ERASE_SIZE as u32;
        (start_sector, end_sector)
    } else {
        error!("Attempting to write outside of defined sectors {:x} {:x}", from, to);
        return Err(Error::Unaligned);
    };

    trace!("Erasing sectors from {} to {}", start, end);
    for sector in start..end {
        let bank = if sector >= 8 { 1 } else { 0 };
        let ret = erase_sector(pac::FLASH.bank(bank), (sector % 8) as u8);
        if ret.is_err() {
            return ret;
        }
    }

    Ok(())
}

unsafe fn erase_sector(bank: pac::flash::Bank, sector: u8) -> Result<(), Error> {
    bank.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(sector)
    });

    bank.cr().modify(|w| {
        w.set_start(true);
    });

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
    bank.sr().modify(|w| {
        if w.wrperr() {
            w.set_wrperr(true);
        }
        if w.pgserr() {
            w.set_pgserr(true);
        }
        if w.strberr() {
            // single address was written multiple times, can be ignored
            w.set_strberr(true);
        }
        if w.incerr() {
            // writing to a different address when programming 256 bit word was not finished
            w.set_incerr(true);
        }
        if w.operr() {
            w.set_operr(true);
        }
        if w.sneccerr1() {
            // single ECC error
            w.set_sneccerr1(true);
        }
        if w.dbeccerr() {
            // double ECC error
            w.set_dbeccerr(true);
        }
        if w.rdperr() {
            w.set_rdperr(true);
        }
        if w.rdserr() {
            w.set_rdserr(true);
        }
    });
}

pub(crate) unsafe fn blocking_wait_ready(bank: pac::flash::Bank) -> Result<(), Error> {
    loop {
        let sr = bank.sr().read();

        if !sr.bsy() && !sr.qw() {
            if sr.wrperr() {
                return Err(Error::Protected);
            }
            if sr.pgserr() {
                error!("pgserr");
                return Err(Error::Seq);
            }
            if sr.incerr() {
                // writing to a different address when programming 256 bit word was not finished
                error!("incerr");
                return Err(Error::Seq);
            }
            if sr.operr() {
                return Err(Error::Prog);
            }
            if sr.sneccerr1() {
                // single ECC error
                return Err(Error::Prog);
            }
            if sr.dbeccerr() {
                // double ECC error
                return Err(Error::Prog);
            }
            if sr.rdperr() {
                return Err(Error::Protected);
            }
            if sr.rdserr() {
                return Err(Error::Protected);
            }

            return Ok(());
        }
    }
}
