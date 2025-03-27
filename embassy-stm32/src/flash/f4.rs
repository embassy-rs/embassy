use core::ptr::write_volatile;
use core::sync::atomic::{fence, AtomicBool, Ordering};

use embassy_sync::waitqueue::AtomicWaker;
use pac::flash::regs::Sr;

use super::{FlashBank, FlashRegion, FlashSector, FLASH_REGIONS, WRITE_SIZE};
use crate::_generated::FLASH_SIZE;
use crate::flash::Error;
use crate::pac;
#[allow(missing_docs)] // TODO
#[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479))]
mod alt_regions {
    use core::marker::PhantomData;

    use crate::Peri;
    use crate::_generated::flash_regions::{OTPRegion, BANK1_REGION1, BANK1_REGION2, BANK1_REGION3, OTP_REGION};
    use crate::_generated::FLASH_SIZE;
    use crate::flash::{asynch, Async, Bank1Region1, Bank1Region2, Blocking, Error, Flash, FlashBank, FlashRegion};
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

    pub struct AltBank1Region3<'d, MODE = Async>(pub &'static FlashRegion, Peri<'d, FLASH>, PhantomData<MODE>);
    pub struct AltBank2Region1<'d, MODE = Async>(pub &'static FlashRegion, Peri<'d, FLASH>, PhantomData<MODE>);
    pub struct AltBank2Region2<'d, MODE = Async>(pub &'static FlashRegion, Peri<'d, FLASH>, PhantomData<MODE>);
    pub struct AltBank2Region3<'d, MODE = Async>(pub &'static FlashRegion, Peri<'d, FLASH>, PhantomData<MODE>);

    pub struct AltFlashLayout<'d, MODE = Async> {
        pub bank1_region1: Bank1Region1<'d, MODE>,
        pub bank1_region2: Bank1Region2<'d, MODE>,
        pub bank1_region3: AltBank1Region3<'d, MODE>,
        pub bank2_region1: AltBank2Region1<'d, MODE>,
        pub bank2_region2: AltBank2Region2<'d, MODE>,
        pub bank2_region3: AltBank2Region3<'d, MODE>,
        pub otp_region: OTPRegion<'d, MODE>,
    }

    impl<'d> Flash<'d> {
        pub fn into_alt_regions(self) -> AltFlashLayout<'d, Async> {
            assert!(!super::is_default_layout());

            // SAFETY: We never expose the cloned peripheral references, and their instance is not public.
            // Also, all async flash region operations are protected with a mutex.
            let p = self.inner;
            AltFlashLayout {
                bank1_region1: Bank1Region1(&BANK1_REGION1, unsafe { p.clone_unchecked() }, PhantomData),
                bank1_region2: Bank1Region2(&BANK1_REGION2, unsafe { p.clone_unchecked() }, PhantomData),
                bank1_region3: AltBank1Region3(&ALT_BANK1_REGION3, unsafe { p.clone_unchecked() }, PhantomData),
                bank2_region1: AltBank2Region1(&ALT_BANK2_REGION1, unsafe { p.clone_unchecked() }, PhantomData),
                bank2_region2: AltBank2Region2(&ALT_BANK2_REGION2, unsafe { p.clone_unchecked() }, PhantomData),
                bank2_region3: AltBank2Region3(&ALT_BANK2_REGION3, unsafe { p.clone_unchecked() }, PhantomData),
                otp_region: OTPRegion(&OTP_REGION, unsafe { p.clone_unchecked() }, PhantomData),
            }
        }

        pub fn into_alt_blocking_regions(self) -> AltFlashLayout<'d, Blocking> {
            assert!(!super::is_default_layout());

            // SAFETY: We never expose the cloned peripheral references, and their instance is not public.
            // Also, all blocking flash region operations are protected with a cs.
            let p = self.inner;
            AltFlashLayout {
                bank1_region1: Bank1Region1(&BANK1_REGION1, unsafe { p.clone_unchecked() }, PhantomData),
                bank1_region2: Bank1Region2(&BANK1_REGION2, unsafe { p.clone_unchecked() }, PhantomData),
                bank1_region3: AltBank1Region3(&ALT_BANK1_REGION3, unsafe { p.clone_unchecked() }, PhantomData),
                bank2_region1: AltBank2Region1(&ALT_BANK2_REGION1, unsafe { p.clone_unchecked() }, PhantomData),
                bank2_region2: AltBank2Region2(&ALT_BANK2_REGION2, unsafe { p.clone_unchecked() }, PhantomData),
                bank2_region3: AltBank2Region3(&ALT_BANK2_REGION3, unsafe { p.clone_unchecked() }, PhantomData),
                otp_region: OTPRegion(&OTP_REGION, unsafe { p.clone_unchecked() }, PhantomData),
            }
        }
    }

    macro_rules! foreach_altflash_region {
        ($type_name:ident, $region:ident) => {
            impl<MODE> $type_name<'_, MODE> {
                pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                    crate::flash::common::blocking_read(self.0.base, self.0.size, offset, bytes)
                }
            }

            impl $type_name<'_, Async> {
                pub async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                    self.blocking_read(offset, bytes)
                }

                pub async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                    let _guard = asynch::REGION_ACCESS.lock().await;
                    unsafe { asynch::write_chunked(self.0.base, self.0.size, offset, bytes).await }
                }

                pub async fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                    let _guard = asynch::REGION_ACCESS.lock().await;
                    unsafe { asynch::erase_sectored(self.0.base, from, to).await }
                }
            }

            impl<MODE> embedded_storage::nor_flash::ErrorType for $type_name<'_, MODE> {
                type Error = Error;
            }

            impl<MODE> embedded_storage::nor_flash::ReadNorFlash for $type_name<'_, MODE> {
                const READ_SIZE: usize = crate::flash::READ_SIZE;

                fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                    self.blocking_read(offset, bytes)
                }

                fn capacity(&self) -> usize {
                    self.0.size as usize
                }
            }

            impl embedded_storage_async::nor_flash::ReadNorFlash for $type_name<'_, Async> {
                const READ_SIZE: usize = crate::flash::READ_SIZE;

                async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                    self.read(offset, bytes).await
                }

                fn capacity(&self) -> usize {
                    self.0.size as usize
                }
            }

            impl embedded_storage_async::nor_flash::NorFlash for $type_name<'_, Async> {
                const WRITE_SIZE: usize = $region.write_size as usize;
                const ERASE_SIZE: usize = $region.erase_size as usize;

                async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
                    self.write(offset, bytes).await
                }

                async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
                    self.erase(from, to).await
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

