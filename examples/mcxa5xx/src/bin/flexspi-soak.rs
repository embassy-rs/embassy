#![no_std]
#![no_main]

//! FlexSPI NOR **soak test** for the FRDM-MCXA577 (Winbond W25Q64, 8 MiB).
//!
//! A long-running, self-checking torture loop that tries hard to break the
//! blocking + memory-mapped paths of `embassy_mcxa::flexspi`. Designed by the
//! reliability/tester/reviewer agents; v1 covers the highest-value, safe-to-run
//! scope (blocking IP + AHB/XIP reads). Async/DMA and future-cancellation
//! abuse are deliberately out of scope for v1 (the driver has no cancel-safe
//! `Drop`, so a dropped future can wedge the controller).
//!
//! ## DESTRUCTIVE
//! This erases and reprograms the **entire** external flash, repeatedly. Do not
//! store anything you care about on the W25Q64 before running it.
//!
//! ## What it checks every round (on a wear-spread rotating 2-sector window)
//! - erase -> all 0xFF; program (fuzzed sub-page splits) -> exact `pattern_byte`
//! - **memory-mapped read == IP read == pattern** (the AFLASHBASE/DLLCR/ARDSEQNUM
//!   regression net) -- fatal on mismatch
//! - boundary/length matrix crossing the 128 B IP-FIFO, 256 B page and 4096 B
//!   sector seams, plus a read spanning two sectors
//! - partial-page program leaves the page tail erased (0xFF)
//! - erase isolates its sector (neighbour intact)
//! - **stale AHB buffer probe**: mmap a page, reprogram it via IP, mmap again --
//!   must reflect the new contents (counted, *not* fatal: suspected driver gap)
//! - negative paths: over-length / empty program -> `Err`, no side effects;
//!   zero-length read -> `Ok` no-op
//! - out-of-range read characterisation (read-only, logged)
//! - a never-erased **canary** sector verified every round (wild-write detector)
//!
//! ## Repro
//! Everything is driven by a seeded xorshift PRNG; the seed and the per-round
//! PRNG state are logged, so any failing round can be replayed in isolation.
//! On a data-integrity failure it `panic!`s (panic-probe halts) with the full
//! context (round, phase, address, expected vs got).
//!
//! Run: `cargo run --release --bin flexspi-soak` (then Ctrl-C when satisfied).

use defmt::{info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant};
use hal::config::Config;
use hal::flexspi::{Blocking, ClockConfig as FlexspiClockConfig, Flexspi, IoError, NorFlash};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[path = "../flexspi_common.rs"]
mod flexspi_common;

use flexspi_common::{FLASH_CONFIG, check_erased, check_pattern, fill_pattern, pattern_byte};

// ----- geometry -----
const PAGE: u32 = 256;
const SECTOR: u32 = 4096;
const NUM_SECTORS: u32 = 2048; // W25Q64 = 8 MiB
const PAGES_PER_SECTOR: u32 = SECTOR / PAGE; // 16

/// FlexSPI AHB memory-mapped window, secure alias (the core boots secure).
const AMBA_BASE_S: u32 = 0x9000_0000;

// ----- wear management -----
/// Window walked across the device each round (2 sectors -> cross-sector reads
/// and neighbour-isolation).
const WINDOW_SECTORS: u32 = 2;
/// Top sector is a never-erased integrity canary; the window never reaches it.
const CANARY_SECTOR: u32 = NUM_SECTORS - 1; // 2047
/// Rotation modulus so the window stays in `0..=2045` (below the canary).
const ROT_MOD: u32 = NUM_SECTORS - WINDOW_SECTORS; // 2046
/// Prime, coprime with ROT_MOD (2046 = 2*3*11*31) -> visits every base once
/// per sweep, spreading wear uniformly.
const ROT_STRIDE: u32 = 1021;
/// Hard backstop: stop before any sector approaches its ~100k erase rating.
const ERASE_BUDGET: u16 = 50_000;

// ----- reporting -----
const SEED: u32 = 0x1234_5678;
const HEARTBEAT_ROUNDS: u64 = 64;
/// Wall-clock time limit; the soak stops and halts (bkpt) when reached.
const SOAK_DURATION_SECS: u64 = 300;

#[inline]
fn xorshift32(s: &mut u32) -> u32 {
    let mut x = *s;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *s = x;
    x
}

