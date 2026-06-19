#![no_std]
#![no_main]

//! FlexSPI NOR **async / DMA cancellation soak** for the FRDM-MCXA577
//! (Winbond W25Q64, 8 MiB).
//!
//! Companion to `flexspi-soak` (which torture-tests the *blocking* + AHB paths).
//! This one drives the **async + DMA** path and, crucially, validates that the
//! driver is **cancel-safe**: a dropped in-flight future must not wedge the
//! controller for the next operation. It is the hardware proof for the
//! self-healing `prepare_ip_transfer` recovery -- which is what makes the
//! application-level `with_timeout` pattern (the intended way to bound an
//! operation) usable: when the caller's timeout cancels an op mid-flight, the
//! driver un-wedges itself before the next one.
//!
//! ## DESTRUCTIVE
//! Erases and reprograms a rotating window of the external flash, repeatedly.
//! Do not keep anything you care about on the W25Q64 before running it.
//!
//! ## What it does every round (wear-spread rotating 2-sector window)
//! - **Phase A - async/DMA integrity:** erase the window, verify erased, program
//!   s0 with random 8-byte-aligned sub-page writes and s1 with full pages (the
//!   DMA write path), verify both via async read (the DMA read path), and
//!   cross-check s0 against the memory-mapped (AHB) window. Fatal on mismatch.
//! - **Phase B - cancellation torture:** repeatedly start an async op and drop
//!   it in flight at a sweep of `with_timeout` delays (from ~1 us, which lands
//!   mid command-shift, up to ~2 ms, mid write-in-progress), then immediately
//!   run a *fresh* op and assert it succeeds with correct data. Cancelling
//!   **reads** exercises pure controller recovery (a read leaves the device
//!   idle); cancelling **program/erase** adds the device-busy path, reconverged
//!   with an erase-until-clean settle. A regression (no cancel-safety) would
//!   **hang** the next op forever, so the test simply finishing -- across
//!   thousands of cancellations -- is itself the pass signal.
//! - a never-erased **canary** sector is verified every round (wild-write net).
//!
//! ## Repro
//! Seeded xorshift PRNG; the seed and per-round state are logged. Data-integrity
//! failures `panic!` (panic-probe halts) with full context. The run is bounded
//! by a wall-clock limit and then halts (`bkpt`).
//!
//! Run: `cargo run --release --bin flexspi-cancel-soak`

