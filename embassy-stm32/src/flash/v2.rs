//! Flash controller v2: STM32F2, F4 and F7.
//!
//! Sectors of different sizes are erased by number (`SNB`). On dual-bank devices there is a
//! single register set and bit 4 of the sector number selects bank 2. Programming is done
//! one 32-bit word at a time, with `PSIZE` = x32.

use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use super::{Config, EraseParallelism, Error, FlashBank, FlashSector, WRITE_SIZE};
use crate::pac;
use crate::pac::flash::vals::Psize;

const KEY1: u32 = 0x4567_0123;
const KEY2: u32 = 0xCDEF_89AB;

impl EraseParallelism {
    fn psize(self) -> Psize {
        match self {
            Self::X8 => Psize::Psize8,
            Self::X16 => Psize::Psize16,
            Self::X32 => Psize::Psize32,
            Self::X64 => Psize::Psize64,
        }
    }
}

pub(super) unsafe fn unlock() {
    if pac::FLASH.cr().read().lock() {
        pac::FLASH.keyr().write_value(KEY1);
        pac::FLASH.keyr().write_value(KEY2);
    }
}

pub(super) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(super) unsafe fn start_write(address: u32, data: &[u8; WRITE_SIZE], interrupt: bool) {
    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(Psize::Psize32);
        w.set_eopie(interrupt);
        w.set_errie(interrupt);
    });
    write_volatile(address as *mut u32, u32::from_le_bytes(*data));
    // RM0410 §3.3.7: DSB before polling BSY.
    cortex_m::asm::dsb();
}

pub(super) unsafe fn start_erase(sector: &FlashSector, config: &Config, interrupt: bool) {
    let snb = match sector.bank {
        FlashBank::Bank2 => 0x10 | sector.index_in_bank as u8,
        _ => sector.index_in_bank as u8,
    };
    pac::FLASH.cr().write(|w| {
        w.set_ser(true);
        w.set_snb(snb);
        w.set_psize(config.erase_parallelism.psize());
        w.set_eopie(interrupt);
        w.set_errie(interrupt);
    });
    pac::FLASH.cr().modify(|w| w.set_strt(true));
    cortex_m::asm::dsb();
}

pub(super) fn busy() -> bool {
    pac::FLASH.sr().read().bsy()
}

pub(super) fn wait_idle() {
    while busy() {}
}

pub(super) fn finish() -> Result<(), Error> {
    let sr = pac::FLASH.sr().read();
    // Clear PG/SER and the interrupt enables. LOCK is write-1-only.
    pac::FLASH.cr().write(|_| {});
    // All status flags are write-1-to-clear.
    pac::FLASH.sr().write_value(sr);
    fence(Ordering::SeqCst);

    #[cfg(not(flash_v2_f7))]
    let sequence_error = sr.pgserr();
    #[cfg(flash_v2_f7)]
    let sequence_error = sr.erserr();

    if sr.wrperr() {
        Err(Error::Protected)
    } else if sequence_error || sr.pgaerr() || sr.pgperr() || sr.operr() {
        warn!("flash error: SR={:08x}", sr.0);
        Err(Error::Hardware)
    } else {
        Ok(())
    }
}

#[cfg(flash_dual_bank)]
pub(super) fn banks_swapped() -> bool {
    pac::SYSCFG.memrmp().read().fb_mode()
}

#[cfg(bank_setup_configurable)]
pub(super) fn dual_bank_mode() -> bool {
    #[cfg(flash_v2_f4)]
    return pac::FLASH.optcr().read().db1m();
    #[cfg(flash_v2_f7)]
    return !pac::FLASH.optcr().read().n_dbank();
}