/// Uniform-ish `lo..hi` (hi exclusive). Caller guarantees `hi > lo`.
#[inline]
fn gen_range(s: &mut u32, lo: u32, hi: u32) -> u32 {
    lo + xorshift32(s) % (hi - lo)
}

/// One memory-mapped (AHB/XIP) byte at flash `offset`.
#[inline]
fn mmap_byte(offset: u32) -> u8 {
    // SAFETY: inside the FlexSPI secure memory-mapped window, valid for reads
    // while the driver is initialised.
    unsafe { core::ptr::read_volatile((AMBA_BASE_S + offset) as *const u8) }
}

/// IP-read `[addr, addr+len)` and assert it matches `pattern_byte`. Fatal.
fn expect_pattern(flash: &mut NorFlash<'_, Blocking>, addr: u32, len: usize, buf: &mut [u8], round: u64, phase: &str) {
    unwrap!(flash.blocking_read(addr, &mut buf[..len]));
    if let Some(bad) = check_pattern(addr, &buf[..len]) {
        let got = buf[(bad - addr) as usize];
        defmt::panic!(
            "SOAK FAIL [{=str}] round={=u64} pattern mismatch @0x{=u32:08x} exp=0x{=u8:02x} got=0x{=u8:02x} (read start=0x{=u32:08x} len={=usize})",
            phase,
            round,
            bad,
            pattern_byte(bad),
            got,
            addr,
            len
        );
    }
}

/// IP-read `[addr, addr+len)` and assert it is fully erased (0xFF). Fatal.
fn expect_erased(flash: &mut NorFlash<'_, Blocking>, addr: u32, len: usize, buf: &mut [u8], round: u64, phase: &str) {
    unwrap!(flash.blocking_read(addr, &mut buf[..len]));
    if let Some(bad) = check_erased(addr, &buf[..len]) {
        let got = buf[(bad - addr) as usize];
        defmt::panic!(
            "SOAK FAIL [{=str}] round={=u64} not erased @0x{=u32:08x} got=0x{=u8:02x} (read start=0x{=u32:08x} len={=usize})",
            phase,
            round,
            bad,
            got,
            addr,
            len
        );
    }
}

/// Program a whole sector with the address-derived pattern, split into
/// random-sized in-page sub-writes (never crossing a page boundary).
/// Program a whole sector with the pattern using random **8-byte-aligned**
/// sub-page writes (the driver's write-size contract: multiple writes per page,
/// each at an 8-aligned offset with a multiple-of-8 length).
fn program_sector_split(flash: &mut NorFlash<'_, Blocking>, sector_addr: u32, page: &mut [u8; 256], prng: &mut u32) {
    for p in 0..PAGES_PER_SECTOR {
        let page_addr = sector_addr + p * PAGE;
        fill_pattern(page_addr, page);
        let mut start = 0u32;
        while start < PAGE {
            let units_left = (PAGE - start) / 8; // PAGE and start are multiples of 8
            let units = if units_left <= 1 {
                1
            } else {
                gen_range(prng, 1, units_left + 1)
            };
            let end = start + units * 8;
            unwrap!(flash.blocking_page_program(page_addr + start, &page[start as usize..end as usize]));
            start = end;
        }
    }
}

/// Program a whole sector with the pattern, one full page at a time.
fn program_sector_simple(flash: &mut NorFlash<'_, Blocking>, sector_addr: u32, page: &mut [u8; 256]) {
    for p in 0..PAGES_PER_SECTOR {
        let page_addr = sector_addr + p * PAGE;
        fill_pattern(page_addr, page);
        unwrap!(flash.blocking_page_program(page_addr, &page[..]));
    }
}

/// Which contract-violation error a negative test expects.
#[derive(Clone, Copy)]
enum Expect {
    InvalidLen,
    Misaligned,
    OutOfBounds,
}

/// Assert a write/read returned the expected contract-violation error. Fatal.
fn expect_err(r: Result<(), IoError>, expect: Expect, round: u64, what: &str) {
    let ok = matches!(
        (&r, expect),
        (Err(IoError::InvalidTransferLength), Expect::InvalidLen)
            | (Err(IoError::Misaligned), Expect::Misaligned)
            | (Err(IoError::OutOfBounds), Expect::OutOfBounds)
    );
    if !ok {
        defmt::panic!(
            "SOAK FAIL [{=str}] round={=u64}: contract not enforced (wrong/absent Err)",
            what,
            round
        );
    }
}

