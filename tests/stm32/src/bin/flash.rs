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

    // First, because on some chips it resets the chip several times.
    #[cfg(any(ecc_double_write, ecc_reset))]
    ecc::test(p.FLASH.reborrow()).await;

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

/// Uncorrectable ECC errors: every read API must return `Error::Ecc` instead of crashing with a
/// bus fault or NMI, and erasing the sector must clear them.
///
/// The errors are planted as described in AN5342. On the chips that don't check for
/// overwrites, by programming flash words twice. On the others, by resetting the chip in the
/// middle of an erase.
#[cfg(any(ecc_double_write, ecc_reset))]
mod ecc {
    use embassy_futures::block_on;
    use embassy_stm32::flash::BANK1_REGION;
    use embassy_stm32::mode::Blocking;
    use embedded_storage::nor_flash::ReadNorFlash;
    use embedded_storage_async::nor_flash::ReadNorFlash as AsyncReadNorFlash;

    use super::*;

    /// A read through one of the driver APIs. `offset` is relative to `FLASH_BASE`.
    type Read<'a> = &'a mut dyn FnMut(u32, &mut [u8]) -> Result<(), Error>;

    pub async fn test(mut p: Peri<'_, peripherals::FLASH>) {
        let s = unwrap!(test_sectors()[0]);
        let planted = plant(&mut Flash::new_blocking(p.reborrow(), Config::default()), &s).await;
        let region_offset = BANK1_REGION.base() - FLASH_BASE as u32;

        info!("ecc: Flash, blocking");
        {
            let mut f = Flash::new_blocking(p.reborrow(), Config::default());
            check(&mut |o, b| f.blocking_read(o, b), &planted);
            check(&mut |o, b| ReadNorFlash::read(&mut f, o, b), &planted);
        }
        info!("ecc: Flash, async");
        {
            let mut f = Flash::new(p.reborrow(), Irqs, Config::default());
            check(&mut |o, b| block_on(AsyncReadNorFlash::read(&mut f, o, b)), &planted);
        }
        info!("ecc: regions, blocking");
        {
            let mut layout = Flash::new_blocking(p.reborrow(), Config::default()).into_blocking_regions();
            let r = &mut layout.bank1_region;
            check(&mut |o, b| r.blocking_read(o - region_offset, b), &planted);
            check(&mut |o, b| ReadNorFlash::read(r, o - region_offset, b), &planted);
        }
        info!("ecc: regions, async");
        {
            let mut layout = Flash::new(p.reborrow(), Irqs, Config::default()).into_regions();
            let r = &mut layout.bank1_region;
            check(&mut |o, b| block_on(r.read(o - region_offset, b)), &planted);
            check(
                &mut |o, b| block_on(AsyncReadNorFlash::read(r, o - region_offset, b)),
                &planted,
            );
        }

        // The Cortex-M7 data cache must not hide the errors.
        #[cfg(any(
            feature = "stm32h753zi",
            feature = "stm32h755zi",
            feature = "stm32h7a3zi",
            feature = "stm32h7s3l8"
        ))]
        {
            info!("ecc: Flash, blocking, D-cache enabled");
            let mut cp = unsafe { cortex_m::Peripherals::steal() };
            cp.SCB.enable_dcache(&mut cp.CPUID);
            let mut f = Flash::new_blocking(p.reborrow(), Config::default());
            check(&mut |o, b| f.blocking_read(o, b), &planted);
            cp.SCB.disable_dcache(&mut cp.CPUID);
        }

        let mut f = Flash::new_blocking(p, Config::default());
        unwrap!(f.blocking_erase(s.offset, s.offset + s.size));
        verify(&mut f, &s, None);
        info!("ecc: OK");
    }

    // ===== Planting by programming flash words twice

    /// The unit protected by ECC, and the AN5342 patterns that give a double error (DED) and a
    /// single, corrected error (SEC) when programmed one after another into the same word.
    #[cfg(any(feature = "stm32h753zi", feature = "stm32h755zi"))]
    mod pattern {
        pub const FLASH_WORD: usize = 32;
        const fn word(first: u8) -> [u8; FLASH_WORD] {
            let mut w = [0xCC; FLASH_WORD];
            w[0] = first;
            w
        }
        pub const DED: &[[u8; FLASH_WORD]] = &[word(0xCB), word(0xCC)];
        // AN5342's SEC pattern (0xC1, then 0xC0) gives a double error on H74x/H75x.
        pub const SEC: &[[u8; FLASH_WORD]] = &[[0xCC; FLASH_WORD], {
            let mut w = [0xCC; FLASH_WORD];
            w[1] &= !(1 << 6);
            w
        }];
        /// What the SEC word reads back as.
        pub const SEC_DATA: [u8; FLASH_WORD] = [0xCC; FLASH_WORD];
    }
    #[cfg(all(ecc_double_write, not(any(feature = "stm32h753zi", feature = "stm32h755zi"))))]
    mod pattern {
        pub const FLASH_WORD: usize = 16;
        const fn word(first: u8) -> [u8; FLASH_WORD] {
            let mut w = [0; FLASH_WORD];
            w[0] = first;
            w
        }
        pub const DED: &[[u8; FLASH_WORD]] = &[word(0x0A), word(0x0C), word(0x04)];
        pub const SEC: &[[u8; FLASH_WORD]] = &[word(0x0A), word(0x0C)];
        pub const SEC_DATA: [u8; FLASH_WORD] = word(0x0A);
    }
    #[cfg(ecc_double_write)]
    use pattern::*;

    #[cfg(ecc_double_write)]
    pub struct Planted {
        /// Word with a double error.
        ded: u32,
        /// Word with a single error.
        sec: u32,
        /// Word with good data.
        good: u32,
        good_data: [u8; FLASH_WORD],
    }

    /// Programs `word` into the flash word at `offset`. The rest of the program unit is zeroed,
    /// which keeps its ECC consistent when programmed again.
    #[cfg(ecc_double_write)]
    fn program(f: &mut Flash<'_, Blocking>, offset: u32, word: &[u8; FLASH_WORD]) {
        let mut buf = [0u8; WRITE_SIZE];
        buf[..FLASH_WORD].copy_from_slice(word);
        unwrap!(f.blocking_write(offset, &buf));
    }

    #[cfg(ecc_double_write)]
    async fn plant(f: &mut Flash<'_, Blocking>, s: &Sector) -> Planted {
        unwrap!(f.blocking_erase(s.offset, s.offset + s.size));
        let p = Planted {
            ded: s.offset + 4 * WRITE_SIZE as u32,
            sec: s.offset + 8 * WRITE_SIZE as u32,
            good: s.offset + 12 * WRITE_SIZE as u32,
            good_data: [0x5A; FLASH_WORD],
        };
        for w in DED {
            program(f, p.ded, w);
        }
        for w in SEC {
            program(f, p.sec, w);
        }
        program(f, p.good, &p.good_data);
        p
    }

    #[cfg(ecc_double_write)]
    fn check(read: Read, p: &Planted) {
        let mut buf = [0u8; FLASH_WORD];

        // The double error is reported on every read of the word, also partial ones.
        for _ in 0..3 {
            defmt::assert_eq!(read(p.ded, &mut buf), Err(Error::Ecc));
        }
        defmt::assert_eq!(read(p.ded + 5, &mut buf[..3]), Err(Error::Ecc));

        // The single error is corrected.
        unwrap!(read(p.sec, &mut buf));
        defmt::assert_eq!(buf, SEC_DATA);
        unwrap!(read(p.good, &mut buf));
        defmt::assert_eq!(buf, p.good_data);

        // A long read fails if it covers the bad word, and works otherwise. It starts at the
        // next 32-byte line: with the Cortex-M7 data cache enabled, a read fills the whole line.
        let mut long = [0u8; 256];
        defmt::assert_eq!(read(p.ded - 64, &mut long), Err(Error::Ecc));
        unwrap!(read(p.ded + 32, &mut long));
        let at = |o: u32| (o - p.ded - 32) as usize;
        defmt::assert_eq!(long[at(p.sec)..][..FLASH_WORD], SEC_DATA);
        defmt::assert_eq!(long[at(p.good)..][..FLASH_WORD], p.good_data);

        // The error doesn't stick.
        unwrap!(read(p.good, &mut buf));
        defmt::assert_eq!(buf, p.good_data);
    }

    // ===== Planting by resetting in the middle of an erase

    /// The sector is zeroed, then erased with a reset `iters` delay loop iterations after the
    /// erase starts. Too early, the sector still reads as zeros; too late, as erased. In between,
    /// partially erased bits give ECC errors. The delay is found by bisection, one reset per
    /// step.
    #[cfg(ecc_reset)]
    #[derive(Clone, Copy)]
    #[repr(C)]
    struct State {
        magic: u32,
        /// Iterations known to leave the sector zeroed.
        lo: u32,
        /// Iterations known to leave the sector erased.
        hi: u32,
        /// Iterations of the current attempt.
        iters: u32,
        resets: u32,
    }

    #[cfg(ecc_reset)]
    const MAGIC: u32 = 0xECC0_5EED;

    /// Kept across resets.
    #[cfg(ecc_reset)]
    #[unsafe(link_section = ".uninit.flash_ecc")]
    static mut STATE: core::mem::MaybeUninit<State> = core::mem::MaybeUninit::uninit();

    #[cfg(ecc_reset)]
    pub struct Planted {
        offset: u32,
        size: u32,
    }

    /// Returns once the sector has ECC errors. Until then, every call ends in a reset.
    #[cfg(ecc_reset)]
    async fn plant(f: &mut Flash<'_, Blocking>, s: &Sector) -> Planted {
        let state = unsafe {
            let p = (&raw mut STATE).cast::<State>();
            let state = p.read_volatile();
            p.write_volatile(State { magic: 0, ..state });
            state
        };

        let (lo, hi, resets) = if state.magic == MAGIC {
            let (zeroed, erased, errors) = scan(f, s);
            // Partially erased bits read either way, so ask for more than a few bad words to
            // make sure every read API sees some.
            if errors >= 16 {
                // Give teleprobe time to re-attach to RTT after the reset.
                embassy_time::Timer::after_millis(300).await;
                info!(
                    "ecc: planted after {} resets, {} iterations: {} words zeroed, {} erased, {} with ECC errors",
                    state.resets, state.iters, zeroed, erased, errors
                );
                return Planted {
                    offset: s.offset,
                    size: s.size,
                };
            }
            if zeroed > erased {
                (state.iters, state.hi, state.resets)
            } else {
                (state.lo, state.iters, state.resets)
            }
        } else {
            if image_end() == 0 {
                install_resume_stub(f);
            }
            // A reset after the erase has completed leaves the sector erased.
            zero(f, s);
            let start = embassy_time::Instant::now();
            unwrap!(f.blocking_erase(s.offset, s.offset + s.size));
            let erase = start.elapsed();
            const N: u32 = 1 << 16;
            let start = embassy_time::Instant::now();
            unsafe { spin(N) };
            let spin = start.elapsed();
            let hi = (erase.as_micros() * N as u64 / spin.as_micros()) as u32;
            info!("ecc: erase takes {} us, {} iterations", erase.as_micros(), hi);
            (0, hi, 0)
        };

        if hi - lo <= 1 || resets >= 40 {
            defmt::panic!("ecc: no ECC errors found, lo={} hi={} resets={}", lo, hi, resets);
        }
        let iters = lo + (hi - lo) / 2;
        zero(f, s);
        unsafe {
            (&raw mut STATE).cast::<State>().write_volatile(State {
                magic: MAGIC,
                lo,
                hi,
                iters,
                resets: resets + 1,
            });
            erase_and_reset(s, iters)
        }
    }

    /// When the test runs from RAM, the chip boots from flash after the reset. Flash page 0
    /// gets a vector table whose reset handler starts the RAM image again, like teleprobe did:
    /// with the RAM vector table in VTOR.
    #[cfg(ecc_reset)]
    fn install_resume_stub(f: &mut Flash<'_, Blocking>) {
        let vtor = unsafe { (*cortex_m::peripheral::SCB::PTR).vtor.read() };
        let sp = unsafe { (vtor as *const u32).read_volatile() };
        let words: [u32; 8] = [
            sp,
            FLASH_BASE as u32 + 8 + 1,
            // ldr r0, [pc, #12]; ldr r1, [pc, #16]
            0x4904_4803,
            // str r1, [r0]; ldr r2, [r1, #4]
            0x684a_6001,
            // bx r2; nop
            0xbf00_4710,
            0,
            // VTOR
            0xE000_ED08,
            vtor,
        ];
        let mut stub = [0u8; 32];
        for (b, w) in stub.chunks_exact_mut(4).zip(words) {
            b.copy_from_slice(&w.to_le_bytes());
        }
        let mut current = [0u8; 32];
        unwrap!(f.blocking_read(0, &mut current));
        if current != stub {
            let page = get_flash_regions()[0].erase_size;
            unwrap!(f.blocking_erase(0, page));
            unwrap!(f.blocking_write(0, &stub));
        }
    }

    #[cfg(ecc_reset)]
    fn zero(f: &mut Flash<'_, Blocking>, s: &Sector) {
        unwrap!(f.blocking_erase(s.offset, s.offset + s.size));
        let zeros = [0u8; 256];
        let mut offset = s.offset;
        while offset < s.offset + s.size {
            unwrap!(f.blocking_write(offset, &zeros));
            offset += zeros.len() as u32;
        }
    }

    /// Counts the words of the sector that read as zeros, as erased, and with ECC errors.
    #[cfg(ecc_reset)]
    fn scan(f: &mut Flash<'_, Blocking>, s: &Sector) -> (u32, u32, u32) {
        let (mut zeroed, mut erased, mut errors) = (0, 0, 0);
        let mut buf = [0u8; WRITE_SIZE];
        for offset in (s.offset..s.offset + s.size).step_by(WRITE_SIZE) {
            match f.blocking_read(offset, &mut buf) {
                Err(Error::Ecc) => errors += 1,
                Err(e) => defmt::panic!("read error {}", e),
                Ok(()) if buf.iter().all(|&b| b == 0) => zeroed += 1,
                Ok(()) if buf.iter().all(|&b| b == 0xFF) => erased += 1,
                Ok(()) => {}
            }
        }
        (zeroed, erased, errors)
    }

    #[cfg(any(
        feature = "stm32l552ze",
        feature = "stm32u585ai",
        feature = "stm32u5a5zj",
        feature = "stm32wba52cg",
        feature = "stm32wba65ri"
    ))]
    macro_rules! reg {
        (cr) => {
            embassy_stm32::pac::FLASH.nscr()
        };
        (sr) => {
            embassy_stm32::pac::FLASH.nssr()
        };
        (keyr) => {
            embassy_stm32::pac::FLASH.nskeyr()
        };
    }
    #[cfg(all(
        ecc_reset,
        not(any(
            feature = "stm32l552ze",
            feature = "stm32u585ai",
            feature = "stm32u5a5zj",
            feature = "stm32wba52cg",
            feature = "stm32wba65ri"
        ))
    ))]
    macro_rules! reg {
        (cr) => {
            embassy_stm32::pac::FLASH.cr()
        };
        (sr) => {
            embassy_stm32::pac::FLASH.sr()
        };
        (keyr) => {
            embassy_stm32::pac::FLASH.keyr()
        };
    }

    /// Starts erasing the sector, and resets the chip `iters` delay loop iterations later.
    #[cfg(ecc_reset)]
    unsafe fn erase_and_reset(s: &Sector, iters: u32) -> ! {
        let page = (FLASH_BASE as u32 + s.offset - FlashBank::Bank1.base()) / s.size;
        cortex_m::interrupt::disable();
        unsafe {
            // Make sure the core doesn't halt on the reset.
            let demcr = 0xE000_EDFC as *mut u32;
            demcr.write_volatile(demcr.read_volatile() & !1);

            if reg!(cr).read().lock() {
                reg!(keyr).write_value(0x4567_0123);
                reg!(keyr).write_value(0xCDEF_89AB);
            }
            let sr = reg!(sr).read();
            reg!(sr).write_value(sr);
            reg!(cr).write(|w| {
                w.set_per(true);
                w.set_pnb(page as u8);
            });
            let mut cr = reg!(cr).read();
            cr.set_strt(true);
            ram_erase_and_reset(reg!(cr).as_ptr() as *mut u32, cr.0, iters)
        }
    }

    /// Writes `cr` (which starts the erase), spins `iters` times and resets. From RAM, because
    /// the CPU can't fetch from a flash bank that is being erased.
    #[cfg(ecc_reset)]
    #[unsafe(link_section = ".data.flash_ecc")]
    #[inline(never)]
    unsafe extern "C" fn ram_erase_and_reset(cr_reg: *mut u32, cr: u32, iters: u32) -> ! {
        unsafe {
            core::arch::asm!(
                "str r1, [r0]",
                "dsb",
                "1:",
                "subs r2, #1",
                "bne 1b",
                // SCB.AIRCR = VECTKEY | SYSRESETREQ
                "str r4, [r3]",
                "dsb",
                "2:",
                "b 2b",
                in("r0") cr_reg,
                in("r1") cr,
                in("r2") iters,
                in("r3") 0xE000_ED0Cu32,
                in("r4") 0x05FA_0004u32,
                options(noreturn),
            )
        }
    }

    /// The delay loop of [`ram_erase_and_reset`], for calibration.
    #[cfg(ecc_reset)]
    #[unsafe(link_section = ".data.flash_ecc")]
    #[inline(never)]
    unsafe extern "C" fn spin(iters: u32) {
        unsafe {
            core::arch::asm!(
                "1:",
                "subs r0, #1",
                "bne 1b",
                inout("r0") iters => _,
            )
        }
    }

    #[cfg(ecc_reset)]
    fn check(read: Read, p: &Planted) {
        // Some bits of the sector are only partially erased and read either way, so which words
        // report an error varies from read to read. Some must.
        let mut buf = [0u8; WRITE_SIZE];
        let mut errors = 0;
        for offset in (p.offset..p.offset + p.size).step_by(WRITE_SIZE) {
            match read(offset, &mut buf) {
                Err(Error::Ecc) => errors += 1,
                Err(e) => defmt::panic!("read error {}", e),
                Ok(()) => {}
            }
        }
        defmt::assert!(errors > 0);

        // Long reads.
        let mut long = [0u8; 256];
        let mut errors = 0;
        for offset in (p.offset..p.offset + p.size).step_by(long.len()) {
            match read(offset, &mut long) {
                Err(Error::Ecc) => errors += 1,
                Err(e) => defmt::panic!("read error {}", e),
                Ok(()) => {}
            }
        }
        defmt::assert!(errors > 0);
    }
}
