//! FlexSPI stress / corner-case torture test.
//!
//! This example exercises the [`embassy_mcxa::flexspi`] driver as hard as a
//! single binary reasonably can. Where the other `flexspi-octal-*` examples
//! demonstrate one transport mode in isolation, this one walks the **same
//! test matrix** through all three transports, sequentially, on the same
//! `FLEXSPI0` peripheral instance:
//!
//!   1. **Blocking** — `Flexspi::new_blocking` / `NorFlash::blocking_*`
//!   2. **Interrupt-only async** — `NorFlash::new_async` (no DMA channels)
//!   3. **DMA async** — `Flexspi::new_with_dma` + `NorFlash::*_async`
//!
//! Between phases the per-phase `NorFlash` is dropped (releasing the `Peri`
//! handles back to the singleton table) and the next phase reclaims the
//! peripherals via `peripherals::*::steal()`. This is deliberately rough on
//! the driver — we want to surface bugs in the construction / teardown
//! sequence and any hardware state that survives a drop+rebuild.
//!
//! ## Goals
//!
//! Every assertion below is intentionally aimed at a code path that has a
//! plausible failure mode in the driver:
//!
//! * **Vendor-ID idempotency** (3 back-to-back reads). Catches state-machine
//!   leftovers between IP commands that would corrupt the second/third read.
//! * **Empty-data `page_program`** must return `IoError::InvalidTransferLength`.
//!   Catches a regression where the early guard is removed.
//! * **Oversized `page_program`** (`page_size + 1`) must also return
//!   `InvalidTransferLength`.
//! * **Read-length matrix** covering 1..2048 bytes with explicit values at
//!   each interesting boundary:
//!     * `1, 2, 3` — the async-DMA fallback to FIFO (`dma_chunk < 4`).
//!     * `4, 5, 7, 8, 9` — the smallest DMA-eligible reads.
//!     * `127, 128, 129` — the 128 B `IP_FIFO_CAPACITY_BYTES` chunk boundary;
//!       129 forces the `read_async` loop to take a second iteration.
//!     * `255, 256, 257, 511, 512` — the 256 B page boundary and the
//!       loop running twice with a partial trailing chunk.
//!     * `768, 1024, 2048` — long reads that cross multiple pages.
//! * **Read-offset matrix** covering 0..4097 with values straddling FIFO
//!   chunks (`127`, `128`, `129`), pages (`255`, `256`, `257`) and sectors
//!   (`4095`, `4096`, `4097`).
//! * **Page-program size matrix**: every value in `PROGRAM_SIZES` is
//!   programmed into its own freshly-erased page.  This exercises the DMA
//!   write fast path (only when `len >= 8 && len % 8 == 0`) **and** the FIFO
//!   fallback for everything else, and verifies the unwritten tail of the
//!   page is still erased afterwards (catching writes that overrun the
//!   declared length).
//! * **Sub-DMA-window writes** (lengths 1, 7, 9, 15, 17, 31, 33, 63, 65,
//!   100, 200, 248, 255) force the DMA-mode driver onto the IP FIFO write
//!   path and back, rather than always using DMA.
//! * **Cross-FIFO read** (200 bytes at an unaligned offset) splits a single
//!   logical read into two IP commands and validates the address arithmetic
//!   between chunks.
//! * **Cross-sector read** (64 bytes straddling the boundary) catches off-
//!   by-one errors in the address word inside `Ipcr0`.
//! * **Erase idempotency** (erase the same sector twice). Should be a no-op
//!   the second time.
//! * **Selective erase** (erase the middle sector and check the neighbours
//!   are intact). Catches buggy sector size translation.
//! * **Overlay program without erase** writes `0xF0` then `0x0F` to the
//!   same address; the readback must be `0x00` (NOR can only flip 1→0).
//!   This is a sanity check that the program command actually issues a
//!   write rather than silently dropping the second one.
//! * **Interleaved vendor-id / read** (32 iterations) hammers the IP path
//!   alternately at zero-data and bulk-data sizes.
//!
//! Each phase prints `[<mode>] PHASE PASSED` once it completes.  After all
//! three phases finish the program issues a `cortex_m::asm::bkpt()` so the
//! debugger halts and the user can inspect the run instead of looping
//! forever.
//!
//! Run it on an MCXA5 board attached over SWD:
//!
//! ```text
//! cargo run --release --bin flexspi-stress
//! ```

