use core::convert::TryInto;
use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use super::{FlashRegion, FlashSector, FLASH_REGIONS, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

pub const fn is_default_layout() -> bool {
    true
}

pub const fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
}

pub(crate) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    pac::FLASH.keyr().write(|w| w.set_key(0x4567_0123));
    pac::FLASH.keyr().write(|w| w.set_key(0xCDEF_89AB));
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
        write_volatile(address as *mut u32, u32::from_le_bytes(val.try_into().unwrap()));
        address += val.len() as u32;

        // prevents parallelism errors
        fence(Ordering::SeqCst);
    }

    blocking_wait_ready()
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(sector.index_in_bank)
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
    // This clears all "write 0 to clear" bits.
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
    use crate::flash::{get_sector, FlashBank};

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
                get_sector(address, &FLASH_REGIONS)
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
    #[cfg(stm32f769)]
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
                get_sector(address, &FLASH_REGIONS)
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
}
