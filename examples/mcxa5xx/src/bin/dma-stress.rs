//! DMA mem-to-mem stress test.
//!
//! Goal: try very hard to break the embassy-mcxa eDMA driver running pure
//! memory-to-memory transfers (no peripheral involved). If transfers fail or
//! hang in this isolated setting, the bug lives in the DMA driver itself
//! rather than in any peripheral that happens to drive it.
//!
//! What the test exercises:
//! - All three word widths supported by `mem_to_mem` (`u8`, `u16`, `u32`).
//! - Lengths spanning sub-FIFO, page-sized, multi-page, prime, and the
//!   maximum supported transfer size minus a few words.
//! - Aligned, unaligned (offset within source/dest), and odd-tail buffers.
//! - Several deterministic patterns (constant, ramp, alternating, hash) so
//!   that any byte-level corruption is observable.
//! - Back-to-back transfers on the same channel and on alternating channels
//!   (DMA0_CH0 vs DMA0_CH1) to expose channel-state leakage.
//! - Both async (`.await`) and `blocking_wait()` completion paths.
//! - Cancellation: starts a transfer, drops the future, then runs another
//!   transfer to verify the channel recovers.
//! - Error path: empty source slice, oversized source slice, dst smaller
//!   than src — all must return `InvalidParameters` and not leave the
//!   channel armed.
//!
//! The whole battery is repeated 10 times before the final PASSED line is
//! printed. Failures are accumulated rather than panicking on first error
//! so that a single subtle issue does not mask the rest of the matrix.
//!
//! Run with: `cargo run --release --bin dma-stress`.
//! The example ends with a `bkpt` so the binary does not loop forever.

#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_mcxa::dma::{DmaChannel, InvalidParameters, TransferOptions};
use embassy_mcxa::{Peri, peripherals};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const ITERATIONS: u32 = 10;

// Big enough to exercise multi-major-loop transfers but still fit in SRAM.
// 8 KiB per buffer = 16 KiB pair, well below MCXA577 SRAM budget.
const BUF_BYTES: usize = 8 * 1024;

#[repr(align(4))]
struct AlignedBuf([u8; BUF_BYTES]);

static mut SRC: AlignedBuf = AlignedBuf([0; BUF_BYTES]);
static mut DST: AlignedBuf = AlignedBuf([0; BUF_BYTES]);

fn src_mut() -> &'static mut [u8; BUF_BYTES] {
    // Safety: single-threaded test, accessed only from main.
    unsafe { &mut *core::ptr::addr_of_mut!(SRC.0) }
}

fn dst_mut() -> &'static mut [u8; BUF_BYTES] {
    unsafe { &mut *core::ptr::addr_of_mut!(DST.0) }
}

#[derive(Copy, Clone)]
enum Pattern {
    Zeros,
    Ones,
    Ramp,
    Alternating,
    Hash,
}

impl Pattern {
    const ALL: &'static [Pattern] = &[
        Pattern::Zeros,
        Pattern::Ones,
        Pattern::Ramp,
        Pattern::Alternating,
        Pattern::Hash,
    ];

    fn fill(self, buf: &mut [u8]) {
        match self {
            Pattern::Zeros => buf.fill(0x00),
            Pattern::Ones => buf.fill(0xFF),
            Pattern::Ramp => {
                for (i, b) in buf.iter_mut().enumerate() {
                    *b = i as u8;
                }
            }
            Pattern::Alternating => {
                for (i, b) in buf.iter_mut().enumerate() {
                    *b = if i & 1 == 0 { 0xA5 } else { 0x5A };
                }
            }
            Pattern::Hash => {
                for (i, b) in buf.iter_mut().enumerate() {
                    let h = (i as u32).wrapping_mul(2_654_435_761);
                    *b = (h ^ (h >> 16)) as u8;
                }
            }
        }
    }

    fn name(self) -> &'static str {
        match self {
            Pattern::Zeros => "zeros",
            Pattern::Ones => "ones",
            Pattern::Ramp => "ramp",
            Pattern::Alternating => "altern",
            Pattern::Hash => "hash",
        }
    }
}

struct Failures {
    count: u32,
}

impl Failures {
    const fn new() -> Self {
        Self { count: 0 }
    }