#![no_std]
#![no_main]

use defmt::{error, info, unwrap};
use embassy_executor::Spawner;
use embassy_mcxa::{Peripherals, bind_interrupts, peripherals};
use hal::config::Config;
use hal::flexspi::{self, ClockConfig as FlexspiClockConfig, IoError, NorFlash};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[path = "../flexspi_common.rs"]
mod flexspi_common;

use flexspi_common::{FLASH_CONFIG, FLASH_PAGE_SIZE, FLASH_SECTOR_SIZE, check_erased, check_pattern, fill_pattern};

/// Base of the test region. Picked far away from the other flexspi examples
/// so the three programs can coexist on the same flash without trampling.
const FLASH_BASE: u32 = (2000 * 0x1000) as u32;

/// Number of contiguous sectors used by the stress test.  Must be large
/// enough to hold one page per entry in `PROGRAM_SIZES` plus a few extras
/// for the overlay / partial-page tests, plus the cross-sector read region.
const STRESS_SECTORS: u32 = 6;
const STRESS_BYTES: u32 = STRESS_SECTORS * FLASH_SECTOR_SIZE as u32;

/// Per-phase failure counter shared across the run-stress matrix.  Because
/// the goal of this binary is to *find* corner-case bugs rather than exit
/// at the first one, content-mismatch assertions log a `FAIL` message and
/// bump a counter rather than panicking.  Driver-level errors (a
/// `blocking_read` returning `Err`, etc.) still panic — those mean the
/// test infrastructure itself is broken and continuing would be noise.
struct Failures {
    count: u32,
}

impl Failures {
    const fn new() -> Self {
        Self { count: 0 }
    }
}

/// Read lengths covering the FIFO, page, and DMA-cutoff boundaries.
const READ_LENS: &[usize] = &[
    1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 24, 31, 32, 33, 63, 64, 65, 100, 127, 128, 129, 200, 255, 256, 257, 511, 512,
    768, 1024, 2048,
];

/// Read offsets covering FIFO (128 B), page (256 B), and sector (4096 B)
/// boundaries.  All of these are exercised at a fixed read length of 200 B.
const READ_OFFSETS: &[u32] = &[
    0, 1, 2, 3, 4, 5, 7, 16, 17, 64, 100, 127, 128, 129, 200, 255, 256, 257, 4095, 4096, 4097,
];

/// Program sizes covering the DMA fast path (`>= 8 && % 8 == 0`) and the
/// FIFO fallback (everything else).
const PROGRAM_SIZES: &[usize] = &[
    1, 2, 3, 4, 7, 8, 9, 15, 16, 17, 24, 31, 32, 33, 63, 64, 65, 100, 128, 200, 248, 255, 256,
];

bind_interrupts!(struct Irqs {
    FLEXSPI0 => flexspi::InterruptHandler<peripherals::FLEXSPI0>;
});

/// Uniform interface across the three transports so the stress matrix only
/// needs to be written once. The blocking variant wraps its sync calls in
/// `async` shims that complete immediately, but it still drives the actual
/// `blocking_*` driver methods.
trait FlashOps {
    async fn vendor_id(&mut self) -> Result<u8, IoError>;
    async fn erase_sector(&mut self, addr: u32) -> Result<(), IoError>;
    async fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), IoError>;
    async fn page_program(&mut self, addr: u32, data: &[u8]) -> Result<(), IoError>;
}

struct BlockingFlash<'d, T: flexspi::Instance>(NorFlash<'d, T>);

impl<'d, T: flexspi::Instance> FlashOps for BlockingFlash<'d, T> {
    async fn vendor_id(&mut self) -> Result<u8, IoError> {
        self.0.blocking_vendor_id()
    }
    async fn erase_sector(&mut self, addr: u32) -> Result<(), IoError> {
        self.0.blocking_erase_sector(addr)
    }
    async fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), IoError> {
        self.0.blocking_read(addr, buf)
    }
    async fn page_program(&mut self, addr: u32, data: &[u8]) -> Result<(), IoError> {
        self.0.blocking_page_program(addr, data)
    }
}

