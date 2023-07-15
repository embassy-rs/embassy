use core::convert::TryInto;
use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use super::{FlashRegion, FlashSector, BANK1_REGION, FLASH_REGIONS, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

pub const fn is_default_layout() -> bool {
    true
}

const fn is_dual_bank() -> bool {
    FLASH_REGIONS.len() == 2
}

pub fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
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

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);
}

pub(crate) unsafe fn disable_blocking_write() {}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    // We cannot have the write setup sequence in begin_write as it depends on the address
    let bank = if start_address < BANK1_REGION.end() {
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
    fence(Ordering::SeqCst);

    let mut res = None;
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(val.try_into().unwrap()));
        address += val.len() as u32;

        res = Some(blocking_wait_ready(bank));
        bank.sr().modify(|w| {
            if w.eop() {
                w.set_eop(true);
            }
        });
        if res.unwrap().is_err() {
            break;
        }
    }

    bank.cr().write(|w| w.set_pg(false));

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    res.unwrap()
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    let bank = pac::FLASH.bank(sector.bank as usize);
    bank.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(sector.index_in_bank)
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

unsafe fn blocking_wait_ready(bank: pac::flash::Bank) -> Result<(), Error> {
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