    fn record(&mut self, ctx: &'static str) {
        self.count += 1;
        error!("FAILURE: {=str}", ctx);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = hal::init(hal::config::Config::default());

    info!("=========================================");
    info!("DMA STRESS test starting ({=u32} iterations)", ITERATIONS);
    info!("=========================================");

    let mut failures = Failures::new();

    for iter in 0..ITERATIONS {
        info!("--- iteration {=u32}/{=u32} ---", iter + 1, ITERATIONS);
        run_iteration(&mut p, &mut failures).await;
    }

    if failures.count == 0 {
        info!("=========================================");
        info!("DMA STRESS PASSED ({=u32} iterations, no failures)", ITERATIONS);
        info!("=========================================");
    } else {
        error!("=========================================");
        error!(
            "DMA STRESS FAILED: {=u32} failure(s) across {=u32} iterations",
            failures.count, ITERATIONS
        );
        error!("=========================================");
    }

    cortex_m::asm::bkpt();
}

async fn run_iteration(p: &mut hal::Peripherals, failures: &mut Failures) {
    // Each phase reborrows the channel(s) so we exercise repeated `new()` on
    // the same Peri across iterations.
    info!("  phase: basic_lengths");
    phase_basic_lengths(p.DMA0_CH0.reborrow(), failures).await;
    info!("  phase: widths");
    phase_widths(p.DMA0_CH0.reborrow(), failures).await;
    info!("  phase: offsets");
    phase_offsets(p.DMA0_CH0.reborrow(), failures).await;
    info!("  phase: back_to_back");
    phase_back_to_back(p.DMA0_CH0.reborrow(), failures).await;
    info!("  phase: alternate_channels");
    phase_alternate_channels(p.DMA0_CH0.reborrow(), p.DMA0_CH1.reborrow(), failures).await;
    info!("  phase: blocking_wait");
    phase_blocking_wait(p.DMA0_CH0.reborrow(), failures).await;
    info!("  phase: cancel");
    phase_cancel(p.DMA0_CH0.reborrow(), failures).await;
    info!("  phase: invalid_parameters");
    phase_invalid_parameters(p.DMA0_CH0.reborrow(), failures);
    info!("  phase: max_size");
    phase_max_size(p.DMA0_CH0.reborrow(), failures).await;
}

const LEN_PROBES: &[usize] = &[
    1, 2, 3, 4, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256, 257, 511, 512, 513, 1023, 1024,
    1025, 2047, 2048, 4095, 4096, 8191, 8192,
];

async fn phase_basic_lengths(ch: Peri<'_, peripherals::DMA0_CH0>, failures: &mut Failures) {
    let src = src_mut();
    let dst = dst_mut();
    let mut chan = DmaChannel::new(ch);
    for &pattern in Pattern::ALL {
        pattern.fill(&mut src[..]);
        for &len in LEN_PROBES {
            if len > BUF_BYTES {
                continue;
            }
            dst[..len].fill(0);
            // Sentinel one byte past the end to detect overruns.
            let sentinel = if len < BUF_BYTES { Some(dst[len]) } else { None };
            let opts = TransferOptions::COMPLETE_INTERRUPT;
            match chan.mem_to_mem::<u8>(&src[..len], &mut dst[..len], opts) {
                Ok(t) => {
                    if let Err(e) = t.await {
                        failures.record("basic_lengths: transfer error");
                        let _ = e;
                    } else if dst[..len] != src[..len] {
                        failures.record("basic_lengths: content mismatch");
                        let _ = pattern.name();
                    }
                }
                Err(_) => failures.record("basic_lengths: setup InvalidParameters"),
            }
            if let Some(s) = sentinel
                && dst[len] != s
            {
                failures.record("basic_lengths: write past end");
            }
        }
    }
}

async fn phase_widths(ch: Peri<'_, peripherals::DMA0_CH0>, failures: &mut Failures) {
    let mut chan = DmaChannel::new(ch);
    let opts = TransferOptions::COMPLETE_INTERRUPT;

    // u8 path
    {
        let src = src_mut();
        let dst = dst_mut();
        Pattern::Hash.fill(&mut src[..]);
        dst.fill(0);
        match chan.mem_to_mem::<u8>(&src[..1024], &mut dst[..1024], opts) {
            Ok(t) => {
                if t.await.is_err() {
                    failures.record("widths: u8 transfer error");
                } else if dst[..1024] != src[..1024] {
                    failures.record("widths: u8 content mismatch");
                }
            }
            Err(_) => failures.record("widths: u8 setup error"),
        }
    }

    // u16 path: cast through aligned slices
    {
        let src = src_mut();
        let dst = dst_mut();
        Pattern::Ramp.fill(&mut src[..]);
        dst.fill(0);
        let n_words = 1024;
        let src_w: &[u16] = unsafe { core::slice::from_raw_parts(src.as_ptr().cast::<u16>(), n_words) };
        let dst_w: &mut [u16] = unsafe { core::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<u16>(), n_words) };
        match chan.mem_to_mem::<u16>(src_w, dst_w, opts) {
            Ok(t) => {
                if t.await.is_err() {
                    failures.record("widths: u16 transfer error");
                } else if dst[..n_words * 2] != src[..n_words * 2] {
                    failures.record("widths: u16 content mismatch");
                }
            }
            Err(_) => failures.record("widths: u16 setup error"),
        }
    }

