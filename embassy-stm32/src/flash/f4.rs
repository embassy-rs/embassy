use core::ptr::write_volatile;
use core::sync::atomic::{AtomicBool, Ordering, fence};

use embassy_sync::waitqueue::AtomicWaker;
use pac::flash::regs::Sr;

use super::{FlashBank, FlashSector, WRITE_SIZE, get_flash_regions};
use crate::_generated::FLASH_SIZE;
use crate::flash::Error;
use crate::pac;

static WAKER: AtomicWaker = AtomicWaker::new();
static DATA_CACHE_WAS_ENABLED: AtomicBool = AtomicBool::new(false);

impl FlashSector {
    const fn snb(&self) -> u8 {
        ((self.bank as u8) << 4) + self.index_in_bank
    }
}

pub(crate) unsafe fn on_interrupt() {
    // Clear IRQ flags
    pac::FLASH.sr().write(|w| {
        w.set_operr(true);
        w.set_eop(true);
    });

    WAKER.wake();
}

pub(crate) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
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
        w.set_psize(pac::flash::vals::Psize::PSIZE32);
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

    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(pac::flash::vals::Psize::PSIZE32);
    });
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
    blocking_wait_ready()
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
    save_data_cache_state();

    trace!("Erasing sector number {}", sector.snb());

    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(sector.snb());
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
    restore_data_cache_state();
    ret
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    save_data_cache_state();

    trace!("Blocking erasing sector number {}", sector.snb());

    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(sector.snb())
    });

    pac::FLASH.cr().modify(|w| {
        w.set_strt(true);
    });

    let ret: Result<(), Error> = blocking_wait_ready();
    clear_all_err();
    restore_data_cache_state();
    ret
}

pub(crate) fn clear_all_err() {
    // read and write back the same value.
    // This clears all "write 1 to clear" bits.
    pac::FLASH.sr().modify(|_| {});
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
            return Poll::Pending;
        }
    })
    .await
}

unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.sr().read();

        if !sr.bsy() {
            return get_result(sr);
        }
    }
}

fn get_result(sr: Sr) -> Result<(), Error> {
    if sr.pgserr() {
        Err(Error::Seq)
    } else if sr.pgperr() {
        Err(Error::Parallelism)
    } else if sr.pgaerr() {
        Err(Error::Unaligned)
    } else if sr.wrperr() {
        Err(Error::Protected)
    } else {
        Ok(())
    }
}

fn save_data_cache_state() {
    let dual_bank = unwrap!(get_flash_regions().last()).bank == FlashBank::Bank2;
    if dual_bank {
        // Disable data cache during write/erase if there are two banks, see errata 2.2.12
        let dcen = pac::FLASH.acr().read().dcen();
        DATA_CACHE_WAS_ENABLED.store(dcen, Ordering::Relaxed);
        if dcen {
            pac::FLASH.acr().modify(|w| w.set_dcen(false));
        }
    }
}

fn restore_data_cache_state() {
    let dual_bank = unwrap!(get_flash_regions().last()).bank == FlashBank::Bank2;
    if dual_bank {
        // Restore data cache if it was enabled
        let dcen = DATA_CACHE_WAS_ENABLED.load(Ordering::Relaxed);
        if dcen {
            // Reset data cache before we enable it again
            pac::FLASH.acr().modify(|w| w.set_dcrst(true));
            pac::FLASH.acr().modify(|w| w.set_dcrst(false));
            pac::FLASH.acr().modify(|w| w.set_dcen(true))
        }
    }
}

pub(crate) fn assert_not_corrupted_read(end_address: u32) {
    #[allow(unused)]
    const REVISION_3: u16 = 0x2001;

    #[allow(unused)]
    let second_bank_read =
        unwrap!(get_flash_regions().last()).bank == FlashBank::Bank2 && end_address > (FLASH_SIZE / 2) as u32;

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
    ))]
    if second_bank_read && pac::DBGMCU.idcode().read().rev_id() < REVISION_3 && !pa12_is_output_pull_low() {
        panic!(
            "Read corruption for stm32f42xxI and stm32f43xxI when PA12 is in use for chips below revision 3, see errata 2.2.11"
        );
    }

    #[cfg(any(
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
    ))]
    if second_bank_read && pac::DBGMCU.idcode().read().rev_id() < REVISION_3 && !pa12_is_output_pull_low() {
        panic!(
            "Read corruption for stm32f42xxG and stm32f43xxG in dual bank mode when PA12 is in use for chips below revision 3, see errata 2.2.11"
        );
    }
}

