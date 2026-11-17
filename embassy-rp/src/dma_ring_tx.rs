//! Run-based DMA transmit ring.
//!
//! This is the transmit counterpart of the receive ring in `dma_ring`, but it is built
//! differently, because the problem is inverted. A receive ring has a hardware producer that
//! never stops, so its DMA loop runs forever. Here the producer is software and it is
//! intermittent: a DMA that looped over the pool would retransmit stale bytes as soon as it
//! caught up with the writer.
//!
//! # How it works
//!
//! The pool is a plain circular buffer with a software write position and a DMA read position.
//! One DMA channel transmits one bounded *run*, from the read position up to the write position
//! or the end of the pool, whichever comes first, and then stops. Its completion interrupt
//! accounts for the run and starts the next one if more data was queued meanwhile. A write that
//! wraps the end of the pool is therefore sent as two runs, so the pool can be any length.
//!
//! The write path and the interrupt handler both start runs, and they decide whether to under
//! a critical section, so a run is never started twice and queued data is never left sitting
//! with the channel idle. The section covers a handful of register writes.
//!
//! A DMA channel has a single interrupt handler. A channel driving a transmit ring must be bound
//! to [`TxInterruptHandler`], not to [`dma::InterruptHandler`].
//!
//! # Space accounting
//!
//! A byte's space is reusable as soon as the DMA has read it, not only once its run completes.
//! The live `READ_ADDR` gives the in-flight run's progress, so a writer blocked on a full pool
//! is not held up for the length of a whole run.

use core::cell::Cell;
use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Waker;

use critical_section::Mutex;

use crate::dma::{self, CHANNEL_COUNT, Channel, ChannelInstance};
use crate::dma_ring::set_trans_count;
use crate::mode::Blocking;
use crate::{Peri, interrupt, pac};

/// Queue accounting for a transmit pool, independent of any hardware.
///
/// Split out so the wrap arithmetic can be tested on the host. All counts are in bytes.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TxPos {
    /// Pool length.
    len: usize,
    /// Offset of the oldest byte not yet fully sent, in `0..len`.
    rd: usize,
    /// Bytes queued and not yet fully sent, including the in-flight run.
    pending: usize,
    /// Length of the run the DMA is working on, or 0 if the channel is idle.
    in_flight: usize,
}

impl TxPos {
    pub(crate) const fn new(len: usize) -> Self {
        Self {
            len,
            rd: 0,
            pending: 0,
            in_flight: 0,
        }
    }

    /// Bytes that can be queued, given how much of the in-flight run the DMA has already read.
    pub(crate) fn free(&self, consumed: usize) -> usize {
        self.len - (self.pending - consumed.min(self.in_flight))
    }

    /// Offset the next byte is written at.
    pub(crate) fn write_off(&self) -> usize {
        (self.rd + self.pending) % self.len
    }

    /// Account for `n` bytes the writer has copied in.
    pub(crate) fn commit(&mut self, n: usize) {
        debug_assert!(self.pending + n <= self.len);
        self.pending += n;
    }

    /// The next run to start as `(offset, length)`, if the channel is idle and data is waiting.
    pub(crate) fn next_run(&self) -> Option<(usize, usize)> {
        if self.in_flight != 0 || self.pending == 0 {
            return None;
        }
        Some((self.rd, self.pending.min(self.len - self.rd)))
    }

    pub(crate) fn start(&mut self, run: usize) {
        self.in_flight = run;
    }

    /// Account for the in-flight run having been fully read by the DMA.
    pub(crate) fn complete(&mut self) {
        self.rd = (self.rd + self.in_flight) % self.len;
        self.pending -= self.in_flight;
        self.in_flight = 0;
    }
}

/// What the interrupt handler needs to restart a channel, in a static per channel.
#[derive(Clone, Copy)]
struct Shared {
    /// Cleared when the ring is dropped, so a late interrupt does nothing.
    active: bool,
    base: u32,
    dst: u32,
    /// Prebuilt `CTRL` value, written to the trigger register to start a run.
    ctrl: u32,
    pos: TxPos,
}

impl Shared {
    const fn new() -> Self {
        Self {
            active: false,
            base: 0,
            dst: 0,
            ctrl: 0,
            pos: TxPos::new(1),
        }
    }

    /// Bytes of the in-flight run the DMA has already read.
    fn consumed(&self, number: u8) -> usize {
        if self.pos.in_flight == 0 {
            return 0;
        }
        let start = self.base + self.pos.rd as u32;
        let read = pac::DMA.ch(number as _).read_addr().read();
        (read.wrapping_sub(start) as usize).min(self.pos.in_flight)
    }

