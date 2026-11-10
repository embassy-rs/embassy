//! Flash controller v4: STM32L4, L4+, L5, U5, U3, WB, WBA, WL, G0, G4, C0 and U0.
//!
//! Programming writes one double-word (or quad-word on U5/WBA) after setting `PG`. Pages are
//! erased by number (`PER` + `PNB` + `STRT`), with `BKER` selecting the physical bank on
//! dual-bank devices. All status flags are write-1-to-clear.
//!
//! The parts with TrustZone (L5, U5, WBA) are driven through their non-secure registers
//! (`NSKEYR`/`NSCR`/`NSSR`), the WL CM0+ core through `C2CR`/`C2SR`.

use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

#[allow(unused_imports)]
use super::FlashBank;
use super::{Config, Error, FlashSector, WRITE_SIZE};
use crate::pac;
use crate::pac::common::{RW, Reg};

const KEY1: u32 = 0x4567_0123;
const KEY2: u32 = 0xCDEF_89AB;

// Register access. The status and control registers have the same field names on all
// these chips, only the register names differ.

#[cfg(any(flash_v4_l5, flash_v4_u5, flash_v4_wba))]
mod regs {
    use super::*;
    pub(super) fn write_key(key: u32) {
        pac::FLASH.nskeyr().write_value(key);
    }
    pub(super) fn lock_cr() -> Reg<pac::flash::regs::Nscr, RW> {
        pac::FLASH.nscr()
    }
    pub(super) fn cr() -> Reg<pac::flash::regs::Nscr, RW> {
        pac::FLASH.nscr()
    }
    pub(super) fn sr() -> Reg<pac::flash::regs::Nssr, RW> {
        pac::FLASH.nssr()
    }
}

#[cfg(all(flash_v4_wl, feature = "_core-cm0p"))]
mod regs {
    use super::*;
    pub(super) fn write_key(key: u32) {
        pac::FLASH.keyr().write_value(key);
    }
    // CPU2 has its own control and status registers, but shares LOCK with CPU1.
    pub(super) fn lock_cr() -> Reg<pac::flash::regs::Cr, RW> {
        pac::FLASH.cr()
    }
    pub(super) fn cr() -> Reg<pac::flash::regs::C2cr, RW> {
        pac::FLASH.c2cr()
    }
    pub(super) fn sr() -> Reg<pac::flash::regs::C2sr, RW> {
        pac::FLASH.c2sr()
    }
}

#[cfg(not(any(flash_v4_l5, flash_v4_u5, flash_v4_wba, all(flash_v4_wl, feature = "_core-cm0p"))))]
mod regs {
    use super::*;
    pub(super) fn write_key(key: u32) {
        pac::FLASH.keyr().write_value(key);
    }
    pub(super) fn lock_cr() -> Reg<pac::flash::regs::Cr, RW> {
        pac::FLASH.cr()
    }
    pub(super) fn cr() -> Reg<pac::flash::regs::Cr, RW> {
        pac::FLASH.cr()
    }
    pub(super) fn sr() -> Reg<pac::flash::regs::Sr, RW> {
        pac::FLASH.sr()
    }
}

pub(super) unsafe fn unlock() {
    if regs::lock_cr().read().lock() {
        regs::write_key(KEY1);
        regs::write_key(KEY2);
    }
}

pub(super) unsafe fn lock() {
    regs::lock_cr().modify(|w| w.set_lock(true));
}

pub(super) unsafe fn start_write(address: u32, data: &[u8; WRITE_SIZE], interrupt: bool) {
    regs::cr().write(|w| {
        w.set_pg(true);
        w.set_eopie(interrupt);
        w.set_errie(interrupt);
    });
    // The words of a double/quad-word must be written back to back, the operation starts
    // after the last one.
    let mut address = address;
    for word in data.chunks_exact(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(word.try_into())));
        address += 4;
    }
    fence(Ordering::SeqCst);
}

pub(super) unsafe fn start_erase(sector: &FlashSector, _config: &Config, interrupt: bool) {
    regs::cr().write(|w| {
        w.set_per(true);
        w.set_pnb(sector.index_in_bank as u8);
        #[cfg(any(
            flash_v4_l4,
            flash_v4_l4p,
            flash_v4_g0x0,
            flash_v4_g0x1,
            flash_v4_g4c3,
            flash_v4_l5,
            flash_v4_u5,
            flash_v4_u3,
            flash_v4_wba
        ))]
        w.set_bker(sector.bank == FlashBank::Bank2);
        w.set_eopie(interrupt);
        w.set_errie(interrupt);
    });
    regs::cr().modify(|w| w.set_strt(true));
    fence(Ordering::SeqCst);
}

/// Whether an operation is ongoing. While this is true, the control register must not be
/// written: depending on the chip, that stalls the bus or causes a HardFault.
pub(super) fn busy() -> bool {
    let sr = regs::sr().read();

    #[cfg(any(flash_v4_g0x0, flash_v4_g0x1))]
    return sr.bsy1() || sr.bsy2() || sr.cfgbsy();
    #[cfg(any(flash_v4_c0, flash_v4_u0))]
    return sr.bsy1() || sr.cfgbsy();
    // PESD: CPU2 has suspended flash operations, CPU1 must not start one.
    #[cfg(any(flash_v4_wb, flash_v4_wl))]
    return sr.bsy() || sr.cfgbsy() || sr.pesd();
    #[cfg(any(flash_v4_u5, flash_v4_u3, flash_v4_wba))]
    return sr.bsy() || sr.wdw();
    #[cfg(any(flash_v4_l4, flash_v4_l4p, flash_v4_l5, flash_v4_g4c2, flash_v4_g4c3, flash_v4_g4c4))]
    return sr.bsy();
}

