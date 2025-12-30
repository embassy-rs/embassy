use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use super::{FlashSector, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

impl FlashSector {
    const fn snb(&self) -> u8 {
        ((self.bank as u8) << 4) + self.index_in_bank
    }
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

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);

    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(pac::flash::vals::Psize::PSIZE32);
    });
}

pub(crate) unsafe fn disable_blocking_write() {
    pac::FLASH.cr().write(|w| w.set_pg(false));
}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;

        // prevents parallelism errors
        fence(Ordering::SeqCst);
    }

    blocking_wait_ready()
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(sector.snb())
    });

    pac::FLASH.cr().modify(|w| {
        w.set_strt(true);
    });

    let ret: Result<(), Error> = blocking_wait_ready();
    pac::FLASH.cr().modify(|w| w.set_ser(false));
    clear_all_err();
    ret
}

pub(crate) unsafe fn clear_all_err() {
    // read and write back the same value.
    // This clears all "write 1 to clear" bits.
    pac::FLASH.sr().modify(|_| {});
}

unsafe fn blocking_wait_ready() -> Result<(), Error> {
    loop {
        let sr = pac::FLASH.sr().read();

        if !sr.bsy() {
            if sr.erserr() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flash::{FlashBank, get_sector};

    #[test]
    #[cfg(stm32f732)]
    fn can_get_sector() {
        const SMALL_SECTOR_SIZE: u32 = 16 * 1024;
        const MEDIUM_SECTOR_SIZE: u32 = 64 * 1024;
        const LARGE_SECTOR_SIZE: u32 = 128 * 1024;

        let assert_sector = |index_in_bank: u8, start: u32, size: u32, address: u32| {
            assert_eq!(
                FlashSector {
                    bank: FlashBank::Bank1,
                    index_in_bank,
                    start,
                    size
                },
                get_sector(address, crate::flash::get_flash_regions())
            )
        };

        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_0000);
        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_3FFF);
        assert_sector(3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_C000);
        assert_sector(3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_FFFF);

        assert_sector(4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_0000);
        assert_sector(4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_FFFF);

        assert_sector(5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0802_0000);
        assert_sector(5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0803_FFFF);
        assert_sector(7, 0x0806_0000, LARGE_SECTOR_SIZE, 0x0806_0000);
        assert_sector(7, 0x0806_0000, LARGE_SECTOR_SIZE, 0x0807_FFFF);
    }

    #[test]
    #[cfg(all(stm32f769, feature = "single-bank"))]
    fn can_get_sector() {
        const SMALL_SECTOR_SIZE: u32 = 32 * 1024;
        const MEDIUM_SECTOR_SIZE: u32 = 128 * 1024;
        const LARGE_SECTOR_SIZE: u32 = 256 * 1024;

        let assert_sector = |index_in_bank: u8, start: u32, size: u32, address: u32| {
            assert_eq!(
                FlashSector {
                    bank: FlashBank::Bank1,
                    index_in_bank,
                    start,
                    size
                },
                get_sector(address, crate::flash::get_flash_regions())
            )
        };

        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_0000);
        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_7FFF);
        assert_sector(3, 0x0801_8000, SMALL_SECTOR_SIZE, 0x0801_8000);
        assert_sector(3, 0x0801_8000, SMALL_SECTOR_SIZE, 0x0801_FFFF);

        assert_sector(4, 0x0802_0000, MEDIUM_SECTOR_SIZE, 0x0802_0000);
        assert_sector(4, 0x0802_0000, MEDIUM_SECTOR_SIZE, 0x0803_FFFF);

        assert_sector(5, 0x0804_0000, LARGE_SECTOR_SIZE, 0x0804_0000);
        assert_sector(5, 0x0804_0000, LARGE_SECTOR_SIZE, 0x0807_FFFF);
        assert_sector(7, 0x080C_0000, LARGE_SECTOR_SIZE, 0x080C_0000);
        assert_sector(7, 0x080C_0000, LARGE_SECTOR_SIZE, 0x080F_FFFF);
    }

    #[test]
    #[cfg(all(stm32f769, feature = "dual-bank"))]
    fn can_get_sector() {
        const SMALL_SECTOR_SIZE: u32 = 16 * 1024;
        const MEDIUM_SECTOR_SIZE: u32 = 64 * 1024;
        const LARGE_SECTOR_SIZE: u32 = 128 * 1024;

        let assert_sector = |index_in_bank: u8, start: u32, size: u32, address: u32, snb: u8, bank: FlashBank| {
            assert_eq!(
                FlashSector {
                    bank: bank,
                    index_in_bank,
                    start,
                    size
                },
                get_sector(address, crate::flash::get_flash_regions())
            );
            assert_eq!(get_sector(address, crate::flash::get_flash_regions()).snb(), snb);
        };

        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_0000, 0x00, FlashBank::Bank1);
        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_3FFF, 0x00, FlashBank::Bank1);
        assert_sector(3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_C000, 0x03, FlashBank::Bank1);
        assert_sector(3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_FFFF, 0x03, FlashBank::Bank1);

        assert_sector(4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_0000, 0x04, FlashBank::Bank1);
        assert_sector(4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_FFFF, 0x04, FlashBank::Bank1);

        assert_sector(5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0802_0000, 0x05, FlashBank::Bank1);
        assert_sector(5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0803_FFFF, 0x05, FlashBank::Bank1);
        assert_sector(10, 0x080C_0000, LARGE_SECTOR_SIZE, 0x080C_0000, 0x0A, FlashBank::Bank1);
        assert_sector(10, 0x080C_0000, LARGE_SECTOR_SIZE, 0x080D_FFFF, 0x0A, FlashBank::Bank1);

        assert_sector(0, 0x0810_0000, SMALL_SECTOR_SIZE, 0x0810_0000, 0x10, FlashBank::Bank2);
        assert_sector(0, 0x0810_0000, SMALL_SECTOR_SIZE, 0x0810_3FFF, 0x10, FlashBank::Bank2);
        assert_sector(3, 0x0810_C000, SMALL_SECTOR_SIZE, 0x0810_C000, 0x13, FlashBank::Bank2);
        assert_sector(3, 0x0810_C000, SMALL_SECTOR_SIZE, 0x0810_FFFF, 0x13, FlashBank::Bank2);

        assert_sector(4, 0x0811_0000, MEDIUM_SECTOR_SIZE, 0x0811_0000, 0x14, FlashBank::Bank2);
        assert_sector(4, 0x0811_0000, MEDIUM_SECTOR_SIZE, 0x0811_FFFF, 0x14, FlashBank::Bank2);

        assert_sector(5, 0x0812_0000, LARGE_SECTOR_SIZE, 0x0812_0000, 0x15, FlashBank::Bank2);
        assert_sector(5, 0x0812_0000, LARGE_SECTOR_SIZE, 0x0813_FFFF, 0x15, FlashBank::Bank2);
        assert_sector(10, 0x081C_0000, LARGE_SECTOR_SIZE, 0x081C_0000, 0x1A, FlashBank::Bank2);
        assert_sector(10, 0x081C_0000, LARGE_SECTOR_SIZE, 0x081D_FFFF, 0x1A, FlashBank::Bank2);
    }
}

#[cfg(all(bank_setup_configurable))]
pub(crate) fn check_bank_setup() {
    if cfg!(feature = "single-bank") && !pac::FLASH.optcr().read().n_dbank() {
        panic!(
            "Embassy is configured as single-bank, but the hardware is running in dual-bank mode. Change the hardware by changing the ndbank value in the user option bytes or configure embassy to use dual-bank config"
        );
    }
    if cfg!(feature = "dual-bank") && pac::FLASH.optcr().read().n_dbank() {
        panic!(
            "Embassy is configured as dual-bank, but the hardware is running in single-bank mode. Change the hardware by changing the ndbank value in the user option bytes or configure embassy to use single-bank config"
        );
    }
}
