#![no_std]
#![no_main]

// Erases, programs and verifies the last sector of each flash bank, through the blocking,
// async and region APIs, and checks error reporting and recovery.

#[path = "../common.rs"]
mod common;

use common::*;
use defmt::{assert, assert_eq};
use embassy_executor::Spawner;
use embassy_stm32::flash::{
    self, Config, Error, FLASH_BASE, FLASH_SIZE, Flash, FlashBank, WRITE_SIZE, get_flash_regions,
};
use embassy_stm32::mode::Mode;
use embassy_stm32::{Peri, bind_interrupts, peripherals};
use panic_probe as _;

teleprobe_meta::timeout!(120);

#[cfg(not(feature = "stm32u083rc"))]
bind_interrupts!(struct Irqs {
    FLASH => flash::InterruptHandler;
});
#[cfg(feature = "stm32u083rc")]
bind_interrupts!(struct Irqs {
    FLASH_ECC => flash::InterruptHandler;
});

/// Chunk size used for programming and verifying.
const CHUNK: usize = 256;

/// The hardware flags programming a location that is not erased.
const DETECTS_NOT_ERASED: bool = cfg!(any(
    // v1
    feature = "stm32f091rc",
    feature = "stm32f100rd",
    feature = "stm32f103c8",
    feature = "stm32f107vc",
    feature = "stm32f303ze",
    // v3: L0 only, L1 has no such check
    feature = "stm32l073rz",
    // v4
    feature = "stm32c031c6",
    feature = "stm32c071rb",
    feature = "stm32g071rb",
    feature = "stm32g491re",
    feature = "stm32l496zg",
    feature = "stm32l4a6zg",
    feature = "stm32l4r5zi",
    feature = "stm32l552ze",
    feature = "stm32u083rc",
    feature = "stm32u585ai",
    feature = "stm32u5a5zj",
    feature = "stm32wb55rg",
    feature = "stm32wba52cg",
    feature = "stm32wba65ri",
    feature = "stm32wl55jc",
));

#[derive(Clone, Copy, defmt::Format)]
struct Sector {
    /// Offset from FLASH_BASE.
    offset: u32,
    size: u32,
    erase_value: u8,
}

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let mut p = init();

    info!(
        "FLASH_BASE=0x{:08x} FLASH_SIZE=0x{:x} WRITE_SIZE={}",
        FLASH_BASE, FLASH_SIZE, WRITE_SIZE
    );

    let sectors = test_sectors();
    for s in &sectors {
        info!("test sector: {}", s);
    }

    for s in sectors.iter().flatten() {
        info!("blocking, sector at 0x{:08x}", FLASH_BASE as u32 + s.offset);
        let mut f = Flash::new_blocking(p.FLASH.reborrow(), Config::default());
        test_blocking(&mut f, s);
        test_errors(&mut f, s);
        test_error_recovery(&mut f, s);
    }

    for s in sectors.iter().flatten() {
        info!("async, sector at 0x{:08x}", FLASH_BASE as u32 + s.offset);
        let mut f = Flash::new(p.FLASH.reborrow(), Irqs, Config::default());
        test_async(&mut f, s).await;
    }

    test_regions(p.FLASH.reborrow()).await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}

/// The last sector of each bank, skipping the ones holding the test binary.
fn test_sectors() -> [Option<Sector>; 2] {
    let mut res = [None, None];
    for (i, bank) in [FlashBank::Bank1, FlashBank::Bank2].into_iter().enumerate() {
        let Some(region) = get_flash_regions().iter().filter(|r| r.bank == bank).last() else {
            continue;
        };
        let end = region.end();

        // On WB/WL, pages from SFSA up are reserved for the CPU2 firmware.
        #[cfg(any(feature = "stm32wb55rg", feature = "stm32wl55jc"))]
        let end = {
            let sfsa = embassy_stm32::pac::FLASH.sfr().read().sfsa() as u32;
            let limit = FLASH_BASE as u32 + sfsa * region.erase_size;
            info!("SFSA = {} (0x{:08x})", sfsa, limit);
            if sfsa != 0 { end.min(limit) } else { end }
        };

        let start = end - region.erase_size;
        assert!(start >= region.base());
        assert!(start + region.erase_size <= image_start() || start >= image_end());
        res[i] = Some(Sector {
            offset: start - FLASH_BASE as u32,
            size: region.erase_size,
            erase_value: region.erase_value,
        });
    }
    res
}

/// Range of flash occupied by the test binary (empty when running from RAM).
fn image_start() -> u32 {
    if image_end() > FLASH_BASE as u32 {
        FLASH_BASE as u32
    } else {
        u32::MAX
    }
}

fn image_end() -> u32 {
    unsafe extern "C" {
        static __sidata: u8;
        static __sdata: u8;
        static __edata: u8;
    }
    let sidata = &raw const __sidata as u32;
    let data_len = &raw const __edata as u32 - &raw const __sdata as u32;
    let end = sidata + data_len;
    if end > FLASH_BASE as u32 && end <= FLASH_BASE as u32 + FLASH_SIZE as u32 {
        end
    } else {
        0
    }
}

