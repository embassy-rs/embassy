//! Continuous DMA receive ring built from two chained channels.
//!
//! This is the transport-agnostic half of the ring-buffered RX drivers. It receives
//! continuously into a circular pool of DMA buffers, with no per-byte interrupt and no CPU
//! involvement between buffers.
//!
//! # How it works
//!
//! Two DMA channels are wired into a self-perpetuating loop:
//!
//! - The **data** channel copies a fixed source register into one segment of the pool.
//! - The **control** channel walks a small table of segment pointers and writes the next one
//!   into the data channel's `WRITE_ADDR`, then chains back to re-trigger it.
//!
//! `TRANS_COUNT` is a reload register on RP2040/RP2350 — the written value is copied into the
//! live counter on every trigger — so neither channel ever needs re-arming. The control
//! channel's read address wraps around the pointer table via `RING_SIZE`/`RING_SEL`, so the
//! loop runs forever without the CPU.
//!
//! The data channel raises its completion interrupt (`irq_quiet = false`) once per segment,
//! which is what wakes a blocked reader. Reception itself does not depend on that interrupt
//! being serviced promptly; only the wakeup latency and the overrun bookkeeping do.
//!
//! A front end supplies exactly four things: the source address, the DREQ, and (implicitly)
//! that transfers are byte-sized with a non-incrementing read address.
//!
//! # Position tracking
//!
//! The hardware has no lap counter, so the absolute write position is reconstructed from two
//! samples: the channel's live `WRITE_ADDR`, which gives the exact position within the pool, and
//! the per-channel completion count maintained by [`dma::InterruptHandler`], which can show that
//! the DMA lapped since the previous sample.
//!
//! A DMA `INTS` bit is a single latched flag, so several segment completions occurring before
//! the handler runs are counted once, and that shortfall is never made up. The count is
//! therefore only a lower bound. `Pos::apply` takes the furthest of it and the previous sample,
//! then moves forward to the position `WRITE_ADDR` shows. That is exact as long as the DMA
//! writes less than a whole pool between two samples, however many interrupts were merged.
//!
//! A reader that falls a whole pool or more behind may not be detected: the counter can only
//! reveal the extra lap if it is less than a lap short. The reader then gets stale bytes with no
//! error. Anything less than a pool behind but within one segment of it is reported as an
//! overrun.

use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};
use core::task::Waker;

use crate::dma::Channel;
use crate::mode::{Async, Blocking};
use crate::{dma, pac};

/// Number of DMA segments the pool is split into.
///
/// The pointer table is this many words and is wrapped by the DMA `RING_SIZE` field, so this
/// must stay a power of two.
pub(crate) const NBUFS: usize = 4;

const _: () = core::assert!(NBUFS.is_power_of_two());

/// `log2(size_of::<[u32; NBUFS]>())`, for `CTRL.RING_SIZE`.
const RING_SIZE: u8 = (NBUFS * 4).trailing_zeros() as u8;

/// Segment pointer table read by the control DMA channel.
///
/// Aligned to its own size so the DMA read-address ring wrap lands back on entry 0.
#[repr(C, align(16))]
struct PtrTable([AtomicU32; NBUFS]);

const _: () = core::assert!(size_of::<PtrTable>() == 1 << RING_SIZE);

/// Storage for the DMA-visible segment pointer table of one receive ring.
///
/// This must outlive the ring, because the DMA hardware reads the table directly while the
/// ring runs and the ring struct itself is movable.
pub struct State {
    table: PtrTable,
}

