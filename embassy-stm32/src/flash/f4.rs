use core::convert::TryInto;
use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use super::{FlashSector, FLASH_BASE, FLASH_SIZE, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

const SMALL_SECTOR_SIZE: u32 = 16 * 1024;
const MEDIUM_SECTOR_SIZE: u32 = 64 * 1024;
const LARGE_SECTOR_SIZE: u32 = 128 * 1024;
const SECOND_BANK_SECTOR_OFFSET: u8 = 12;

#[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479))]
mod alt_regions {
    use embassy_hal_common::PeripheralRef;
    use stm32_metapac::FLASH_SIZE;

    use crate::_generated::flash_regions::{BANK1_REGION1, BANK1_REGION2, BANK1_REGION3};
    use crate::flash::{Bank1Region1, Bank1Region2, Flash, FlashRegion};
    use crate::peripherals::FLASH;

    pub const ALT_BANK1_REGION3: FlashRegion = FlashRegion {
        size: 3 * BANK1_REGION3.erase_size,
        ..BANK1_REGION3
    };
    pub const ALT_BANK2_REGION1: FlashRegion = FlashRegion {
        base: BANK1_REGION1.base + FLASH_SIZE as u32 / 2,
        ..BANK1_REGION1
    };
    pub const ALT_BANK2_REGION2: FlashRegion = FlashRegion {
        base: BANK1_REGION2.base + FLASH_SIZE as u32 / 2,
        ..BANK1_REGION2
    };
    pub const ALT_BANK2_REGION3: FlashRegion = FlashRegion {
        base: BANK1_REGION3.base + FLASH_SIZE as u32 / 2,
        size: 3 * BANK1_REGION3.erase_size,
        ..BANK1_REGION3
    };

    pub type AltBank1Region1 = Bank1Region1;
    pub type AltBank1Region2 = Bank1Region2;
    pub struct AltBank1Region3(&'static FlashRegion);
    pub struct AltBank2Region1(&'static FlashRegion);
    pub struct AltBank2Region2(&'static FlashRegion);
    pub struct AltBank2Region3(&'static FlashRegion);

    pub struct AltFlashLayout<'d> {
        _inner: PeripheralRef<'d, FLASH>,
        pub bank1_region1: AltBank1Region1,
        pub bank1_region2: AltBank1Region2,
        pub bank1_region3: AltBank1Region3,
        pub bank2_region1: AltBank2Region1,
        pub bank2_region2: AltBank2Region2,
        pub bank2_region3: AltBank2Region3,
    }

    impl<'d> Flash<'d> {
        pub fn into_alt_regions(self) -> AltFlashLayout<'d> {
            unsafe { crate::pac::FLASH.optcr().modify(|r| r.set_db1m(true)) };
            AltFlashLayout {
                _inner: self.release(),
                bank1_region1: Bank1Region1(&BANK1_REGION1),
                bank1_region2: Bank1Region2(&BANK1_REGION2),
                bank1_region3: AltBank1Region3(&ALT_BANK1_REGION3),
                bank2_region1: AltBank2Region1(&ALT_BANK2_REGION1),
                bank2_region2: AltBank2Region2(&ALT_BANK2_REGION2),
                bank2_region3: AltBank2Region3(&ALT_BANK2_REGION3),
            }
        }
    }

    impl Drop for AltFlashLayout<'_> {
        fn drop(&mut self) {
            unsafe {
                super::lock();
                crate::pac::FLASH.optcr().modify(|r| r.set_db1m(false))
            };
        }
    }
}

#[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479))]
pub use alt_regions::AltFlashLayout;

fn is_dual_bank() -> bool {
    // let asd: super::Bank1Region1;
    // let sad = &super::BANK_1_REGION_1;

    match FLASH_SIZE / 1024 {
        // 1 MB devices depend on configuration
        1024 => {
            if cfg!(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479)) {
                unsafe { pac::FLASH.optcr().read().db1m() }
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

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    let sector = sector.index;
    let bank = sector / SECOND_BANK_SECTOR_OFFSET as u8;
    let snb = (bank << 4) + (sector % SECOND_BANK_SECTOR_OFFSET as u8);

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
        w.set_eop(true);
    });
}

unsafe fn blocking_wait_ready() -> Result<(), Error> {
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

pub(crate) fn get_sector(address: u32) -> FlashSector {
    get_sector_inner(address, is_dual_bank(), FLASH_SIZE as u32)
}

fn get_sector_inner(address: u32, dual_bank: bool, flash_size: u32) -> FlashSector {
    let offset = address - FLASH_BASE as u32;
    if !dual_bank {
        get_single_bank_sector(offset)
    } else {
        let bank_size = flash_size / 2;
        if offset < bank_size {
            get_single_bank_sector(offset)
        } else {
            let sector = get_single_bank_sector(offset - bank_size);
            FlashSector {
                index: SECOND_BANK_SECTOR_OFFSET + sector.index,
                start: sector.start + bank_size,
                size: sector.size,
            }
        }
    }
}

fn get_single_bank_sector(offset: u32) -> FlashSector {
    // First 4 sectors are 16KB, then one 64KB, and rest are 128KB
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
    fn can_get_sector_single_bank() {
        let assert_sector = |index: u8, start: u32, size: u32, addr: u32| {
            assert_eq!(
                FlashSector { index, start, size },
                get_sector_inner(addr, false, 1024 * 1024)
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
        assert_sector(11, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080E_0000);
        assert_sector(11, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080F_FFFF);
    }

    #[test]
    fn can_get_sector_dual_bank() {
        let assert_sector = |index: u8, start: u32, size: u32, addr: u32| {
            assert_eq!(
                FlashSector { index, start, size },
                get_sector_inner(addr, true, 1024 * 1024)
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

        assert_sector(12, 0x0808_0000, SMALL_SECTOR_SIZE, 0x0808_0000);
        assert_sector(12, 0x0808_0000, SMALL_SECTOR_SIZE, 0x0808_3FFF);
        assert_sector(15, 0x0808_C000, SMALL_SECTOR_SIZE, 0x0808_C000);
        assert_sector(15, 0x0808_C000, SMALL_SECTOR_SIZE, 0x0808_FFFF);

        assert_sector(16, 0x0809_0000, MEDIUM_SECTOR_SIZE, 0x0809_0000);
        assert_sector(16, 0x0809_0000, MEDIUM_SECTOR_SIZE, 0x0809_FFFF);

        assert_sector(17, 0x080A_0000, LARGE_SECTOR_SIZE, 0x080A_0000);
        assert_sector(17, 0x080A_0000, LARGE_SECTOR_SIZE, 0x080B_FFFF);
        assert_sector(19, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080E_0000);
        assert_sector(19, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080F_FFFF);
    }
}