pub(super) fn wait_idle() {
    while busy() {}
}

pub(super) fn finish() -> Result<(), Error> {
    let sr = regs::sr().read();
    // Clear PG/PER and the interrupt enables. LOCK is write-1-only.
    regs::cr().modify(|w| {
        w.set_pg(false);
        w.set_per(false);
        w.set_strt(false);
        w.set_eopie(false);
        w.set_errie(false);
    });
    // All status flags are write-1-to-clear, the busy flags are read-only.
    regs::sr().write_value(sr);

    if sr.wrperr() {
        Err(Error::Protected)
    } else if sr.progerr() {
        Err(Error::NotErased)
    } else if sr.pgaerr() || sr.sizerr() || sr.pgserr() {
        warn!("flash error: SR={:08x}", sr.0);
        Err(Error::Hardware)
    } else {
        Ok(())
    }
}

#[cfg(flash_dual_bank)]
pub(super) fn banks_swapped() -> bool {
    #[cfg(any(flash_v4_l4, flash_v4_l4p, flash_v4_g4c3))]
    return pac::SYSCFG.memrmp().read().fb_mode();
    #[cfg(flash_v4_g0x1)]
    return !pac::FLASH.optr().read().n_swap_bank();
    #[cfg(any(flash_v4_l5, flash_v4_u5, flash_v4_u3, flash_v4_wba))]
    return pac::FLASH.optr().read().swap_bank();
    // RM0454 documents no bank swap option for G0B0.
    #[cfg(flash_v4_g0x0)]
    return false;
}

#[cfg(bank_setup_configurable)]
pub(super) fn dual_bank_mode() -> bool {
    let optr = pac::FLASH.optr().read();
    #[cfg(any(flash_v4_g0x0, flash_v4_g0x1))]
    return optr.dual_bank();
    #[cfg(any(flash_v4_l4, flash_v4_u5, flash_v4_u3, flash_v4_wba))]
    return optr.dualbank();
    #[cfg(flash_v4_g4c3)]
    return optr.dbank();
    // RM0438: on 256 KB devices the dual-bank layout also needs DB256K.
    #[cfg(flash_v4_l5)]
    return optr.dbank() && (super::FLASH_SIZE != 256 * 1024 || optr.db256k());
    // RM0432: DBANK selects the bank mode on the largest density of each line (2 MB L4R/S,
    // 1 MB L4P/Q), DB1M on the smaller one.
    #[cfg(flash_v4_l4p)]
    {
        let largest = if cfg!(any(stm32l4rx, stm32l4sx)) {
            2 * 1024 * 1024
        } else {
            1024 * 1024
        };
        return if super::FLASH_SIZE == largest {
            optr.dbank()
        } else {
            optr.db1m()
        };
    }
}

/// Whether a double ECC error has been detected since the flags were last cleared.
#[cfg(flash_ecc_nmi)]
pub(super) fn ecc_double_error() -> bool {
    #[cfg(flash_v4_u3)]
    return pac::FLASH.eccdetr().read().eccd();

    #[cfg(not(flash_v4_u3))]
    {
        let eccr = pac::FLASH.eccr().read();
        // ECCD2: double error in the upper double-word of a 128-bit read (single-bank mode).
        #[cfg(any(flash_v4_l4p, flash_v4_l5, flash_v4_g4c2, flash_v4_g4c3, flash_v4_g4c4))]
        let eccd2 = eccr.eccd2();
        #[cfg(not(any(flash_v4_l4p, flash_v4_l5, flash_v4_g4c2, flash_v4_g4c3, flash_v4_g4c4)))]
        let eccd2 = false;
        // G0 reports bank 2 errors in ECCR2.
        #[cfg(all(any(flash_v4_g0x0, flash_v4_g0x1), flash_dual_bank))]
        let bank2 = pac::FLASH.eccr2().read().eccd();
        #[cfg(not(all(any(flash_v4_g0x0, flash_v4_g0x1), flash_dual_bank)))]
        let bank2 = false;
        eccr.eccd() || eccd2 || bank2
    }
}

/// Clears the ECC correction and detection flags.
///
/// While a correction or detection flag is set, the hardware doesn't flag (nor raise the NMI
/// for) new double errors, so they must be cleared before reading.
#[cfg(flash_ecc_nmi)]
pub(super) fn ecc_clear() {
    // The flags are write-1-to-clear, the other fields are read-only or keep their value.
    #[cfg(flash_v4_u3)]
    {
        let r = pac::FLASH.ecccorr();
        r.write_value(r.read());
        let r = pac::FLASH.eccdetr();
        r.write_value(r.read());
    }
    #[cfg(not(flash_v4_u3))]
    {
        let r = pac::FLASH.eccr();
        r.write_value(r.read());
    }
    #[cfg(all(any(flash_v4_g0x0, flash_v4_g0x1), flash_dual_bank))]
    {
        let r = pac::FLASH.eccr2();
        r.write_value(r.read());
    }
}

/// On WB and WL, the flash from SFSA up is the secure area of CPU2 when security is enabled.
/// CPU1 accesses there cause a bus error instead of an error flag, so they are rejected.
#[cfg(all(any(flash_v4_wb, flash_v4_wl), not(feature = "_core-cm0p")))]
pub(super) fn check_secure_area(start: u32, end: u32) -> Result<(), Error> {
    let sfr = pac::FLASH.sfr().read();
    if sfr.fsd() {
        return Ok(());
    }
    let limit = super::FLASH_BANK1_BASE + sfr.sfsa() as u32 * super::BANK1_REGION.erase_size;
    if start >= limit || end > limit {
        Err(Error::Protected)
    } else {
        Ok(())
    }
}