impl State {
    /// Create a new, empty state.
    pub const fn new() -> Self {
        Self {
            table: PtrTable([const { AtomicU32::new(0) }; NBUFS]),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// Reconstructed stream position of a receive ring.
///
/// Split out from [`RxRing`] so the arithmetic — which is where the subtle bugs live — can be
/// exercised on the host without any DMA hardware. See the tests at the bottom of this file.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Pos {
    /// Total pool length in bytes.
    len: usize,
    /// One segment, `len / NBUFS`.
    seg_len: usize,
    /// Value of the channel's completion counter when the ring started.
    completions_base: u32,
    /// Reader position within the pool, in `0..len`.
    read_pos: usize,
    /// Monotonic count of bytes the DMA has written.
    written: u64,
    /// `written` as an offset into the pool, kept alongside so the hot path needs no 64-bit
    /// division.
    write_pos: usize,
    /// Monotonic count of bytes handed to the caller.
    read: u64,
}

impl Pos {
    fn new(len: usize, completions_base: u32) -> Self {
        Self {
            len,
            seg_len: len / NBUFS,
            completions_base,
            read_pos: 0,
            written: 0,
            write_pos: 0,
            read: 0,
        }
    }

    /// Fold a `(completion count, write position)` sample into the monotonic counters.
    ///
    /// `completions` is the raw per-channel counter; `write_pos` is the DMA write address
    /// relative to the pool base, in `0..len`.
    pub(crate) fn apply(&mut self, completions: u32, write_pos: usize) {
        let counted = completions.wrapping_sub(self.completions_base) as u64;

        // Two lower bounds on the bytes written: what the previous sample saw, and the counted
        // segments. The counter only falls behind, never runs ahead: completions merged into one
        // interrupt are counted once, and that shortfall is never made up. So it cannot be
        // anchored to directly, only used to tell that the DMA lapped since the last sample.
        let counted_bytes = counted * self.seg_len as u64;
        let (lower, lower_pos) = if counted_bytes > self.written {
            (counted_bytes, (counted as usize % NBUFS) * self.seg_len)
        } else {
            (self.written, self.write_pos)
        };
        // The write address then gives the exact position within the lap.
        let advance = if write_pos >= lower_pos {
            write_pos - lower_pos
        } else {
            write_pos + self.len - lower_pos
        };
        self.written = lower + advance as u64;
        self.write_pos = write_pos;
    }

    /// Bytes written by the DMA but not yet handed to the caller.
    pub(crate) fn available(&self) -> usize {
        (self.written - self.read) as usize
    }

    /// Whether the writer is within one segment of lapping the reader, i.e. anything still
    /// unread is about to be, or already has been, overwritten.
    pub(crate) fn is_overrun(&self) -> bool {
        self.available() > self.len - self.seg_len
    }

    /// Drop everything buffered and restart the reader at the current write position.
    pub(crate) fn resync(&mut self) {
        self.read_pos = self.write_pos;
        self.read = self.written;
    }

    /// Account for `n` bytes copied out to the caller.
    pub(crate) fn advance_read(&mut self, n: usize) {
        self.read_pos = (self.read_pos + n) % self.len;
        self.read += n as u64;
    }
}

/// A continuously running DMA receive ring.
///
/// See the [module docs](self) for how the DMA loop is constructed.
pub(crate) struct RxRing<'d> {
    /// Data channel: source register -> pool. Held for the lifetime so the channel stays claimed.
    data: Channel<'d, Async>,
    /// Control channel: pointer table -> data channel's `WRITE_ADDR`.
    control: Channel<'d, Blocking>,

    /// The DMA pool, `NBUFS` equal contiguous segments.
    buf: &'d mut [u8],

    pos: Pos,
}

impl<'d> RxRing<'d> {
    /// Build and start a receive ring.
    ///
    /// `src` is the source register the data channel reads from; it is read without
    /// incrementing, one byte at a time, paced by `dreq`.
    ///
    /// `buf` length must be a non-zero multiple of [`NBUFS`].
    pub(crate) fn new(
        state: &'d State,
        src: *const u8,
        dreq: pac::dma::vals::TreqSel,
        data: Channel<'d, Async>,
        control: Channel<'d, Blocking>,
        buf: &'d mut [u8],
    ) -> Self {
        let len = buf.len();
        assert!(
            len > 0 && len.is_multiple_of(NBUFS),
            "rx_buffer length must be a non-zero multiple of {}",
            NBUFS
        );
        let seg_len = len / NBUFS;
        assert!(
            seg_len <= 0x0fff_ffff,
            "rx_buffer segments must fit in a DMA transfer count"
        );

        let base = buf.as_mut_ptr();

        // The data channel starts on segment 0, so the control channel must supply segment 1
        // first: table[i] is the segment that follows segment i.
        for i in 0..NBUFS {
            let next = unsafe { base.add(((i + 1) % NBUFS) * seg_len) };
            state.table.0[i].store(next as u32, Ordering::Relaxed);
        }

        let dch = pac::DMA.ch(data.number() as _);
        let cch = pac::DMA.ch(control.number() as _);

        // Control channel: one word from the pointer table into the data channel's WRITE_ADDR,
        // then chain back to the data channel.
        cch.read_addr()
            .write_value(&state.table.0[0] as *const AtomicU32 as u32);
        cch.write_addr().write_value(dch.write_addr().as_ptr() as u32);
        set_trans_count(cch, 1);
        // AL1_CTRL is not a trigger register, so this programs the channel without starting it.
        cch.al1_ctrl().write_value({
            let mut w = pac::dma::regs::Ctrl(0);
            w.set_data_size(pac::dma::vals::DataSize::SizeWord);
            w.set_incr_read(true);
            w.set_incr_write(false);
            w.set_ring_sel(false); // wrap the *read* address, i.e. the pointer table
            w.set_ring_size(RING_SIZE);
            w.set_treq_sel(pac::dma::vals::TreqSel::Permanent);
            w.set_chain_to(data.number());
            w.set_irq_quiet(true);
            w.set_en(true);
            w
        });

        // Data channel: source into segment 0, then chain to the control channel.
        dch.read_addr().write_value(src as u32);
        dch.write_addr().write_value(base as u32);
        set_trans_count(dch, seg_len as u32);
        dch.al1_ctrl().write_value({
            let mut w = pac::dma::regs::Ctrl(0);
            w.set_data_size(pac::dma::vals::DataSize::SizeByte);
            w.set_incr_read(false); // the source is a fixed register address
            w.set_incr_write(true);
            w.set_treq_sel(dreq);
            w.set_chain_to(control.number());
            w.set_irq_quiet(false); // completion IRQ is the reader's wakeup and lap counter
            w.set_en(true);
            w
        });

        // Take the counter baseline before the first transfer can complete.
        let pos = Pos::new(len, dma::channel_completions(data.number()));

        compiler_fence(Ordering::SeqCst);

        // Start the loop. CTRL_TRIG is a trigger register, so this write starts the data
        // channel; the control channel is already enabled and will be pulled in by the chain.
        dch.ctrl_trig().write_value(dch.al1_ctrl().read());

        Self {
            data,
            control,
            buf,
            pos,
        }
    }