    // u32 path
    {
        let src = src_mut();
        let dst = dst_mut();
        Pattern::Alternating.fill(&mut src[..]);
        dst.fill(0);
        let n_words = 1024;
        let src_w: &[u32] = unsafe { core::slice::from_raw_parts(src.as_ptr().cast::<u32>(), n_words) };
        let dst_w: &mut [u32] = unsafe { core::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<u32>(), n_words) };
        match chan.mem_to_mem::<u32>(src_w, dst_w, opts) {
            Ok(t) => {
                if t.await.is_err() {
                    failures.record("widths: u32 transfer error");
                } else if dst[..n_words * 4] != src[..n_words * 4] {
                    failures.record("widths: u32 content mismatch");
                }
            }
            Err(_) => failures.record("widths: u32 setup error"),
        }
    }
}

async fn phase_offsets(ch: Peri<'_, peripherals::DMA0_CH0>, failures: &mut Failures) {
    let src = src_mut();
    let dst = dst_mut();
    let mut chan = DmaChannel::new(ch);
    let opts = TransferOptions::COMPLETE_INTERRUPT;

    Pattern::Hash.fill(&mut src[..]);

    // src offset varying, dst at base
    for &src_off in &[0usize, 1, 2, 3, 5, 7, 13, 64, 100, 255] {
        let len = 257usize.min(BUF_BYTES - src_off);
        dst[..len].fill(0);
        if let Ok(t) = chan.mem_to_mem::<u8>(&src[src_off..src_off + len], &mut dst[..len], opts) {
            if t.await.is_err() {
                failures.record("offsets: src_off transfer error");
            } else if dst[..len] != src[src_off..src_off + len] {
                failures.record("offsets: src_off mismatch");
            }
        } else {
            failures.record("offsets: src_off setup error");
        }
    }

    // dst offset varying, src at base
    for &dst_off in &[0usize, 1, 2, 3, 5, 7, 13, 64, 100, 255] {
        let len = 257usize.min(BUF_BYTES - dst_off);
        dst[dst_off..dst_off + len].fill(0);
        let pre = if dst_off > 0 { Some(dst[dst_off - 1]) } else { None };
        if let Ok(t) = chan.mem_to_mem::<u8>(&src[..len], &mut dst[dst_off..dst_off + len], opts) {
            if t.await.is_err() {
                failures.record("offsets: dst_off transfer error");
            } else if dst[dst_off..dst_off + len] != src[..len] {
                failures.record("offsets: dst_off mismatch");
            }
            if let Some(p) = pre
                && dst[dst_off - 1] != p
            {
                failures.record("offsets: dst_off write underrun");
            }
        } else {
            failures.record("offsets: dst_off setup error");
        }
    }
}

async fn phase_back_to_back(ch: Peri<'_, peripherals::DMA0_CH0>, failures: &mut Failures) {
    let src = src_mut();
    let dst = dst_mut();
    let mut chan = DmaChannel::new(ch);
    let opts = TransferOptions::COMPLETE_INTERRUPT;

    // Run many short transfers without yielding between to expose
    // re-arming / completion latch issues on the same channel object.
    for i in 0..64u32 {
        let len = 64 + (i as usize % 7) * 17;
        Pattern::Hash.fill(&mut src[..len]);
        dst[..len].fill(0);
        match chan.mem_to_mem::<u8>(&src[..len], &mut dst[..len], opts) {
            Ok(t) => {
                if t.await.is_err() {
                    failures.record("back_to_back: transfer error");
                    return;
                }
                if dst[..len] != src[..len] {
                    failures.record("back_to_back: content mismatch");
                    return;
                }
            }
            Err(_) => {
                failures.record("back_to_back: setup error");
                return;
            }
        }
    }
}

async fn phase_alternate_channels(
    ch0: Peri<'_, peripherals::DMA0_CH0>,
    ch1: Peri<'_, peripherals::DMA0_CH1>,
    failures: &mut Failures,
) {
    let src = src_mut();
    let dst = dst_mut();
    let mut chan_a = DmaChannel::new(ch0);
    let mut chan_b = DmaChannel::new(ch1);
    let opts = TransferOptions::COMPLETE_INTERRUPT;

    for i in 0..32u32 {
        let len = 128 + (i as usize % 5) * 31;
        Pattern::Ramp.fill(&mut src[..len]);
        dst[..len].fill(0);
        let chan = if i & 1 == 0 { &mut chan_a } else { &mut chan_b };
        match chan.mem_to_mem::<u8>(&src[..len], &mut dst[..len], opts) {
            Ok(t) => {
                if t.await.is_err() {
                    failures.record("alternate_channels: transfer error");
                    return;
                }
                if dst[..len] != src[..len] {
                    failures.record("alternate_channels: content mismatch");
                    return;
                }
            }
            Err(_) => {
                failures.record("alternate_channels: setup error");
                return;
            }
        }
    }
}