use defmt::{info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_mcxa::{bind_interrupts, peripherals};
use embassy_time::{Duration, Instant, with_timeout};
use hal::config::Config;
use hal::flexspi::{self, Async, ClockConfig as FlexspiClockConfig, Flexspi, NorFlash};
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

// ----- wear management (mirrors flexspi-soak) -----
const WINDOW_SECTORS: u32 = 2;
const CANARY_SECTOR: u32 = NUM_SECTORS - 1; // 2047, never erased after init
const ROT_MOD: u32 = NUM_SECTORS - WINDOW_SECTORS; // 2046
const ROT_STRIDE: u32 = 1021; // prime, coprime with 2046 -> even wear
const ERASE_BUDGET: u16 = 50_000;

// ----- cancellation sweep -----
/// `with_timeout` delays (microseconds) used to drop an in-flight op at varied
/// points: ~1-20 us lands mid command-shift (SEQIDLE low -> exercises the
/// software-reset recovery in `prepare_ip_transfer`); ~0.1-2 ms lands mid
/// write-in-progress poll (device busy, engine idle).
const CANCEL_DELAYS_US: &[u64] = &[1, 3, 8, 20, 60, 200, 800, 2000];

// ----- reporting -----
const SEED: u32 = 0x0bad_c0de;
const HEARTBEAT_ROUNDS: u64 = 8;
/// Wall-clock limit; the soak stops and halts (bkpt) when reached.
const SOAK_DURATION_SECS: u64 = 120;

bind_interrupts!(struct Irqs {
    FLEXSPI0 => flexspi::InterruptHandler<peripherals::FLEXSPI0>;
});

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

/// Async IP-read `[addr, addr+len)` and assert it matches `pattern_byte`. Fatal.
async fn expect_pattern(
    flash: &mut NorFlash<'_, Async>,
    addr: u32,
    len: usize,
    buf: &mut [u8],
    round: u64,
    phase: &str,
) {
    unwrap!(flash.read_async(addr, &mut buf[..len]).await);
    if let Some(bad) = check_pattern(addr, &buf[..len]) {
        let got = buf[(bad - addr) as usize];
        defmt::panic!(
            "CANCEL-SOAK FAIL [{=str}] round={=u64} pattern mismatch @0x{=u32:08x} exp=0x{=u8:02x} got=0x{=u8:02x} (read start=0x{=u32:08x} len={=usize})",
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

/// Async IP-read `[addr, addr+len)` and assert it is fully erased (0xFF). Fatal.
async fn expect_erased(
    flash: &mut NorFlash<'_, Async>,
    addr: u32,
    len: usize,
    buf: &mut [u8],
    round: u64,
    phase: &str,
) {
    unwrap!(flash.read_async(addr, &mut buf[..len]).await);
    if let Some(bad) = check_erased(addr, &buf[..len]) {
        let got = buf[(bad - addr) as usize];
        defmt::panic!(
            "CANCEL-SOAK FAIL [{=str}] round={=u64} not erased @0x{=u32:08x} got=0x{=u8:02x} (read start=0x{=u32:08x} len={=usize})",
            phase,
            round,
            bad,
            got,
            addr,
            len
        );
    }
}

/// Program a whole sector with the address-derived pattern using random
/// **8-byte-aligned** sub-page writes (the driver's write-size contract).
async fn program_sector_split(flash: &mut NorFlash<'_, Async>, sector_addr: u32, page: &mut [u8; 256], prng: &mut u32) {
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
            unwrap!(
                flash
                    .page_program_async(page_addr + start, &page[start as usize..end as usize])
                    .await
            );
            start = end;
        }
    }
}

/// Program a whole sector with the pattern, one full page at a time (the DMA
/// write path: 256-byte, 8-aligned writes).
async fn program_sector_simple(flash: &mut NorFlash<'_, Async>, sector_addr: u32, page: &mut [u8; 256]) {
    for p in 0..PAGES_PER_SECTOR {
        let page_addr = sector_addr + p * PAGE;
        fill_pattern(page_addr, page);
        unwrap!(flash.page_program_async(page_addr, &page[..]).await);
    }
}

/// Erase `sector` and confirm it (via async read) until it reads all-0xFF, with
/// a bounded number of retries. After a cancelled erase/program the device may
/// still be mid-write; the first `erase_sector_async` recovers the controller
/// and its trailing WIP wait drains the in-flight op, and a second pass then
/// actually erases. Fatal if it never converges.
async fn settle_erase(flash: &mut NorFlash<'_, Async>, sector: u32, buf: &mut [u8], round: u64) {
    for _ in 0..6 {
        unwrap!(flash.erase_sector_async(sector).await);
        unwrap!(flash.read_async(sector, &mut buf[..SECTOR as usize]).await);
        if check_erased(sector, &buf[..SECTOR as usize]).is_none() {
            return;
        }
    }
    defmt::panic!(
        "CANCEL-SOAK FAIL [settle] round={=u64} sector 0x{=u32:08x} not erased after retries",
        round,
        sector
    );
}

/// `(offset within window, length)` async reads forced every round so the
/// FIFO/page/sector seams (and the DMA multi-chunk path) are always exercised.
const BOUNDARY: &[(u32, usize)] = &[
    (0, 1),
    (7, 13),
    (100, 100),   // crosses the 128 B IP-FIFO seam
    (200, 120),   // crosses the 256 B page seam
    (255, 257),   // crosses 256, multi-chunk
    (2048, 4096), // spans the 4096 B sector seam into the second sector
    (4095, 2),    // straddles the sector boundary
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Config::default());

    info!("FlexSPI NOR async/DMA CANCELLATION soak (DESTRUCTIVE to a rotating window of the external flash)");
    info!(
        "seed=0x{=u32:08x}  window={=u32} sectors  rotation: base += {=u32} mod {=u32}  canary sector={=u32}  limit={=u64}s",
        SEED, WINDOW_SECTORS, ROT_STRIDE, ROT_MOD, CANARY_SECTOR, SOAK_DURATION_SECS
    );

    let flexspi = unwrap!(Flexspi::new_with_dma(
        p.FLEXSPI0,
        p.P3_0,
        p.P3_7,
        p.P3_6,
        p.P3_8,
        p.P3_9,
        p.P3_10,
        p.P3_11,
        p.DMA0_CH0,
        p.DMA0_CH1,
        Irqs,
        FlexspiClockConfig::default(),
        FLASH_CONFIG,
    ));
    let mut flash = NorFlash::new(flexspi);

    // Vendor-id sanity (W25Q64 == 0xEF). 0x00/0x7F would mean a dead/floating bus.
    let id = unwrap!(flash.read_vendor_id_async().await);
    if id != 0xEF {
        defmt::panic!("unexpected vendor id 0x{=u8:02x} (expected 0xEF Winbond)", id);
    }
    info!("vendor id 0x{=u8:02x} OK", id);

    let mut buf = [0u8; 4096];
    let mut page = [0u8; 256];
    let mut erase_count = [0u16; NUM_SECTORS as usize];

    // ---- one-time: program the canary sector (never erased afterwards) ----
    let canary_addr = CANARY_SECTOR * SECTOR;
    unwrap!(flash.erase_sector_async(canary_addr).await);
    erase_count[CANARY_SECTOR as usize] += 1;
    program_sector_simple(&mut flash, canary_addr, &mut page).await;
    expect_pattern(&mut flash, canary_addr, SECTOR as usize, &mut buf, 0, "canary-init").await;
    info!("canary programmed at 0x{=u32:08x}", canary_addr);

    info!("starting cancellation soak loop");

    // ---- soak loop ----
    let mut prng = SEED;
    let mut base: u32 = 0;
    let mut round: u64 = 0;
    let mut total_erases: u64 = 0;
    let mut cancels: u64 = 0;
    let mut cancel_inflight: u64 = 0; // dropped while the op was still pending
    let mut cancel_completed: u64 = 0; // op finished before the cancel point
    let mut recoveries: u64 = 0; // fresh ops that succeeded right after a cancel
    let start = Instant::now();

    loop {
        let prng_at_round = prng;
        let s0 = base * SECTOR;
        let s1 = (base + 1) * SECTOR;

        if start.elapsed() >= Duration::from_secs(SOAK_DURATION_SECS) {
            warn!("cancel-soak time limit ({=u64}s) reached", SOAK_DURATION_SECS);
            break;
        }
        let max_in_window = erase_count[base as usize].max(erase_count[(base + 1) as usize]);
        if max_in_window >= ERASE_BUDGET {
            warn!(
                "erase budget reached on sector {=u32} ({=u16} erases) -- stopping to protect the flash",
                base, max_in_window
            );
            break;
        }

        // ===== Phase A: async/DMA data integrity =====
        unwrap!(flash.erase_sector_async(s0).await);
        unwrap!(flash.erase_sector_async(s1).await);
        erase_count[base as usize] += 1;
        erase_count[(base + 1) as usize] += 1;
        total_erases += 2;
        expect_erased(&mut flash, s0, SECTOR as usize, &mut buf, round, "A-erase-s0").await;
        expect_erased(&mut flash, s1, SECTOR as usize, &mut buf, round, "A-erase-s1").await;

        program_sector_split(&mut flash, s0, &mut page, &mut prng).await;
        program_sector_simple(&mut flash, s1, &mut page).await;

        expect_pattern(&mut flash, s0, SECTOR as usize, &mut buf, round, "A-verify-s0").await;
        expect_pattern(&mut flash, s1, SECTOR as usize, &mut buf, round, "A-verify-s1").await;

        // mmap (AHB) cross-check of s0 against the async-read ground truth.
        unwrap!(flash.read_async(s0, &mut buf[..SECTOR as usize]).await);
        for i in 0..SECTOR {
            let m = mmap_byte(s0 + i);
            let ip = buf[i as usize];
            let exp = pattern_byte(s0 + i);
            if m != ip || m != exp {
                defmt::panic!(
                    "CANCEL-SOAK FAIL [A-mmap-xcheck] round={=u64} @0x{=u32:08x} mmap=0x{=u8:02x} ip=0x{=u8:02x} exp=0x{=u8:02x} (prng=0x{=u32:08x})",
                    round,
                    s0 + i,
                    m,
                    ip,
                    exp,
                    prng_at_round
                );
            }
        }
        for &(off, len) in BOUNDARY {
            expect_pattern(&mut flash, s0 + off, len, &mut buf, round, "A-boundary").await;
        }

        // ===== Phase B1: cancel READS, then prove a clean read recovers =====
        // A read leaves the device idle, so any wedge here is purely the
        // controller -> this isolates the cancel-safety recovery. `with_timeout`
        // is the application-level way to bound an op: when it elapses it drops
        // the in-flight future, which is exactly the cancellation we recover from.

        // A cancelled vendor-id read (smallest command) then a clean one.
        let _ = with_timeout(Duration::from_micros(1), flash.read_vendor_id_async()).await;
        cancels += 1;
        let id = unwrap!(flash.read_vendor_id_async().await);
        if id != 0xEF {
            defmt::panic!(
                "CANCEL-SOAK FAIL [B1-vendor-recover] round={=u64} vendor id 0x{=u8:02x} after cancelled read",
                round,
                id
            );
        }
        recoveries += 1;

        // Timed cancels across the delay sweep.
        for &d_us in CANCEL_DELAYS_US {
            match with_timeout(
                Duration::from_micros(d_us),
                flash.read_async(s0, &mut buf[..SECTOR as usize]),
            )
            .await
            {
                Ok(inner) => {
                    unwrap!(inner); // a completed read must not error
                    cancel_completed += 1;
                }
                Err(_) => cancel_inflight += 1,
            }
            cancels += 1;
            // Fresh read must return correct data -> controller recovered.
            expect_pattern(&mut flash, s0, SECTOR as usize, &mut buf, round, "B1-timed-recover").await;
            recoveries += 1;
        }

        // ===== Phase B2: cancel PROGRAM/ERASE, settle, reverify =====
        // These can leave the device mid-write; settle_erase reconverges it.
        // 1-2 us lands mid command-shift; 30 us / 1 ms lands mid write-in-progress.
        fill_pattern(s0, &mut page);
        let _ = with_timeout(Duration::from_micros(2), flash.page_program_async(s0, &page[..])).await;
        cancels += 1;
        let _ = with_timeout(Duration::from_micros(30), flash.page_program_async(s0, &page[..])).await;
        cancels += 1;
        let _ = with_timeout(Duration::from_micros(2), flash.erase_sector_async(s0)).await;
        cancels += 1;
        let _ = with_timeout(Duration::from_millis(1), flash.erase_sector_async(s0)).await;
        cancels += 1;

        // Recovery + device reconvergence: erase-until-clean, then a fresh
        // program/read round-trips correctly.
        settle_erase(&mut flash, s0, &mut buf, round).await;
        erase_count[base as usize] += 2; // settle does at least one extra erase
        total_erases += 2;
        program_sector_simple(&mut flash, s0, &mut page).await;
        expect_pattern(&mut flash, s0, SECTOR as usize, &mut buf, round, "B2-recover").await;
        recoveries += 1;

        // ===== canary integrity (wild-write detector), every round =====
        expect_pattern(&mut flash, canary_addr, SECTOR as usize, &mut buf, round, "canary").await;
        unwrap!(flash.read_async(canary_addr, &mut buf[..PAGE as usize]).await);
        for i in 0..PAGE {
            if mmap_byte(canary_addr + i) != buf[i as usize] {
                defmt::panic!(
                    "CANCEL-SOAK FAIL [canary-xcheck] round={=u64} @0x{=u32:08x} mmap != ip",
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
                "[hb] round={=u64} erases={=u64} cancels={=u64} (inflight={=u64} completed={=u64}) recoveries={=u64} max_sector_erases={=u16} next_base={=u32}",
                round, total_erases, cancels, cancel_inflight, cancel_completed, recoveries, max_erase, base
            );
        }
    }

    info!(
        "cancel-soak finished: rounds={=u64} erases={=u64} cancels={=u64} (inflight={=u64} completed={=u64}) recoveries={=u64}",
        round, total_erases, cancels, cancel_inflight, cancel_completed, recoveries
    );
    loop {
        cortex_m::asm::bkpt();
    }
}