fn pattern(offset: u32, seed: u8) -> u8 {
    let x = offset.wrapping_mul(0x9E37_79B1) >> 13;
    let b = (x as u8) ^ seed;
    // Never all ones or all zeros, so the pattern differs from the erased state.
    if b == 0 || b == 0xFF { 0x5A ^ seed } else { b }
}

fn fill(buf: &mut [u8], offset: u32, seed: u8) {
    for (i, b) in buf.iter_mut().enumerate() {
        *b = pattern(offset + i as u32, seed);
    }
}

fn verify<M: Mode>(f: &mut Flash<'_, M>, s: &Sector, seed: Option<u8>) {
    let chunk = CHUNK.min(s.size as usize);
    let mut buf = [0u8; CHUNK];
    let mut expected = [0u8; CHUNK];
    let (buf, expected) = (&mut buf[..chunk], &mut expected[..chunk]);
    let mut offset = s.offset;
    while offset < s.offset + s.size {
        unwrap!(f.blocking_read(offset, buf));
        match seed {
            Some(seed) => fill(expected, offset, seed),
            None => expected.fill(s.erase_value),
        }
        if buf != expected {
            error!(
                "mismatch at 0x{:08x}: read {:02x}, expected {:02x}",
                FLASH_BASE as u32 + offset,
                buf[..16],
                expected[..16]
            );
            defmt::panic!("verify failed");
        }
        offset += chunk as u32;
    }
}

fn test_blocking(f: &mut Flash<'_, embassy_stm32::mode::Blocking>, s: &Sector) {
    unwrap!(f.blocking_erase(s.offset, s.offset + s.size));
    verify(f, s, None);

    let mut buf = [0u8; CHUNK];
    let buf = &mut buf[..CHUNK.min(s.size as usize)];
    let mut offset = s.offset;
    while offset < s.offset + s.size {
        fill(buf, offset, 0x11);
        unwrap!(f.blocking_write(offset, buf));
        offset += buf.len() as u32;
    }
    verify(f, s, Some(0x11));

    unwrap!(f.blocking_erase(s.offset, s.offset + s.size));
    verify(f, s, None);
}

fn test_errors(f: &mut Flash<'_, embassy_stm32::mode::Blocking>, s: &Sector) {
    let data = [0x42u8; WRITE_SIZE];

    // Alignment and bounds are checked before touching the hardware.
    assert_eq!(f.blocking_write(s.offset + 1, &data), Err(Error::Unaligned));
    assert_eq!(
        f.blocking_write(s.offset, &data[..WRITE_SIZE - 1]),
        Err(Error::Unaligned)
    );
    assert_eq!(f.blocking_erase(s.offset + 1, s.offset + s.size), Err(Error::Unaligned));
    assert_eq!(f.blocking_erase(s.offset, s.offset + s.size - 1), Err(Error::Unaligned));
    assert_eq!(f.blocking_write(FLASH_SIZE as u32, &data), Err(Error::OutOfBounds));

    // On WB the test sector is the last one below the CPU2 secure area, which CPU1 must not
    // touch: the driver rejects it instead of letting the hardware raise a bus fault.
    #[cfg(feature = "stm32wb55rg")]
    assert_eq!(f.blocking_erase(s.offset, s.offset + 2 * s.size), Err(Error::Protected));

    // Erase, then make the hardware report an error by programming a location twice. The
    // next operations must work normally.
    unwrap!(f.blocking_erase(s.offset, s.offset + s.size));
    if DETECTS_NOT_ERASED {
        let mut a = [0u8; WRITE_SIZE];
        let mut b = [0u8; WRITE_SIZE];
        fill(&mut a, s.offset, 0x22);
        fill(&mut b, s.offset, 0x33);
        unwrap!(f.blocking_write(s.offset, &a));
        assert_eq!(f.blocking_write(s.offset, &b), Err(Error::NotErased));
        info!("got NotErased as expected");

        let mut c = [0u8; WRITE_SIZE];
        let off = s.offset + WRITE_SIZE as u32;
        fill(&mut c, off, 0x44);
        unwrap!(f.blocking_write(off, &c));
        let mut r = [0u8; WRITE_SIZE];
        unwrap!(f.blocking_read(off, &mut r));
        assert_eq!(r, c);
    }
    unwrap!(f.blocking_erase(s.offset, s.offset + s.size));
    verify(f, s, None);
}

async fn test_async(f: &mut Flash<'_, embassy_stm32::mode::Async>, s: &Sector) {
    unwrap!(f.erase(s.offset, s.offset + s.size).await);
    verify(f, s, None);

    let mut buf = [0u8; CHUNK];
    let buf = &mut buf[..CHUNK.min(s.size as usize)];
    let mut offset = s.offset;
    while offset < s.offset + s.size {
        fill(buf, offset, 0x55);
        unwrap!(f.write(offset, buf).await);
        offset += buf.len() as u32;
    }
    verify(f, s, Some(0x55));

    if DETECTS_NOT_ERASED {
        let mut b = [0u8; WRITE_SIZE];
        fill(&mut b, s.offset, 0x66);
        assert_eq!(f.write(s.offset, &b).await, Err(Error::NotErased));
    }

    unwrap!(f.erase(s.offset, s.offset + s.size).await);
    verify(f, s, None);
}

