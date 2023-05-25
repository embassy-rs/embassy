use core::convert::TryInto;
use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use super::{FlashRegion, FlashSector, FLASH_REGIONS, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

#[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479))]
mod alt_regions {
    use embassy_hal_common::PeripheralRef;
    use stm32_metapac::FLASH_SIZE;

    use crate::_generated::flash_regions::{OTPRegion, BANK1_REGION1, BANK1_REGION2, BANK1_REGION3, OTP_REGION};
    use crate::flash::{
        blocking_erase_sectored, blocking_read, blocking_write_chunked, Bank1Region1, Bank1Region2, Error, Flash,
        FlashBank, FlashRegion,
    };
    use crate::peripherals::FLASH;

    pub const ALT_BANK1_REGION3: FlashRegion = FlashRegion {
        size: 3 * BANK1_REGION3.erase_size,
        ..BANK1_REGION3
    };
    pub const ALT_BANK2_REGION1: FlashRegion = FlashRegion {
        bank: FlashBank::Bank2,
        base: BANK1_REGION1.base + FLASH_SIZE as u32 / 2,
        ..BANK1_REGION1
    };
    pub const ALT_BANK2_REGION2: FlashRegion = FlashRegion {
        bank: FlashBank::Bank2,
        base: BANK1_REGION2.base + FLASH_SIZE as u32 / 2,
        ..BANK1_REGION2
    };
    pub const ALT_BANK2_REGION3: FlashRegion = FlashRegion {
        bank: FlashBank::Bank2,
        base: BANK1_REGION3.base + FLASH_SIZE as u32 / 2,
        size: 3 * BANK1_REGION3.erase_size,
        ..BANK1_REGION3
    };

    pub const ALT_FLASH_REGIONS: [&FlashRegion; 6] = [
        &BANK1_REGION1,
        &BANK1_REGION2,
        &ALT_BANK1_REGION3,
        &ALT_BANK2_REGION1,
        &ALT_BANK2_REGION2,
        &ALT_BANK2_REGION3,
    ];

    pub struct AltBank1Region3<'d>(pub &'static FlashRegion, PeripheralRef<'d, FLASH>);
    pub struct AltBank2Region1<'d>(pub &'static FlashRegion, PeripheralRef<'d, FLASH>);
    pub struct AltBank2Region2<'d>(pub &'static FlashRegion, PeripheralRef<'d, FLASH>);
    pub struct AltBank2Region3<'d>(pub &'static FlashRegion, PeripheralRef<'d, FLASH>);

    pub struct AltFlashLayout<'d> {
        pub bank1_region1: Bank1Region1<'d>,
        pub bank1_region2: Bank1Region2<'d>,
        pub bank1_region3: AltBank1Region3<'d>,
        pub bank2_region1: AltBank2Region1<'d>,
        pub bank2_region2: AltBank2Region2<'d>,
        pub bank2_region3: AltBank2Region3<'d>,
        pub otp_region: OTPRegion<'d>,
    }

    impl<'d> Flash<'d> {
        pub fn into_alt_regions(self) -> AltFlashLayout<'d> {
            unsafe { crate::pac::FLASH.optcr().modify(|r| r.set_db1m(true)) };

            // SAFETY: We never expose the cloned peripheral references, and their instance is not public.
            // Also, all blocking flash region operations are protected with a cs.
            let p = self.release();
            AltFlashLayout {
                bank1_region1: Bank1Region1(&BANK1_REGION1, unsafe { p.clone_unchecked() }),
                bank1_region2: Bank1Region2(&BANK1_REGION2, unsafe { p.clone_unchecked() }),
                bank1_region3: AltBank1Region3(&ALT_BANK1_REGION3, unsafe { p.clone_unchecked() }),
                bank2_region1: AltBank2Region1(&ALT_BANK2_REGION1, unsafe { p.clone_unchecked() }),
                bank2_region2: AltBank2Region2(&ALT_BANK2_REGION2, unsafe { p.clone_unchecked() }),
                bank2_region3: AltBank2Region3(&ALT_BANK2_REGION3, unsafe { p.clone_unchecked() }),
                otp_region: OTPRegion(&OTP_REGION, unsafe { p.clone_unchecked() }),
            }
        }
    }

    macro_rules! foreach_altflash_region {
        ($type_name:ident, $region:ident) => {
            impl $type_name<'_> {
                pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                    blocking_read(self.0.base, self.0.size, offset, bytes)
                }

                pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                    unsafe { blocking_write_chunked(self.0.base, self.0.size, offset, bytes) }
                }

                pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                    unsafe { blocking_erase_sectored(self.0.base, from, to) }
                }
            }

            impl embedded_storage::nor_flash::ErrorType for $type_name<'_> {
                type Error = Error;
            }

            impl embedded_storage::nor_flash::ReadNorFlash for $type_name<'_> {
                const READ_SIZE: usize = 1;

                fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                    self.blocking_read(offset, bytes)
                }

                fn capacity(&self) -> usize {
                    self.0.size as usize
                }
            }

            impl embedded_storage::nor_flash::NorFlash for $type_name<'_> {
                const WRITE_SIZE: usize = $region.write_size as usize;
                const ERASE_SIZE: usize = $region.erase_size as usize;

                fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
                    self.blocking_write(offset, bytes)
                }

                fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
                    self.blocking_erase(from, to)
                }
            }
        };
    }

    foreach_altflash_region!(AltBank1Region3, ALT_BANK1_REGION3);
    foreach_altflash_region!(AltBank2Region1, ALT_BANK2_REGION1);
    foreach_altflash_region!(AltBank2Region2, ALT_BANK2_REGION2);
    foreach_altflash_region!(AltBank2Region3, ALT_BANK2_REGION3);
}