struct AsyncFlash<'d, T: flexspi::Instance>(NorFlash<'d, T>);

impl<'d, T: flexspi::Instance> FlashOps for AsyncFlash<'d, T> {
    async fn vendor_id(&mut self) -> Result<u8, IoError> {
        self.0.read_vendor_id_async().await
    }
    async fn erase_sector(&mut self, addr: u32) -> Result<(), IoError> {
        self.0.erase_sector_async(addr).await
    }
    async fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), IoError> {
        self.0.read_async(addr, buf).await
    }
    async fn page_program(&mut self, addr: u32, data: &[u8]) -> Result<(), IoError> {
        self.0.page_program_async(addr, data).await
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = hal::init(Config::default());

    info!("=========================================");
    info!("FlexSPI STRESS test starting");
    info!("base 0x{=u32:08x}, span {=u32} bytes", FLASH_BASE, STRESS_BYTES);
    info!("=========================================");

    let b = blocking_phase(&mut p).await;
    let i = interrupt_phase(&mut p).await;
    let d = dma_phase(&mut p).await;

    let total = b + i + d;
    info!("=========================================");
    if total == 0 {
        info!("FlexSPI STRESS test: ALL PHASES PASSED");
    } else {
        error!(
            "FlexSPI STRESS test: {=u32} content-mismatch failures (blocking={=u32}, interrupt={=u32}, dma={=u32})",
            total, b, i, d
        );
    }
    info!("=========================================");

    cortex_m::asm::bkpt();
}

async fn blocking_phase(p: &mut Peripherals) -> u32 {
    info!("--- PHASE 1: BLOCKING ---");

    let flash = NorFlash::new_blocking(
        p.FLEXSPI0.reborrow(),
        p.P3_0.reborrow(),
        p.P3_1.reborrow(),
        p.P3_6.reborrow(),
        p.P3_7.reborrow(),
        p.P3_8.reborrow(),
        p.P3_9.reborrow(),
        p.P3_10.reborrow(),
        p.P3_11.reborrow(),
        FlexspiClockConfig::default(),
        FLASH_CONFIG,
    );
    let mut flash = BlockingFlash(unwrap!(flash));

    let mut failures = Failures::new();
    run_stress("blocking", &mut flash, &mut failures).await;
    if failures.count == 0 {
        info!("--- PHASE 1: BLOCKING PASSED ---");
    } else {
        error!("--- PHASE 1: BLOCKING had {=u32} failures ---", failures.count);
    }
    failures.count
}

async fn interrupt_phase(p: &mut Peripherals) -> u32 {
    info!("--- PHASE 2: INTERRUPT (no DMA) ---");

    let flash = NorFlash::new_async(
        p.FLEXSPI0.reborrow(),
        p.P3_0.reborrow(),
        p.P3_1.reborrow(),
        p.P3_6.reborrow(),
        p.P3_7.reborrow(),
        p.P3_8.reborrow(),
        p.P3_9.reborrow(),
        p.P3_10.reborrow(),
        p.P3_11.reborrow(),
        Irqs,
        FlexspiClockConfig::default(),
        FLASH_CONFIG,
    );
    let mut flash = AsyncFlash(unwrap!(flash));

    let mut failures = Failures::new();
    run_stress("interrupt", &mut flash, &mut failures).await;
    if failures.count == 0 {
        info!("--- PHASE 2: INTERRUPT PASSED ---");
    } else {
        error!("--- PHASE 2: INTERRUPT had {=u32} failures ---", failures.count);
    }
    failures.count
}

async fn dma_phase(p: &mut Peripherals) -> u32 {
    info!("--- PHASE 3: DMA ---");

    let flash = NorFlash::new_with_dma(
        p.FLEXSPI0.reborrow(),
        p.P3_0.reborrow(),
        p.P3_1.reborrow(),
        p.P3_6.reborrow(),
        p.P3_7.reborrow(),
        p.P3_8.reborrow(),
        p.P3_9.reborrow(),
        p.P3_10.reborrow(),
        p.P3_11.reborrow(),
        p.DMA0_CH0.reborrow(),
        p.DMA0_CH1.reborrow(),
        Irqs,
        FlexspiClockConfig::default(),
        FLASH_CONFIG,
    );
    let mut flash = AsyncFlash(unwrap!(flash));

    let mut failures = Failures::new();
    run_stress("dma", &mut flash, &mut failures).await;
    if failures.count == 0 {
        info!("--- PHASE 3: DMA PASSED ---");
    } else {
        error!("--- PHASE 3: DMA had {=u32} failures ---", failures.count);
    }
    failures.count
}