    /// Start the next run if the channel is idle and data is waiting.
    fn arm(&mut self, number: u8) {
        let Some((off, run)) = self.pos.next_run() else {
            return;
        };
        let ch = pac::DMA.ch(number as _);
        ch.read_addr().write_value(self.base + off as u32);
        ch.write_addr().write_value(self.dst);
        set_trans_count(ch, run as u32);
        // The bytes were written to plain memory; make sure that is done before the DMA can
        // start reading it.
        compiler_fence(Ordering::SeqCst);
        // CTRL_TRIG is a trigger register, so this write starts the run.
        ch.ctrl_trig().write_value(pac::dma::regs::Ctrl(self.ctrl));
        self.pos.start(run);
    }
}

static SHARED: [Mutex<Cell<Shared>>; CHANNEL_COUNT] = [const { Mutex::new(Cell::new(Shared::new())) }; CHANNEL_COUNT];

/// Interrupt handler for a DMA channel driving a ring-buffered TX driver.
///
/// Bind this instead of [`dma::InterruptHandler`] for the channel
/// handed to a ring-buffered TX driver.
pub struct TxInterruptHandler<T: ChannelInstance> {
    _phantom: PhantomData<T>,
}

impl<T: ChannelInstance> interrupt::typelevel::Handler<T::Interrupt> for TxInterruptHandler<T> {
    unsafe fn on_interrupt() {
        let number = T::number();
        if pac::DMA.ints(0).read() & (1 << number) == 0 {
            return;
        }
        pac::DMA.ints(0).write_value(1 << number);

        critical_section::with(|cs| {
            let cell = SHARED[number as usize].borrow(cs);
            let mut s = cell.get();
            if s.active {
                if s.pos.in_flight != 0 {
                    s.pos.complete();
                }
                s.arm(number);
                cell.set(s);
            }
        });

        dma::channel_waker(number).wake();
    }
}

/// A run-based DMA transmit ring.
///
/// See the [module docs](self) for how it works.
pub(crate) struct TxRing<'d> {
    ch: Channel<'d, Blocking>,
    buf: &'d mut [u8],
}

impl<'d> TxRing<'d> {
    /// Build a transmit ring. Nothing is sent until something is written.
    ///
    /// `dst` is the register the channel writes to, one byte at a time, paced by `dreq`.
    pub(crate) fn new<T: ChannelInstance>(
        ch: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, TxInterruptHandler<T>> + 'd,
        dst: *mut u8,
        dreq: pac::dma::vals::TreqSel,
        buf: &'d mut [u8],
    ) -> Self {
        let len = buf.len();
        assert!(len > 0, "tx_buffer must not be empty");
        assert!(len <= 0x0fff_ffff, "tx_buffer must fit in a DMA transfer count");

        // SAFETY: the interrupt is handled by `TxInterruptHandler`, which the `Binding` above
        // proves is bound.
        let ch = unsafe { Channel::new_blocking(ch) };
        let number = ch.number();

        let mut ctrl = pac::dma::regs::Ctrl(0);
        ctrl.set_data_size(pac::dma::vals::DataSize::SizeByte);
        ctrl.set_incr_read(true);
        ctrl.set_incr_write(false); // the destination is a fixed register
        ctrl.set_treq_sel(dreq);
        ctrl.set_chain_to(number); // chained to itself, i.e. no chaining
        ctrl.set_irq_quiet(false);
        ctrl.set_en(true);

        // A completion left latched by a previous user of the channel would be mistaken for ours.
        pac::DMA.ints(0).write_value(1 << number);

        critical_section::with(|cs| {
            SHARED[number as usize].borrow(cs).set(Shared {
                active: true,
                base: buf.as_ptr() as u32,
                dst: dst as u32,
                ctrl: ctrl.0,
                pos: TxPos::new(len),
            });
        });

        Self { ch, buf }
    }