#[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479))]
pub use alt_regions::*;

#[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479))]
pub fn set_default_layout() {
    unsafe { crate::pac::FLASH.optcr().modify(|r| r.set_db1m(false)) };
}

#[cfg(not(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479)))]
pub const fn set_default_layout() {}

#[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479))]
pub fn get_flash_regions() -> &'static [&'static FlashRegion] {
    if unsafe { pac::FLASH.optcr().read().db1m() } {
        &ALT_FLASH_REGIONS
    } else {
        &FLASH_REGIONS
    }
}

#[cfg(not(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479)))]
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
    let snb = ((sector.bank as u8) << 4) + sector.index_in_bank;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flash::{get_sector, FlashBank};

    #[test]
    #[cfg(stm32f429)]
    fn can_get_sector_single_bank() {
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
        assert_sector(11, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080E_0000);
        assert_sector(11, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080F_FFFF);

        let assert_sector = |bank: FlashBank, index_in_bank: u8, start: u32, size: u32, address: u32| {
            assert_eq!(
                FlashSector {
                    bank,
                    index_in_bank,
                    start,
                    size
                },
                get_sector(address, &ALT_FLASH_REGIONS)
            )
        };

        assert_sector(FlashBank::Bank1, 0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_0000);
        assert_sector(FlashBank::Bank1, 0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_3FFF);
        assert_sector(FlashBank::Bank1, 3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_C000);
        assert_sector(FlashBank::Bank1, 3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_FFFF);

        assert_sector(FlashBank::Bank1, 4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_0000);
        assert_sector(FlashBank::Bank1, 4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_FFFF);

        assert_sector(FlashBank::Bank1, 5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0802_0000);
        assert_sector(FlashBank::Bank1, 5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0803_FFFF);
        assert_sector(FlashBank::Bank1, 7, 0x0806_0000, LARGE_SECTOR_SIZE, 0x0806_0000);
        assert_sector(FlashBank::Bank1, 7, 0x0806_0000, LARGE_SECTOR_SIZE, 0x0807_FFFF);

        assert_sector(FlashBank::Bank2, 0, 0x0808_0000, SMALL_SECTOR_SIZE, 0x0808_0000);
        assert_sector(FlashBank::Bank2, 0, 0x0808_0000, SMALL_SECTOR_SIZE, 0x0808_3FFF);
        assert_sector(FlashBank::Bank2, 3, 0x0808_C000, SMALL_SECTOR_SIZE, 0x0808_C000);
        assert_sector(FlashBank::Bank2, 3, 0x0808_C000, SMALL_SECTOR_SIZE, 0x0808_FFFF);

        assert_sector(FlashBank::Bank2, 4, 0x0809_0000, MEDIUM_SECTOR_SIZE, 0x0809_0000);
        assert_sector(FlashBank::Bank2, 4, 0x0809_0000, MEDIUM_SECTOR_SIZE, 0x0809_FFFF);

        assert_sector(FlashBank::Bank2, 5, 0x080A_0000, LARGE_SECTOR_SIZE, 0x080A_0000);
        assert_sector(FlashBank::Bank2, 5, 0x080A_0000, LARGE_SECTOR_SIZE, 0x080B_FFFF);
        assert_sector(FlashBank::Bank2, 7, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080E_0000);
        assert_sector(FlashBank::Bank2, 7, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080F_FFFF);
    }
}