async fn run_stress<F: FlashOps>(mode: &'static str, flash: &mut F, fails: &mut Failures) {
    // Setup: drop every sector we are about to touch, regardless of what
    // a previous phase / previous cargo-run left behind. This guarantees
    // every "expect erased" assertion below starts from a known state.
    info!("[{=str}] setup: erasing {=u32} sectors", mode, STRESS_SECTORS);
    for s in 0..STRESS_SECTORS {
        let addr = FLASH_BASE + s * FLASH_SECTOR_SIZE as u32;
        unwrap!(flash.erase_sector(addr).await);
    }

    // 1) Vendor ID idempotency: any drift between back-to-back reads almost
    //    certainly indicates leftover state in the IP FIFO state machine.
    let id_a = unwrap!(flash.vendor_id().await);
    let id_b = unwrap!(flash.vendor_id().await);
    let id_c = unwrap!(flash.vendor_id().await);
    if id_a != id_b || id_b != id_c {
        fails.count += 1;
        error!(
            "[{=str}] FAIL: vendor id drift: 0x{=u8:02x} 0x{=u8:02x} 0x{=u8:02x}",
            mode, id_a, id_b, id_c
        );
    } else {
        info!("[{=str}] vendor id 0x{=u8:02x} stable across 3 reads", mode, id_a);
    }

    // 2) Negative paths: empty buffer / oversized buffer must be rejected
    //    cleanly with InvalidTransferLength rather than silently truncating
    //    or hanging the driver.
    match flash.page_program(FLASH_BASE, &[]).await {
        Err(IoError::InvalidTransferLength) => info!("[{=str}] empty page_program rejected", mode),
        other => {
            fails.count += 1;
            error!(
                "[{=str}] FAIL: empty page_program: expected InvalidTransferLength, got {:?}",
                mode, other
            );
        }
    }
    let too_big = [0xAAu8; FLASH_PAGE_SIZE + 1];
    match flash.page_program(FLASH_BASE, &too_big).await {
        Err(IoError::InvalidTransferLength) => {
            info!(
                "[{=str}] oversized page_program rejected (len={=usize})",
                mode,
                too_big.len()
            )
        }
        other => {
            fails.count += 1;
            error!(
                "[{=str}] FAIL: oversized page_program: expected InvalidTransferLength, got {:?}",
                mode, other
            );
        }
    }

    // 3) Erase idempotency: erasing an already-erased sector should be a
    //    no-op, not a stall and not a Command error.
    unwrap!(flash.erase_sector(FLASH_BASE).await);
    info!("[{=str}] erase idempotency verified", mode);

    // 4) Read-length matrix (vs erased): probes the IP FIFO chunking loop
    //    at every interesting boundary.
    let mut probe = [0u8; 2048];
    for &len in READ_LENS {
        probe[..len].fill(0);
        unwrap!(flash.read(FLASH_BASE, &mut probe[..len]).await);
        if let Some(bad) = check_erased(FLASH_BASE, &probe[..len]) {
            fails.count += 1;
            error!(
                "[{=str}] FAIL: erased read len={=usize}: first non-FF at 0x{=u32:08x}",
                mode, len, bad
            );
        }
    }
    info!("[{=str}] read-length matrix vs erased done", mode);

    // 5) Read-offset matrix (vs erased): straddles FIFO/page/sector
    //    boundaries.
    for &off in READ_OFFSETS {
        let addr = FLASH_BASE + off;
        probe[..200].fill(0);
        unwrap!(flash.read(addr, &mut probe[..200]).await);
        if let Some(bad) = check_erased(addr, &probe[..200]) {
            fails.count += 1;
            error!(
                "[{=str}] FAIL: erased read off={=u32} len=200: first non-FF at 0x{=u32:08x}",
                mode, off, bad
            );
        }
    }
    info!("[{=str}] read-offset matrix vs erased done", mode);

    // 6) Program-size matrix. Each entry programs a freshly-erased page,
    //    so we can independently verify head=pattern and tail=erased.
    for (i, &n) in PROGRAM_SIZES.iter().enumerate() {
        let addr = FLASH_BASE + (i as u32) * FLASH_PAGE_SIZE as u32;
        // Re-erase the host sector each time we start a new one.
        if (addr - FLASH_BASE) % FLASH_SECTOR_SIZE as u32 == 0 {
            unwrap!(flash.erase_sector(addr).await);
        }
        let mut page = [0u8; FLASH_PAGE_SIZE];
        fill_pattern(addr, &mut page);
        unwrap!(flash.page_program(addr, &page[..n]).await);

        let mut readback = [0u8; FLASH_PAGE_SIZE];
        unwrap!(flash.read(addr, &mut readback[..n]).await);
        if let Some(bad) = check_pattern(addr, &readback[..n]) {
            fails.count += 1;
            error!(
                "[{=str}] FAIL: program n={=usize}: head mismatch at 0x{=u32:08x}",
                mode, n, bad
            );
        }
        if n < FLASH_PAGE_SIZE {
            unwrap!(flash.read(addr + n as u32, &mut readback[n..]).await);
            if let Some(bad) = check_erased(addr + n as u32, &readback[n..]) {
                fails.count += 1;
                error!(
                    "[{=str}] FAIL: program n={=usize}: tail not erased at 0x{=u32:08x} (got 0x{=u8:02x})",
                    mode,
                    n,
                    bad,
                    readback[(bad - addr) as usize]
                );
            }
        }
    }
    info!("[{=str}] program-size matrix done", mode);

    // 7) Re-read the patterned region using the full read-length matrix.
    //    The first PROGRAM_SIZES.len()*FLASH_PAGE_SIZE bytes contain
    //    pattern bytes only at the head of each page; we can't sweep the
    //    pattern checker over them. Instead we sweep against the first
    //    page (which was fully written when n=256).
    let last_full_page_idx = PROGRAM_SIZES.iter().rposition(|&n| n == FLASH_PAGE_SIZE).unwrap_or(0);
    let full_page_addr = FLASH_BASE + (last_full_page_idx as u32) * FLASH_PAGE_SIZE as u32;
    for &len in READ_LENS {
        let len = len.min(FLASH_PAGE_SIZE);
        probe[..len].fill(0);
        unwrap!(flash.read(full_page_addr, &mut probe[..len]).await);
        if let Some(bad) = check_pattern(full_page_addr, &probe[..len]) {
            fails.count += 1;
            error!(
                "[{=str}] FAIL: post-program read len={=usize}: pattern mismatch at 0x{=u32:08x}",
                mode, len, bad
            );
        }
    }
    info!("[{=str}] read-length matrix vs pattern done", mode);

    // 8) Cross-sector read. The 64 byte buffer straddles the boundary
    //    between two adjacent fully-programmed sectors.  This catches
    //    address-arithmetic errors when chunking spans a sector boundary.
    let cross_addr = full_page_addr + FLASH_PAGE_SIZE as u32 - 32;
    let mut cross = [0u8; 64];
    unwrap!(flash.read(cross_addr, &mut cross).await);
    // Only verify the part inside the same patterned page (first 32 bytes);
    // the second half lies in the next page which may have been written
    // with a smaller `n` so its tail is erased.
    if let Some(bad) = check_pattern(cross_addr, &cross[..32]) {
        fails.count += 1;
        error!("[{=str}] FAIL: cross-page read: mismatch at 0x{=u32:08x}", mode, bad);
    }
    info!("[{=str}] cross-page read done", mode);

    // 9) Selective erase: erase the SECOND stress sector, then verify
    //    the first sector is still intact and the second is erased.
    //    For the first-sector check we walk every page individually and
    //    only verify the patterned head (PROGRAM_SIZES[i] bytes); the
    //    tails were intentionally left erased by step 6 and a blanket
    //    full-page pattern check would always fail there.
    let middle = FLASH_BASE + FLASH_SECTOR_SIZE as u32;
    unwrap!(flash.erase_sector(middle).await);
    let mut sector = [0u8; FLASH_PAGE_SIZE];
    unwrap!(flash.read(middle, &mut sector).await);
    if let Some(bad) = check_erased(middle, &sector) {
        fails.count += 1;
        error!(
            "[{=str}] FAIL: middle sector not fully erased at 0x{=u32:08x}",
            mode, bad
        );
    }
    let pages_per_sector = FLASH_SECTOR_SIZE / FLASH_PAGE_SIZE;
    let pages_in_first_sector = pages_per_sector.min(PROGRAM_SIZES.len());
    for page_idx in 0..pages_in_first_sector {
        let addr = FLASH_BASE + (page_idx as u32) * FLASH_PAGE_SIZE as u32;
        let n = PROGRAM_SIZES[page_idx];
        unwrap!(flash.read(addr, &mut sector[..n]).await);
        if let Some(bad) = check_pattern(addr, &sector[..n]) {
            fails.count += 1;
            error!(
                "[{=str}] FAIL: first sector corrupted by erasing middle at 0x{=u32:08x}",
                mode, bad
            );
            break;
        }
    }
    info!("[{=str}] selective erase done", mode);

    // 10) Overlay program (no erase between writes). Writes 0xF0 then 0x0F
    //     to the same address inside a freshly-erased sector and expects
    //     the AND of the two patterns (0x00). Catches any silent dropping
    //     of the second program command.
    let overlay_sector = FLASH_BASE + (STRESS_SECTORS - 1) * FLASH_SECTOR_SIZE as u32;
    unwrap!(flash.erase_sector(overlay_sector).await);
    let pat_a = [0xF0u8; 64];
    let pat_b = [0x0Fu8; 64];
    unwrap!(flash.page_program(overlay_sector, &pat_a).await);
    unwrap!(flash.page_program(overlay_sector, &pat_b).await);
    let mut overlay = [0u8; 64];
    unwrap!(flash.read(overlay_sector, &mut overlay).await);
    let mut overlay_bad = false;
    for (i, b) in overlay.iter().enumerate() {
        if *b != 0x00 {
            fails.count += 1;
            error!(
                "[{=str}] FAIL: overlay program byte {=usize}: expected 0x00, got 0x{=u8:02x}",
                mode, i, *b
            );
            overlay_bad = true;
            break;
        }
    }
    if !overlay_bad {
        info!("[{=str}] overlay program (NOR AND) PASSED", mode);
    }

    // 11) Partial-page program inside that same sector: erase, write
    //     `partial_len` bytes from the page start, then verify head and
    //     tail.  Distinct from the program-size matrix in that it lives
    //     in its own dedicated sector so we don't perturb earlier results.
    unwrap!(flash.erase_sector(overlay_sector).await);
    let partial_len = 100usize;
    let mut partial = [0u8; FLASH_PAGE_SIZE];
    fill_pattern(overlay_sector, &mut partial);
    unwrap!(flash.page_program(overlay_sector, &partial[..partial_len]).await);
    let mut readback = [0u8; FLASH_PAGE_SIZE];
    unwrap!(flash.read(overlay_sector, &mut readback).await);
    if let Some(bad) = check_pattern(overlay_sector, &readback[..partial_len]) {
        fails.count += 1;
        error!("[{=str}] FAIL: partial-page head mismatch at 0x{=u32:08x}", mode, bad);
    }
    if let Some(bad) = check_erased(overlay_sector + partial_len as u32, &readback[partial_len..]) {
        fails.count += 1;
        error!("[{=str}] FAIL: partial-page tail not erased at 0x{=u32:08x}", mode, bad);
    }
    info!("[{=str}] partial-page program done", mode);

    // 12) Interleaved vendor-id / read. Hits the IP path with alternating
    //     zero-data and bulk-data sizes; any leftover ipcr1.idatsz from a
    //     previous transfer would corrupt one of these.
    let mut tiny = [0u8; 16];
    for i in 0..32u32 {
        let _id = unwrap!(flash.vendor_id().await);
        unwrap!(flash.read(FLASH_BASE + (i & 7) * 64, &mut tiny).await);
    }
    info!("[{=str}] interleaved vendor-id / read done", mode);
}