/// Region drivers, on the chips whose bank 1 is a single region.
#[allow(unused_variables)]
async fn test_regions(p: Peri<'_, peripherals::FLASH>) {
    #[cfg(not(any(
        feature = "stm32f207zg",
        feature = "stm32f429zi",
        feature = "stm32f446re",
        feature = "stm32f767zi"
    )))]
    {
        let Some(s) = test_sectors()[0] else { return };
        let region_offset = s.offset - (embassy_stm32::flash::BANK1_REGION.base() - FLASH_BASE as u32);
        info!("regions, offset 0x{:x} in bank 1 region", region_offset);

        let mut layout = Flash::new(p, Irqs, Config::default()).into_regions();
        let region = &mut layout.bank1_region;
        let mut buf = [0u8; CHUNK];
        let mut read = [0u8; CHUNK];
        let n = CHUNK.min(s.size as usize);
        let (buf, read) = (&mut buf[..n], &mut read[..n]);

        unwrap!(region.erase(region_offset, region_offset + s.size).await);
        fill(buf, s.offset, 0x77);
        unwrap!(region.write(region_offset, buf).await);
        unwrap!(region.read(region_offset, read).await);
        assert_eq!(read, buf);
        assert_eq!(
            region.erase(region_offset, region.0.size + s.size).await,
            Err(Error::OutOfBounds)
        );
        unwrap!(region.erase(region_offset, region_offset + s.size).await);
        unwrap!(region.read(region_offset, read).await);
        assert!(read.iter().all(|&b| b == s.erase_value));
    }
}

/// Leaves an error flag set in the flash controller behind the driver's back, then checks the
/// next operation succeeds.
fn test_error_recovery(f: &mut Flash<'_, embassy_stm32::mode::Blocking>, s: &Sector) {
    #[cfg(any(
        feature = "stm32h753zi",
        feature = "stm32h755zi",
        feature = "stm32h7a3zi",
        feature = "stm32h563zi",
        feature = "stm32h503rb",
        feature = "stm32h7s3l8"
    ))]
    {
        use core::ptr::write_volatile;

        use embassy_stm32::pac;

        let addr = FLASH_BASE as u32 + s.offset;
        unwrap!(f.blocking_erase(s.offset, s.offset + s.size));

        // H7: a write without PG set raises PGSERR, which blocks any further programming
        // until it is cleared.
        #[cfg(any(feature = "stm32h753zi", feature = "stm32h755zi", feature = "stm32h7a3zi"))]
        let error = {
            let r = pac::FLASH.bank(if s.offset >= FLASH_SIZE as u32 / 2 { 1 } else { 0 });
            if r.cr().read().lock() {
                r.keyr().write_value(0x4567_0123);
                r.keyr().write_value(0xCDEF_89AB);
            }
            unsafe { write_volatile(addr as *mut u32, 0x1234_5678) };
            cortex_m::asm::dsb();
            let error = r.sr().read().pgserr();
            r.cr().modify(|w| w.set_lock(true));
            error
        };
        // H5: writing the same write buffer location twice raises STRBERR.
        #[cfg(any(feature = "stm32h563zi", feature = "stm32h503rb"))]
        let error = {
            let f = pac::FLASH;
            if f.nscr().read().lock() {
                f.nskeyr().write_value(0x4567_0123);
                f.nskeyr().write_value(0xCDEF_89AB);
            }
            f.nscr().modify(|w| w.set_pg(true));
            unsafe {
                write_volatile(addr as *mut u32, 0x1234_5678);
                cortex_m::asm::dsb();
                write_volatile(addr as *mut u32, 0x1234_5678);
            }
            cortex_m::asm::dsb();
            let strberr = f.nssr().read().strberr();
            f.nscr().modify(|w| w.set_pg(false));
            f.nscr().modify(|w| w.set_lock(true));
            strberr
        };
        #[cfg(feature = "stm32h7s3l8")]
        let error = {
            let f = pac::FLASH;
            if f.cr().read().lock() {
                f.keyr().write_value(0x4567_0123);
                f.keyr().write_value(0xCDEF_89AB);
            }
            unsafe { write_volatile(addr as *mut u32, 0x1234_5678) };
            cortex_m::asm::dsb();
            let error = f.isr().read().pgserr();
            f.cr().modify(|w| w.set_lock(true));
            error
        };
        info!("provoked error: {}", error);
        assert!(error);

        // The partially filled flash word may or may not have been programmed, use the next one.
        let off = s.offset + 2 * WRITE_SIZE as u32;
        let mut c = [0u8; WRITE_SIZE];
        fill(&mut c, off, 0x88);
        unwrap!(f.blocking_write(off, &c));
        let mut r = [0u8; WRITE_SIZE];
        unwrap!(f.blocking_read(off, &mut r));
        assert_eq!(r, c);
        unwrap!(f.blocking_erase(s.offset, s.offset + s.size));
        verify(f, s, None);
    }
    let _ = (f, s);
}