    fn cell(&self) -> &'static Mutex<Cell<Shared>> {
        &SHARED[self.ch.number() as usize]
    }

    /// Queue as much of `data` as fits and return how much that was. Never waits.
    pub(crate) fn try_write(&mut self, data: &[u8]) -> usize {
        let number = self.ch.number();
        let len = self.buf.len();

        let (n, off) = critical_section::with(|cs| {
            let s = self.cell().borrow(cs).get();
            let free = s.pos.free(s.consumed(number));
            (free.min(data.len()), s.pos.write_off())
        });
        if n == 0 {
            return 0;
        }

        // Only this ring writes the pool, and `[off, off + n)` (mod len) is free, so the copy
        // does not touch anything the DMA may still read. The interrupt handler can only free
        // more space, and never moves `rd + pending`, so `off` stays valid.
        let head = (len - off).min(n);
        self.buf[off..off + head].copy_from_slice(&data[..head]);
        self.buf[..n - head].copy_from_slice(&data[head..n]);

        critical_section::with(|cs| {
            let cell = self.cell().borrow(cs);
            let mut s = cell.get();
            s.pos.commit(n);
            s.arm(number);
            cell.set(s);
        });
        n
    }

    /// Bytes that could be queued right now.
    pub(crate) fn free(&mut self) -> usize {
        let number = self.ch.number();
        critical_section::with(|cs| {
            let s = self.cell().borrow(cs).get();
            s.pos.free(s.consumed(number))
        })
    }

    /// Bytes queued that the DMA has not finished reading.
    pub(crate) fn pending(&mut self) -> usize {
        critical_section::with(|cs| self.cell().borrow(cs).get().pos.pending)
    }

    /// Register to be woken when a run completes.
    pub(crate) fn register_waker(&self, waker: &Waker) {
        dma::channel_waker(self.ch.number()).register(waker);
    }
}

impl<'d> Drop for TxRing<'d> {
    fn drop(&mut self) {
        let number = self.ch.number();
        critical_section::with(|cs| {
            let cell = self.cell().borrow(cs);
            let mut s = cell.get();
            s.active = false;
            cell.set(s);
        });

        let ch = pac::DMA.ch(number as _);
        // RP2350 errata RP2350-E5: clear EN before the abort so the channel cannot re-trigger.
        #[cfg(feature = "_rp235x")]
        ch.al1_ctrl().modify(|w| w.set_en(false));
        pac::DMA.chan_abort().modify(|m| m.set_chan_abort(1 << number));
        while ch.ctrl_trig().read().busy() {}
        pac::DMA.ints(0).write_value(1 << number);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Start whatever run is next, as the interrupt handler would.
    fn arm(p: &mut TxPos) -> Option<(usize, usize)> {
        let run = p.next_run()?;
        p.start(run.1);
        Some(run)
    }

    #[test]
    fn starts_empty_and_idle() {
        let p = TxPos::new(16);
        assert_eq!(p.free(0), 16);
        assert_eq!(p.next_run(), None);
    }

    #[test]
    fn one_run_when_it_does_not_wrap() {
        let mut p = TxPos::new(16);
        p.commit(10);
        assert_eq!(arm(&mut p), Some((0, 10)));
        assert_eq!(p.free(0), 6);
        p.complete();
        assert_eq!(p.free(0), 16);
        assert_eq!(p.next_run(), None);
    }

    #[test]
    fn a_wrapping_write_is_two_runs() {
        let mut p = TxPos::new(16);
        p.commit(12);
        arm(&mut p);
        p.complete();
        // rd = 12: 4 bytes to the end, then 6 from the start.
        assert_eq!(p.write_off(), 12);
        p.commit(10);
        assert_eq!(arm(&mut p), Some((12, 4)));
        p.complete();
        assert_eq!(arm(&mut p), Some((0, 6)));
        p.complete();
        assert_eq!(p.next_run(), None);
        assert_eq!(p.rd, 6);
    }

    #[test]
    fn no_second_run_while_one_is_in_flight() {
        let mut p = TxPos::new(16);
        p.commit(4);
        arm(&mut p);
        p.commit(4);
        assert_eq!(p.next_run(), None);
        p.complete();
        assert_eq!(arm(&mut p), Some((4, 4)));
    }

    #[test]
    fn space_is_reusable_as_the_dma_reads() {
        let mut p = TxPos::new(16);
        p.commit(16);
        arm(&mut p);
        assert_eq!(p.free(0), 0);
        assert_eq!(p.free(5), 5);
        // Never claims more than the run.
        assert_eq!(p.free(100), 16);
    }

    #[test]
    fn write_offset_is_stable_across_completion() {
        let mut p = TxPos::new(16);
        p.commit(10);
        arm(&mut p);
        let before = p.write_off();
        p.complete();
        assert_eq!(p.write_off(), before);
    }

    #[test]
    fn full_pool_wraps_correctly_over_many_laps() {
        let mut p = TxPos::new(10);
        let mut sent = 0usize;
        for i in 0..1000 {
            let n = (i % 7) + 1;
            let n = n.min(p.free(0));
            p.commit(n);
            while arm(&mut p).is_some() {
                sent += p.in_flight;
                p.complete();
            }
            assert_eq!(p.pending, 0);
        }
        assert!(sent > 0);
    }
}