    /// Current DMA write position within the pool, in `0..len`.
    fn write_pos(&self) -> usize {
        let base = self.buf.as_ptr() as u32;
        let addr = pac::DMA.ch(self.data.number() as _).write_addr().read();
        // At a chain boundary the write address momentarily sits one past the end of the segment
        // that just finished. Segments are contiguous, so that is already the start of the next
        // segment — except past the final segment, where it is one past the whole pool.
        (addr.wrapping_sub(base) as usize) % self.buf.len()
    }

    /// Sample the hardware and fold it into the position counters.
    fn sync(&mut self) {
        // Read the counter first, then the address. In that order a completion landing in
        // between is seen by the address only, which `Pos::apply` handles. The other order
        // would let the counter run ahead of the address and claim bytes not yet written.
        let completions = dma::channel_completions(self.data.number());
        let pos = self.write_pos();
        // The position is read from a register, but the bytes it accounts for were written to
        // plain memory by the DMA. Keep the compiler from hoisting a read of that memory above
        // the position sample that authorised it.
        compiler_fence(Ordering::Acquire);
        self.pos.apply(completions, pos);
    }

    /// Register to be woken when the data channel completes a segment.
    pub(crate) fn register_waker(&self, waker: &Waker) {
        dma::channel_waker(self.data.number()).register(waker);
    }

    /// Bytes currently available to read.
    pub(crate) fn len(&mut self) -> usize {
        self.sync();
        self.pos.available()
    }

    /// Whether any data is currently available to read.
    pub(crate) fn is_empty(&mut self) -> bool {
        self.len() == 0
    }

    /// Copy out whatever is currently available, if anything.
    ///
    /// Returns `Err(())` if the DMA has caught up with the reader, in which case the buffered
    /// data is discarded and reception continues from the current position. The caller maps
    /// that to its own overrun error.
    pub(crate) fn try_read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        self.sync();

        if self.pos.is_overrun() {
            self.pos.resync();
            return Err(());
        }

        let n = self.pos.available().min(buf.len());
        if n == 0 {
            return Ok(0);
        }

        let len = self.buf.len();
        let read_pos = self.pos.read_pos;

        // The available run may wrap the end of the pool.
        let head = (len - read_pos).min(n);
        buf[..head].copy_from_slice(&self.buf[read_pos..read_pos + head]);
        if n > head {
            buf[head..n].copy_from_slice(&self.buf[..n - head]);
        }