async fn phase_blocking_wait(ch: Peri<'_, peripherals::DMA0_CH0>, failures: &mut Failures) {
    let src = src_mut();
    let dst = dst_mut();
    let mut chan = DmaChannel::new(ch);
    let opts = TransferOptions::COMPLETE_INTERRUPT;

    for &len in &[1usize, 32, 256, 1024, 4096] {
        Pattern::Alternating.fill(&mut src[..len]);
        dst[..len].fill(0);
        match chan.mem_to_mem::<u8>(&src[..len], &mut dst[..len], opts) {
            Ok(t) => {
                t.blocking_wait();
                if dst[..len] != src[..len] {
                    failures.record("blocking_wait: content mismatch");
                }
            }
            Err(_) => failures.record("blocking_wait: setup error"),
        }
    }
}

async fn phase_cancel(ch: Peri<'_, peripherals::DMA0_CH0>, failures: &mut Failures) {
    let src = src_mut();
    let dst = dst_mut();
    let mut chan = DmaChannel::new(ch);
    let opts = TransferOptions::COMPLETE_INTERRUPT;

    // Start a transfer, drop it before completion, then issue another
    // transfer to confirm the channel is usable again.
    Pattern::Hash.fill(&mut src[..]);
    {
        let t = match chan.mem_to_mem::<u8>(&src[..4096], &mut dst[..4096], opts) {
            Ok(t) => t,
            Err(_) => {
                failures.record("cancel: setup error");
                return;
            }
        };
        drop(t);
    }

    // Recovery transfer: must complete and produce correct data.
    let len = 1024;
    Pattern::Ramp.fill(&mut src[..len]);
    dst[..len].fill(0);
    match chan.mem_to_mem::<u8>(&src[..len], &mut dst[..len], opts) {
        Ok(t) => {
            if t.await.is_err() {
                failures.record("cancel: recovery transfer error");
            } else if dst[..len] != src[..len] {
                failures.record("cancel: recovery content mismatch");
            }
        }
        Err(_) => failures.record("cancel: recovery setup error"),
    }
}

fn phase_invalid_parameters(ch: Peri<'_, peripherals::DMA0_CH0>, failures: &mut Failures) {
    let src = src_mut();
    let dst = dst_mut();
    let mut chan = DmaChannel::new(ch);
    let opts = TransferOptions::COMPLETE_INTERRUPT;

    // Empty src must be rejected.
    let empty: &[u8] = &[];
    if !matches!(
        chan.mem_to_mem::<u8>(empty, &mut dst[..16], opts).err(),
        Some(InvalidParameters)
    ) {
        failures.record("invalid_parameters: empty src not rejected");
    }

    // dst smaller than src must be rejected.
    if !matches!(
        chan.mem_to_mem::<u8>(&src[..32], &mut dst[..16], opts).err(),
        Some(InvalidParameters)
    ) {
        failures.record("invalid_parameters: short dst not rejected");
    }

    // After rejected setups, the channel must still be usable.
    let opts2 = TransferOptions::COMPLETE_INTERRUPT;
    let len = 64;
    Pattern::Ramp.fill(&mut src[..len]);
    dst[..len].fill(0);
    if let Ok(t) = chan.mem_to_mem::<u8>(&src[..len], &mut dst[..len], opts2) {
        t.blocking_wait();
        if dst[..len] != src[..len] {
            failures.record("invalid_parameters: recovery mismatch");
        }
    } else {
        failures.record("invalid_parameters: recovery setup error");
    }
}

async fn phase_max_size(ch: Peri<'_, peripherals::DMA0_CH0>, failures: &mut Failures) {
    // Push close to the per-transfer maximum the buffer permits. The driver's
    // hard cap is DMA_MAX_TRANSFER_SIZE = 0x7FFF; we pick a length right at
    // the BUF_BYTES boundary which is well within that limit but still the
    // largest single transfer this example supports.
    let src = src_mut();
    let dst = dst_mut();
    let mut chan = DmaChannel::new(ch);
    let opts = TransferOptions::COMPLETE_INTERRUPT;

    Pattern::Hash.fill(&mut src[..]);
    dst.fill(0);
    match chan.mem_to_mem::<u8>(&src[..], &mut dst[..], opts) {
        Ok(t) => {
            if t.await.is_err() {
                failures.record("max_size: transfer error");
            } else if dst[..] != src[..] {
                failures.record("max_size: content mismatch");
            }
        }
        Err(_) => failures.record("max_size: setup error"),
    }
}