#[allow(unused)]
fn pa12_is_output_pull_low() -> bool {
    use pac::GPIOA;
    use pac::gpio::vals;
    const PIN: usize = 12;
    GPIOA.moder().read().moder(PIN) == vals::Moder::OUTPUT
        && GPIOA.pupdr().read().pupdr(PIN) == vals::Pupdr::PULL_DOWN
        && GPIOA.odr().read().odr(PIN) == vals::Odr::LOW
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flash::{FlashBank, get_sector};

    #[test]
    #[cfg(stm32f429)]
    fn can_get_sector() {
        const SMALL_SECTOR_SIZE: u32 = 16 * 1024;
        const MEDIUM_SECTOR_SIZE: u32 = 64 * 1024;
        const LARGE_SECTOR_SIZE: u32 = 128 * 1024;

        if !cfg!(feature = "dual-bank") {
            let assert_sector = |snb: u8, index_in_bank: u8, start: u32, size: u32, address: u32| {
                let sector = get_sector(address, crate::flash::get_flash_regions());
                assert_eq!(snb, sector.snb());
                assert_eq!(
                    FlashSector {
                        bank: sector.bank,
                        index_in_bank,
                        start,
                        size
                    },
                    sector
                );
            };

            assert_sector(0x00, 0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_0000);
            assert_sector(0x00, 0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_3FFF);
            assert_sector(0x03, 3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_C000);
            assert_sector(0x03, 3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_FFFF);

            assert_sector(0x04, 4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_0000);
            assert_sector(0x04, 4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_FFFF);

            assert_sector(0x05, 5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0802_0000);
            assert_sector(0x05, 5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0803_FFFF);
            assert_sector(0x0B, 11, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080E_0000);
            assert_sector(0x0B, 11, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080F_FFFF);
        } else {
            let assert_sector = |snb: u8, bank: FlashBank, index_in_bank: u8, start: u32, size: u32, address: u32| {
                let sector = get_sector(address, crate::flash::get_flash_regions());
                assert_eq!(snb, sector.snb());
                assert_eq!(
                    FlashSector {
                        bank,
                        index_in_bank,
                        start,
                        size
                    },
                    sector
                )
            };

            assert_sector(0x00, FlashBank::Bank1, 0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_0000);
            assert_sector(0x00, FlashBank::Bank1, 0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_3FFF);
            assert_sector(0x03, FlashBank::Bank1, 3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_C000);
            assert_sector(0x03, FlashBank::Bank1, 3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_FFFF);

            assert_sector(0x04, FlashBank::Bank1, 4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_0000);
            assert_sector(0x04, FlashBank::Bank1, 4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_FFFF);

            assert_sector(0x05, FlashBank::Bank1, 5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0802_0000);
            assert_sector(0x05, FlashBank::Bank1, 5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0803_FFFF);
            assert_sector(0x07, FlashBank::Bank1, 7, 0x0806_0000, LARGE_SECTOR_SIZE, 0x0806_0000);
            assert_sector(0x07, FlashBank::Bank1, 7, 0x0806_0000, LARGE_SECTOR_SIZE, 0x0807_FFFF);

            assert_sector(0x10, FlashBank::Bank2, 0, 0x0808_0000, SMALL_SECTOR_SIZE, 0x0808_0000);
            assert_sector(0x10, FlashBank::Bank2, 0, 0x0808_0000, SMALL_SECTOR_SIZE, 0x0808_3FFF);
            assert_sector(0x13, FlashBank::Bank2, 3, 0x0808_C000, SMALL_SECTOR_SIZE, 0x0808_C000);
            assert_sector(0x13, FlashBank::Bank2, 3, 0x0808_C000, SMALL_SECTOR_SIZE, 0x0808_FFFF);

            assert_sector(0x14, FlashBank::Bank2, 4, 0x0809_0000, MEDIUM_SECTOR_SIZE, 0x0809_0000);
            assert_sector(0x14, FlashBank::Bank2, 4, 0x0809_0000, MEDIUM_SECTOR_SIZE, 0x0809_FFFF);

            assert_sector(0x15, FlashBank::Bank2, 5, 0x080A_0000, LARGE_SECTOR_SIZE, 0x080A_0000);
            assert_sector(0x15, FlashBank::Bank2, 5, 0x080A_0000, LARGE_SECTOR_SIZE, 0x080B_FFFF);
            assert_sector(0x17, FlashBank::Bank2, 7, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080E_0000);
            assert_sector(0x17, FlashBank::Bank2, 7, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080F_FFFF);
        }
    }
}

#[cfg(all(bank_setup_configurable))]
pub(crate) fn check_bank_setup() {
    if cfg!(feature = "single-bank") && pac::FLASH.optcr().read().db1m() {
        panic!(
            "Embassy is configured as single-bank, but the hardware is running in dual-bank mode. Change the hardware by changing the db1m value in the user option bytes or configure embassy to use dual-bank config"
        );
    }
    if cfg!(feature = "dual-bank") && !pac::FLASH.optcr().read().db1m() {
        panic!(
            "Embassy is configured as dual-bank, but the hardware is running in single-bank mode. Change the hardware by changing the db1m value in the user option bytes or configure embassy to use single-bank config"
        );
    }
}