/// Checks for the STM32F42x/F43x read corruption erratum (ES0206 2.2.11): on silicon before
/// revision 3, reads from flash bank 2 can be corrupted while PA12 is in use.
#[cfg(flash_v2_f4)]
pub(super) fn assert_not_corrupted_read(start_address: u32, len: u32) {
    #[cfg(any(
        feature = "stm32f427ai",
        feature = "stm32f427ii",
        feature = "stm32f427vi",
        feature = "stm32f427zi",
        feature = "stm32f429ai",
        feature = "stm32f429bi",
        feature = "stm32f429ii",
        feature = "stm32f429ni",
        feature = "stm32f429vi",
        feature = "stm32f429zi",
        feature = "stm32f437ai",
        feature = "stm32f437ii",
        feature = "stm32f437vi",
        feature = "stm32f437zi",
        feature = "stm32f439ai",
        feature = "stm32f439bi",
        feature = "stm32f439ii",
        feature = "stm32f439ni",
        feature = "stm32f439vi",
        feature = "stm32f439zi",
        all(
            feature = "dual-bank",
            any(
                feature = "stm32f427ag",
                feature = "stm32f427ig",
                feature = "stm32f427vg",
                feature = "stm32f427zg",
                feature = "stm32f429ag",
                feature = "stm32f429bg",
                feature = "stm32f429ig",
                feature = "stm32f429ng",
                feature = "stm32f429vg",
                feature = "stm32f429zg",
                feature = "stm32f437ig",
                feature = "stm32f437vg",
                feature = "stm32f437zg",
                feature = "stm32f439bg",
                feature = "stm32f439ig",
                feature = "stm32f439ng",
                feature = "stm32f439vg",
                feature = "stm32f439zg",
            )
        )
    ))]
    {
        const REVISION_3: u16 = 0x2001;

        let bank2 = FlashBank::Bank2.base();
        let touches_bank2 = start_address < bank2 + (super::FLASH_SIZE as u32 / 2) && start_address + len > bank2;
        if touches_bank2 && pac::DBGMCU.idcode().read().rev_id() < REVISION_3 && !pa12_is_output_pull_low() {
            panic!(
                "Read corruption for stm32f42x and stm32f43x in dual bank mode when PA12 is in use for chips below revision 3, see errata 2.2.11"
            );
        }
    }

    let _ = (start_address, len);
}

#[cfg(flash_v2_f4)]
#[allow(unused)]
fn pa12_is_output_pull_low() -> bool {
    use pac::GPIOA;
    use pac::gpio::vals;
    const PIN: usize = 12;
    GPIOA.moder().read().moder(PIN) == vals::Moder::Output
        && GPIOA.pupdr().read().pupdr(PIN) == vals::Pupdr::PullDown
        && GPIOA.odr().read().odr(PIN) == vals::Odr::Low
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flash::{FlashBank, get_sector};

    const K: u32 = 1024;

    fn assert_sector(bank: FlashBank, index_in_bank: u32, start: u32, size: u32, address: u32) {
        assert_eq!(
            FlashSector {
                bank,
                index_in_bank,
                start,
                size
            },
            unwrap!(get_sector(address, crate::flash::get_flash_regions()))
        )
    }

    /// Sectors of a bank with 4 small, 1 medium and `large` large sectors of the given sizes.
    fn assert_bank(bank: FlashBank, base: u32, small: u32, large: u32) {
        let medium = 4 * small;
        let large_size = 8 * small;
        for i in 0..4 {
            let start = base + i * small;
            assert_sector(bank, i, start, small, start);
            assert_sector(bank, i, start, small, start + small - 1);
        }
        let start = base + 4 * small;
        assert_sector(bank, 4, start, medium, start);
        assert_sector(bank, 4, start, medium, start + medium - 1);
        for i in 0..large {
            let start = base + 8 * small + i * large_size;
            assert_sector(bank, 5 + i, start, large_size, start);
            assert_sector(bank, 5 + i, start, large_size, start + large_size - 1);
        }
    }

    #[test]
    #[cfg(all(stm32f429, feature = "single-bank"))]
    fn f429_1m_single_bank() {
        assert_bank(FlashBank::Bank1, 0x0800_0000, 16 * K, 7);
    }

    #[test]
    #[cfg(all(stm32f429, feature = "dual-bank"))]
    fn f429_1m_dual_bank() {
        assert_bank(FlashBank::Bank1, 0x0800_0000, 16 * K, 3);
        assert_bank(FlashBank::Bank2, 0x0808_0000, 16 * K, 3);
    }

    #[test]
    #[cfg(stm32f732)]
    fn f732() {
        assert_bank(FlashBank::Bank1, 0x0800_0000, 16 * K, 3);
    }

    #[test]
    #[cfg(all(stm32f769, feature = "single-bank"))]
    fn f769_single_bank() {
        assert_bank(FlashBank::Bank1, 0x0800_0000, 32 * K, 7);
    }

    #[test]
    #[cfg(all(stm32f769, feature = "dual-bank"))]
    fn f769_dual_bank() {
        assert_bank(FlashBank::Bank1, 0x0800_0000, 16 * K, 7);
        assert_bank(FlashBank::Bank2, 0x0810_0000, 16 * K, 7);
    }
}
