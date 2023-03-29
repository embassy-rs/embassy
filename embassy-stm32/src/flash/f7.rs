use core::convert::TryInto;
use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use super::{FlashSector, FLASH_BASE, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

const SMALL_SECTOR_SIZE: u32 = 32 * 1024;
const MEDIUM_SECTOR_SIZE: u32 = 128 * 1024;
const LARGE_SECTOR_SIZE: u32 = 256 * 1024;

pub(crate) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    pac::FLASH.keyr().write(|w| w.set_key(0x4567_0123));
    pac::FLASH.keyr().write(|w| w.set_key(0xCDEF_89AB));
}

pub(crate) unsafe fn begin_write() {
    assert_eq!(0, WRITE_SIZE % 4);

    pac::FLASH.cr().write(|w| {
        w.set_pg(true);
        w.set_psize(pac::flash::vals::Psize::PSIZE32);
    });
}

pub(crate) unsafe fn end_write() {
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

pub(crate) unsafe fn blocking_erase(start_address: u32, end_address: u32) -> Result<(), Error> {
    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address);
        erase_sector(sector.index)?;
        address += sector.size;
    }
    Ok(())
}

unsafe fn erase_sector(sector: u8) -> Result<(), Error> {
    pac::FLASH.cr().modify(|w| {
        w.set_ser(true);
        w.set_snb(sector)
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
    pac::FLASH.sr().modify(|w| {
        if w.erserr() {
            w.set_erserr(true);
        }
        if w.pgperr() {
            w.set_pgperr(true);
        }
        if w.pgaerr() {
            w.set_pgaerr(true);
        }
        if w.wrperr() {
            w.set_wrperr(true);
        }
        if w.eop() {
            w.set_eop(true);
        }
    });
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

pub(crate) fn get_sector(address: u32) -> FlashSector {
    // First 4 sectors are 32KB, then one 128KB, and rest are 256KB
    let offset = address - FLASH_BASE as u32;
    match offset / LARGE_SECTOR_SIZE {
        0 => {
            if offset < 4 * SMALL_SECTOR_SIZE {
                let small_sector_index = offset / SMALL_SECTOR_SIZE;
                FlashSector {
                    index: small_sector_index as u8,
                    start: FLASH_BASE as u32 + small_sector_index * SMALL_SECTOR_SIZE,
                    size: SMALL_SECTOR_SIZE,
                }
            } else {
                FlashSector {
                    index: 4,
                    start: FLASH_BASE as u32 + 4 * SMALL_SECTOR_SIZE,
                    size: MEDIUM_SECTOR_SIZE,
                }
            }
        }
        i => {
            let large_sector_index = i - 1;
            FlashSector {
                index: (5 + large_sector_index) as u8,
                start: FLASH_BASE as u32
                    + 4 * SMALL_SECTOR_SIZE
                    + MEDIUM_SECTOR_SIZE
                    + large_sector_index * LARGE_SECTOR_SIZE,
                size: LARGE_SECTOR_SIZE,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_sector() {
        let assert_sector = |index: u8, start: u32, size: u32, addr: u32| {
            assert_eq!(FlashSector { index, start, size }, get_sector(addr))
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