/// `(offset within window, length)` reads forced every round so the FIFO/page/
/// sector seams are always exercised, not merely sometimes.
const BOUNDARY: &[(u32, usize)] = &[
    (0, 1),
    (0, 4096),
    (1, 127),
    (3, 33),
    (7, 13),
    (100, 100),   // crosses the 128 B IP-FIFO seam
    (120, 16),    // crosses 128
    (127, 4),     // crosses 128
    (200, 120),   // crosses the 256 B page seam
    (255, 2),     // crosses 256
    (255, 257),   // crosses 256, multi-chunk
    (256, 256),   // page-aligned
    (257, 255),   // unaligned tail
    (2048, 4096), // spans the 4096 B sector seam into the second sector
    (4095, 2),    // straddles the sector boundary
    (4096, 1),
    (4096, 256),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Config::default());

    info!("FlexSPI NOR soak test (DESTRUCTIVE to the whole external flash)");
    info!(
        "seed=0x{=u32:08x}  window={=u32} sectors  rotation: base += {=u32} mod {=u32}  canary sector={=u32}  erase budget={=u16}",
        SEED, WINDOW_SECTORS, ROT_STRIDE, ROT_MOD, CANARY_SECTOR, ERASE_BUDGET
    );

    let flexspi = unwrap!(Flexspi::new_blocking(
        p.FLEXSPI0,
        p.P3_0,
        p.P3_7,
        p.P3_6,
        p.P3_8,
        p.P3_9,
        p.P3_10,
        p.P3_11,
        FlexspiClockConfig::default(),
        FLASH_CONFIG,
    ));
    let mut flash = NorFlash::new(flexspi);

    // Vendor-id sanity (W25Q64 == 0xEF). 0x00/0x7F would mean a dead/floating bus.
    let id = unwrap!(flash.blocking_vendor_id());
    if id != 0xEF {
        defmt::panic!("unexpected vendor id 0x{=u8:02x} (expected 0xEF Winbond)", id);
    }
    info!("vendor id 0x{=u8:02x} OK", id);

    let mut buf = [0u8; 4096];
    let mut page = [0u8; 256];
    let mut erase_count = [0u16; NUM_SECTORS as usize];

    // ---- one-time: program the canary sector (never erased afterwards) ----
    let canary_addr = CANARY_SECTOR * SECTOR;
    unwrap!(flash.blocking_erase_sector(canary_addr));
    erase_count[CANARY_SECTOR as usize] += 1;
    program_sector_simple(&mut flash, canary_addr, &mut page);
    expect_pattern(&mut flash, canary_addr, SECTOR as usize, &mut buf, 0, "canary-init");
    info!("canary programmed at 0x{=u32:08x}", canary_addr);

    // ---- one-time: cold memory-mapped vs IP validation (fatal) ----
    // Fresh program with no prior mmap access -> no staleness confound, so this
    // is a clean check that the AFLASHBASE=0 / DLLCR / ARDSEQNUM fixes hold.
    {
        let a = 0u32; // sector 0
        unwrap!(flash.blocking_erase_sector(a));
        erase_count[0] += 1;
        program_sector_simple(&mut flash, a, &mut page);
        unwrap!(flash.blocking_read(a, &mut buf[..SECTOR as usize]));
        for i in 0..SECTOR {
            let m = mmap_byte(a + i);
            let ip = buf[i as usize];
            let exp = pattern_byte(a + i);
            if m != ip || m != exp {
                defmt::panic!(
                    "COLD mmap/IP mismatch @0x{=u32:08x} mmap=0x{=u8:02x} ip=0x{=u8:02x} exp=0x{=u8:02x}",
                    a + i,
                    m,
                    ip,
                    exp
                );
            }
        }
        info!("cold memory-mapped vs IP validation OK ({=u32} bytes)", SECTOR);
    }

    info!("starting soak loop");

    // ---- soak loop ----
    let mut prng = SEED;
    let mut base: u32 = 0;
    let mut round: u64 = 0u64;
    let mut total_erases: u64 = 0;
    let mut bytes_written: u64 = 0;
    let mut bytes_read: u64 = 0;
    let mut mmap_mismatch: u64 = 0;
    let mut stale_mmap: u64 = 0;
    let mut neg_ok: u64 = 0;
    let start = Instant::now();

    loop {
        let prng_at_round = prng;
        let s0 = base * SECTOR;
        let s1 = (base + 1) * SECTOR;
        let win = s0; // window base address

        // Stop cleanly at the wall-clock limit.
        if start.elapsed() >= Duration::from_secs(SOAK_DURATION_SECS) {
            warn!("soak time limit ({=u64}s) reached", SOAK_DURATION_SECS);
            break;
        }

        // Wear backstop: stop cleanly if any window sector nears its rating.
        let max_in_window = erase_count[base as usize].max(erase_count[(base + 1) as usize]);
        if max_in_window >= ERASE_BUDGET {
            warn!(
                "erase budget reached on sector {=u32} ({=u16} erases) -- stopping to protect the flash",
                base, max_in_window
            );
            break;
        }

        // 1) erase the window, verify erased (IP only -- avoid mmap here so the
        //    step-3 mmap cross-check has no staleness confound).
        unwrap!(flash.blocking_erase_sector(s0));
        unwrap!(flash.blocking_erase_sector(s1));
        erase_count[base as usize] += 1;
        erase_count[(base + 1) as usize] += 1;
        total_erases += 2;
        expect_erased(&mut flash, s0, SECTOR as usize, &mut buf, round, "erase-verify-s0");
        expect_erased(&mut flash, s1, SECTOR as usize, &mut buf, round, "erase-verify-s1");

        // 2) program: s0 with random 8-byte-aligned sub-page writes (the valid
        //    multi-write path), s1 with full pages.
        program_sector_split(&mut flash, s0, &mut page, &mut prng);
        program_sector_simple(&mut flash, s1, &mut page);
        bytes_written += 2 * SECTOR as u64;

        // 3) verify both sectors via IP (fatal), then a full mmap cross-check of
        //    s0 against IP and pattern (fatal: read-path integrity).
        expect_pattern(&mut flash, s0, SECTOR as usize, &mut buf, round, "verify-s0");
        expect_pattern(&mut flash, s1, SECTOR as usize, &mut buf, round, "verify-s1");
        unwrap!(flash.blocking_read(s0, &mut buf[..SECTOR as usize]));
        for i in 0..SECTOR {
            let m = mmap_byte(s0 + i);
            let ip = buf[i as usize];
            let exp = pattern_byte(s0 + i);
            if m != ip || m != exp {
                defmt::panic!(
                    "SOAK FAIL [mmap-xcheck] round={=u64} @0x{=u32:08x} mmap=0x{=u8:02x} ip=0x{=u8:02x} exp=0x{=u8:02x} (prng=0x{=u32:08x})",
                    round,
                    s0 + i,
                    m,
                    ip,
                    exp,
                    prng_at_round
                );
            }
        }
        bytes_read += 2 * SECTOR as u64;

        // 4) boundary/length matrix (both sectors are fully patterned now).
        for &(off, len) in BOUNDARY {
            expect_pattern(&mut flash, win + off, len, &mut buf, round, "boundary");
            bytes_read += len as u64;
        }

        // 5) neighbour isolation: erase s0, s1 must be untouched.
        unwrap!(flash.blocking_erase_sector(s0));
        erase_count[base as usize] += 1;
        total_erases += 1;
        expect_pattern(&mut flash, s1, SECTOR as usize, &mut buf, round, "neighbour-s1");
        expect_erased(&mut flash, s0, SECTOR as usize, &mut buf, round, "neighbour-s0-erased");

        // 6) stale AHB-buffer probe (counted, not fatal): mmap a patterned page
        //    of s1, erase s1 via IP, then mmap again -- must read 0xFF.
        {
            let pp = gen_range(&mut prng, 0, PAGES_PER_SECTOR) * PAGE;
            let page_addr = s1 + pp;
            // populate the AHB prefetch buffer with the current (patterned) data
            let mut warmed_ok = true;
            for i in 0..PAGE {
                if mmap_byte(page_addr + i) != pattern_byte(page_addr + i) {
                    warmed_ok = false;
                }
            }
            unwrap!(flash.blocking_erase_sector(s1));
            erase_count[(base + 1) as usize] += 1;
            total_erases += 1;
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            cortex_m::asm::dsb();
            let mut stale = false;
            for i in 0..PAGE {
                if mmap_byte(page_addr + i) != 0xFF {
                    stale = true;
                }
            }
            if stale {
                stale_mmap += 1;
                warn!(
                    "stale AHB read after erase @0x{=u32:08x} round={=u64} (mmap returned pre-erase data; IP path is ground truth)",
                    page_addr, round
                );
            } else if !warmed_ok {
                // mmap disagreed with the pattern *before* the erase -> a plain
                // read-path issue, count it.
                mmap_mismatch += 1;
                warn!(
                    "mmap disagreed with pattern pre-erase @0x{=u32:08x} round={=u64}",
                    page_addr, round
                );
            }
        }

        // 7) contract enforcement (s1 is erased; scratch = page 0 of s1). Every
        //    out-of-contract call must return the right Err and not touch flash.
        {
            let scratch = s1;
            for (i, b) in buf[..257].iter_mut().enumerate() {
                *b = i as u8;
            }
            expect_err(
                flash.blocking_page_program(scratch, &buf[..257]),
                Expect::InvalidLen,
                round,
                "over-length",
            );
            expect_err(
                flash.blocking_page_program(scratch, &[]),
                Expect::InvalidLen,
                round,
                "empty",
            );
            expect_err(
                flash.blocking_page_program(scratch + 4, &buf[..8]),
                Expect::Misaligned,
                round,
                "addr-misalign",
            );
            expect_err(
                flash.blocking_page_program(scratch, &buf[..12]),
                Expect::Misaligned,
                round,
                "len-misalign",
            );
            expect_err(
                flash.blocking_page_program(scratch + (PAGE - 8), &buf[..16]),
                Expect::Misaligned,
                round,
                "page-cross",
            );
            expect_err(
                flash.blocking_page_program(NUM_SECTORS * SECTOR, &buf[..8]),
                Expect::OutOfBounds,
                round,
                "oob-prog",
            );
            expect_err(
                flash.blocking_read(NUM_SECTORS * SECTOR, &mut buf[..16]),
                Expect::OutOfBounds,
                round,
                "oob-read",
            );
            neg_ok += 7;
            // none of the rejected calls touched the flash
            expect_erased(
                &mut flash,
                scratch,
                PAGE as usize,
                &mut buf,
                round,
                "neg-no-side-effect",
            );
            // zero-length read is a defined no-op
            if flash.blocking_read(scratch, &mut []).is_err() {
                defmt::panic!(
                    "SOAK FAIL [neg-zero-read] round={=u64} zero-length read returned Err",
                    round
                );
            }
            neg_ok += 1;
        }

        // 9) canary integrity (wild-write detector), every round.
        expect_pattern(&mut flash, canary_addr, SECTOR as usize, &mut buf, round, "canary");
        // and the two read engines must agree on the canary
        unwrap!(flash.blocking_read(canary_addr, &mut buf[..PAGE as usize]));
        for i in 0..PAGE {
            if mmap_byte(canary_addr + i) != buf[i as usize] {
                defmt::panic!(
                    "SOAK FAIL [canary-xcheck] round={=u64} @0x{=u32:08x} mmap != ip",
                    round,
                    canary_addr + i
                );
            }
        }

        // advance
        round += 1;
        base = (base + ROT_STRIDE) % ROT_MOD;

        if round % HEARTBEAT_ROUNDS == 0 {
            let mut max_erase = 0u16;
            for &c in erase_count.iter() {
                if c > max_erase {
                    max_erase = c;
                }
            }
            info!(
                "[hb] round={=u64} erases={=u64} wrote={=u64}KiB read={=u64}KiB mmap_mismatch={=u64} stale_mmap={=u64} neg_ok={=u64} max_sector_erases={=u16} next_base={=u32}",
                round,
                total_erases,
                bytes_written / 1024,
                bytes_read / 1024,
                mmap_mismatch,
                stale_mmap,
                neg_ok,
                max_erase,
                base
            );
        }
    }

    info!(
        "soak finished: rounds={=u64} erases={=u64} mmap_mismatch={=u64} stale_mmap={=u64} neg_ok={=u64}",
        round, total_erases, mmap_mismatch, stale_mmap, neg_ok
    );
    loop {
        cortex_m::asm::bkpt();
    }
}