        self.pos.advance_read(n);
        Ok(n)
    }
}

impl<'d> Drop for RxRing<'d> {
    fn drop(&mut self) {
        let dch = pac::DMA.ch(self.data.number() as _);
        let cch = pac::DMA.ch(self.control.number() as _);

        // Break the chain first: aborting a channel that another channel chains to would just
        // let it be re-triggered. A channel chained to itself does not chain.
        dch.al1_ctrl().modify(|w| w.set_chain_to(self.data.number()));
        cch.al1_ctrl().modify(|w| w.set_chain_to(self.control.number()));

        // RP2350 errata RP2350-E5: clear EN before the abort so the channel cannot re-trigger.
        #[cfg(feature = "_rp235x")]
        {
            dch.al1_ctrl().modify(|w| w.set_en(false));
            cch.al1_ctrl().modify(|w| w.set_en(false));
        }

        pac::DMA
            .chan_abort()
            .modify(|m| m.set_chan_abort((1 << self.data.number()) | (1 << self.control.number())));
        while dch.ctrl_trig().read().busy() || cch.ctrl_trig().read().busy() {}
    }
}

/// Write a channel's transfer count, papering over the RP2040/RP2350 register shape difference.
pub(crate) fn set_trans_count(ch: pac::dma::Channel, count: u32) {
    #[cfg(feature = "rp2040")]
    ch.trans_count().write_value(count);
    #[cfg(feature = "_rp235x")]
    ch.trans_count().write(|w| {
        w.set_mode(pac::dma::vals::TransCountMode::Normal);
        w.set_count(count);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 256-byte pool: 4 segments of 64.
    const LEN: usize = 256;
    const SEG: usize = LEN / NBUFS;

    fn pos() -> Pos {
        Pos::new(LEN, 0)
    }

    #[test]
    fn starts_empty() {
        let p = pos();
        assert_eq!(p.available(), 0);
        assert!(!p.is_overrun());
    }

    #[test]
    fn tracks_within_first_segment() {
        let mut p = pos();
        p.apply(0, 10);
        assert_eq!(p.available(), 10);
        p.apply(0, 40);
        assert_eq!(p.available(), 40);
    }

    #[test]
    fn tracks_across_a_segment_completion() {
        let mut p = pos();
        // One segment done, 5 bytes into the second.
        p.apply(1, SEG + 5);
        assert_eq!(p.available(), SEG + 5);
    }

    #[test]
    fn tracks_across_a_full_lap() {
        let mut p = pos();
        // Four segments done: the write pointer is back at 0, but a whole pool was written.
        p.apply(4, 0);
        assert_eq!(p.available(), LEN);
        // And on into the next lap.
        p.apply(5, SEG + 3);
        assert_eq!(p.available(), LEN + SEG + 3);
    }

    /// The write address, not the counter, is the authority on how far the DMA got. A
    /// coalesced interrupt must not lose bytes.
    #[test]
    fn recovers_missed_completions() {
        let mut p = pos();
        // The counter says one segment, the address says we are three segments in.
        p.apply(1, 3 * SEG + 7);
        assert_eq!(p.available(), 3 * SEG + 7);

        let mut p = pos();
        // Worst recoverable case: NBUFS - 1 completions lost.
        p.apply(0, (NBUFS - 1) * SEG + 1);
        assert_eq!(p.available(), (NBUFS - 1) * SEG + 1);
    }

    /// Each merged interrupt leaves the counter one segment further behind for good. That drift
    /// must not add up to a lap and silently drop one.
    #[test]
    fn merged_completions_do_not_accumulate() {
        let mut p = pos();
        let mut counted = 0u32;
        let mut real = 0usize;
        let mut read = 0usize;
        for _ in 0..3 * NBUFS {
            // Two segments complete while the interrupt is held off: counted once.
            real += 2 * SEG;
            counted += 1;
            p.apply(counted, (real + 5) % LEN);
            assert_eq!(read + p.available(), real + 5);
            assert!(!p.is_overrun());
            read += p.available();
            p.advance_read(p.available());
        }
    }

    /// A completion counted after the address that already showed it must not count twice.
    #[test]
    fn a_late_count_is_not_counted_twice() {
        let mut p = pos();
        // The address is past a boundary the counter has not caught up with.
        p.apply(0, SEG + 20);
        assert_eq!(p.available(), SEG + 20);
        // The interrupt runs; nothing new was written.
        p.apply(1, SEG + 20);
        assert_eq!(p.available(), SEG + 20);
    }

    #[test]
    fn reading_consumes() {
        let mut p = pos();
        p.apply(1, SEG);
        assert_eq!(p.available(), SEG);
        p.advance_read(10);
        assert_eq!(p.available(), SEG - 10);
        assert_eq!(p.read_pos, 10);
    }

    #[test]
    fn read_pos_wraps() {
        let mut p = pos();
        p.apply(4, 0);
        p.advance_read(LEN - 4);
        assert_eq!(p.read_pos, LEN - 4);
        p.advance_read(8);
        assert_eq!(p.read_pos, 4);
    }

    #[test]
    fn overrun_when_writer_closes_on_reader() {
        let mut p = pos();
        // Three segments unread is fine; the fourth is the one being written into.
        p.apply(3, 3 * SEG);
        assert!(!p.is_overrun());
        p.apply(3, 3 * SEG + 1);
        assert!(p.is_overrun());
    }

    #[test]
    fn resync_discards_and_realigns() {
        let mut p = pos();
        p.apply(5, SEG + 9);
        assert!(p.is_overrun());
        p.resync();
        assert_eq!(p.available(), 0);
        assert!(!p.is_overrun());
        // The reader now sits exactly where the DMA is writing.
        assert_eq!(p.read_pos, (LEN + SEG + 9) % LEN);
    }

    /// A reader that keeps up stays correct over many laps.
    #[test]
    fn steady_state_over_many_laps() {
        let mut p = pos();
        let mut total = 0u64;
        for c in 1..=100u32 {
            p.apply(c, (c as usize % NBUFS) * SEG);
            let n = p.available();
            assert!(!p.is_overrun(), "unexpected overrun at completion {}", c);
            p.advance_read(n);
            total += n as u64;
        }
        assert_eq!(total, 100 * SEG as u64);
        assert_eq!(p.available(), 0);
    }

    /// A reader that stalls for more than the guard band must be told, not handed stale bytes.
    #[test]
    fn stalled_reader_gets_overrun() {
        let mut p = pos();
        // Reader takes the first segment, then goes away.
        p.apply(1, SEG);
        p.advance_read(SEG);
        assert!(!p.is_overrun());
        // DMA keeps going for three more segments: now unread data spans the guard band.
        p.apply(4, 0);
        p.apply(5, SEG + 1);
        assert!(p.is_overrun());
    }

    /// The limit of the reconstruction in `apply`: a whole lap between two samples that the
    /// counter does not show cannot be told apart from no progress, because the address is back
    /// where it was. Documented in the module docs; asserted here so it stays honest.
    #[test]
    fn a_full_lap_of_lost_completions_is_the_limit() {
        let mut p = pos();
        // NBUFS - 1 lost is recovered exactly.
        p.apply(0, (NBUFS - 1) * SEG);
        assert_eq!(p.available(), (NBUFS - 1) * SEG);

        // A full NBUFS lost looks identical to no progress at all.
        let mut q = pos();
        q.apply(0, 0); // really a whole lap in, but indistinguishable
        assert_eq!(q.available(), 0);
    }

    /// Interleaved reads and DMA progress keep both cursors consistent across a wrap.
    #[test]
    fn interleaved_reads_across_a_wrap() {
        let mut p = pos();
        let mut consumed = 0usize;
        for c in 1..=12u32 {
            p.apply(c, (c as usize % NBUFS) * SEG);
            // Drain half of what is there each time, leaving a remainder to carry.
            let n = p.available() / 2;
            p.advance_read(n);
            consumed += n;
            assert_eq!(p.read_pos, consumed % LEN);
            assert!(!p.is_overrun());
        }
    }

    /// The completion counter is a `u32` that wraps; the baseline subtraction must too.
    #[test]
    fn counter_wrap_is_handled() {
        let mut p = Pos::new(LEN, u32::MAX);
        p.apply(u32::MAX, 0);
        assert_eq!(p.available(), 0);
        // Counter wraps to 0, meaning one segment completed.
        p.apply(0, SEG);
        assert_eq!(p.available(), SEG);
        p.apply(1, 2 * SEG);
        assert_eq!(p.available(), 2 * SEG);
    }
}