static WAKER: AtomicWaker = AtomicWaker::new();
static DATA_CACHE_WAS_ENABLED: AtomicBool = AtomicBool::new(false);

impl FlashSector {
    const fn snb(&self) -> u8 {
        ((self.bank as u8) << 4) + self.index_in_bank
    }
}

#[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479))]
pub(crate) fn is_default_layout() -> bool {
    !pac::FLASH.optcr().read().db1m()
}

#[cfg(not(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479)))]
pub(crate) const fn is_default_layout() -> bool {
    true
}

#[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479))]
pub fn get_flash_regions() -> &'static [&'static FlashRegion] {
    if is_default_layout() {
        &FLASH_REGIONS
    } else {
        &ALT_FLASH_REGIONS
    }
}

#[cfg(not(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f469, stm32f479)))]
pub const fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
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
        panic!("Read corruption for stm32f42xxI and stm32f43xxI when PA12 is in use for chips below revision 3, see errata 2.2.11");
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
        panic!("Read corruption for stm32f42xxG and stm32f43xxG in dual bank mode when PA12 is in use for chips below revision 3, see errata 2.2.11");
    }
}

#[allow(unused)]
fn pa12_is_output_pull_low() -> bool {
    use pac::gpio::vals;
    use pac::GPIOA;
    const PIN: usize = 12;
    GPIOA.moder().read().moder(PIN) == vals::Moder::OUTPUT
        && GPIOA.pupdr().read().pupdr(PIN) == vals::Pupdr::PULL_DOWN
        && GPIOA.odr().read().odr(PIN) == vals::Odr::LOW
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flash::{get_sector, FlashBank};

    #[test]
    #[cfg(stm32f429)]
    fn can_get_sector() {
        const SMALL_SECTOR_SIZE: u32 = 16 * 1024;
        const MEDIUM_SECTOR_SIZE: u32 = 64 * 1024;
        const LARGE_SECTOR_SIZE: u32 = 128 * 1024;

        let assert_sector = |snb: u8, index_in_bank: u8, start: u32, size: u32, address: u32| {
            let sector = get_sector(address, &FLASH_REGIONS);
            assert_eq!(snb, sector.snb());
            assert_eq!(
                FlashSector {
                    bank: FlashBank::Bank1,
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

        let assert_sector = |snb: u8, bank: FlashBank, index_in_bank: u8, start: u32, size: u32, address: u32| {
            let sector = get_sector(address, &ALT_FLASH_REGIONS);
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
