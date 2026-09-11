//! Ring-buffered DMA UART RX driver.
//!
//! This driver receives continuously into a circular pool of DMA buffers, with no
//! per-byte interrupt and no CPU involvement between buffers. Use it when the
//! interrupt-driven [`BufferedUart`](super::BufferedUart) can no longer keep up —
//! roughly above 460 kBaud, or when the RX interrupt load is crowding out other work.
//!
//! # How it works
//!
//! Two DMA channels are wired into a self-perpetuating loop:
//!
//! - The **data** channel copies `UARTDR` into one segment of the RX pool.
//! - The **control** channel walks a small table of segment pointers and writes the next
//!   one into the data channel's `WRITE_ADDR`, then chains back to re-trigger it.
//!
//! `TRANS_COUNT` is a reload register on RP2040/RP2350 — the written value is copied into
//! the live counter on every trigger — so neither channel ever needs re-arming. The control
//! channel's read address wraps around the pointer table via `RING_SIZE`/`RING_SEL`, so the
//! loop runs forever without the CPU.
//!
//! The data channel raises its completion interrupt (`irq_quiet = false`) once per segment,
//! which is what wakes a pending [`read`](RingBufferedUartRx::read). Reception itself does not
//! depend on that interrupt being serviced promptly; only the wakeup latency does.
//!
//! # Latency
//!
//! Data becomes readable as soon as the DMA write pointer passes it, but a *blocked* reader is
//! only woken on segment boundaries. Wake latency is therefore one segment
//! (`rx_buffer.len() / NBUFS` bytes). Size the buffer accordingly: a smaller pool wakes sooner,
//! a larger pool tolerates a slower reader.
//!
//! Note that the PL011 receive-timeout interrupt is useless here: with `RXDMAE` set, the RX DREQ
//! asserts whenever the FIFO is non-empty, so DMA drains it long before the 32-bit-period idle
//! condition can hold. There is no idle-line detection on this path.
//!
//! # Limitations
//!
//! Overrun detection is not exact. There is no lap counter in the hardware, so the driver
//! reconstructs how far the DMA advanced by differencing successive samples of the channel's
//! write address, and that difference is taken modulo the pool size. A reader that is starved
//! for a whole pool or more is therefore indistinguishable from one that is up to date, and can
//! be handed stale bytes without an [`Error::Overrun`].
//!
//! In practice the reader is woken once per segment, so it would have to miss `NBUFS`
//! consecutive wakeups to get there — but size the pool so that the worst-case gap between
//! reads stays well under one pool, and treat `Overrun` as "I was late", not as the only way
//! lateness can show up. Making this exact requires counting segment completions in the DMA
//! interrupt, which would mean this driver shipping its own handler instead of reusing
//! [`dma::InterruptHandler`].

use core::future::poll_fn;
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};
use core::task::Poll;

use super::*;

/// Number of DMA segments the RX pool is split into.
///
/// The pointer table is this many words and is wrapped by the DMA `RING_SIZE` field, so this
/// must stay a power of two.
const NBUFS: usize = 4;

const _: () = core::assert!(NBUFS.is_power_of_two());

/// `log2(size_of::<[u32; NBUFS]>())`, for `CTRL.RING_SIZE`.
const RING_SIZE: u8 = (NBUFS * 4).trailing_zeros() as u8;

/// Segment pointer table read by the control DMA channel.
///
/// Aligned to its own size so the DMA read-address ring wrap lands back on entry 0.
#[repr(C, align(16))]
struct PtrTable([AtomicU32; NBUFS]);

const _: () = core::assert!(size_of::<PtrTable>() == 1 << RING_SIZE);

/// State for a [`RingBufferedUartRx`].
///
/// This lives in a `static` per UART instance because the DMA hardware reads the pointer table
/// directly and the driver struct itself is movable.
pub struct State {
    table: PtrTable,
}

impl State {
    pub const fn new() -> Self {
        Self {
            table: PtrTable([const { AtomicU32::new(0) }; NBUFS]),
        }
    }
}

/// Ring-buffered DMA UART RX driver.
///
/// See the [module docs](self) for how the DMA loop is constructed.
pub struct RingBufferedUartRx<'d> {
    info: &'static Info,
    dma_state: &'static DmaState,

    /// Data channel: `UARTDR` -> pool. Held for the lifetime so the channel stays claimed.
    data: Channel<'d, Async>,
    /// Control channel: pointer table -> data channel's `WRITE_ADDR`.
    _control: Channel<'d, Blocking>,

    /// Base of the RX pool, and its total length. The pool is `NBUFS` equal segments.
    buf: &'d mut [u8],

    /// Reader position within the pool, in `0..buf.len()`.
    read_pos: usize,
    /// DMA write position as of the last poll, in `0..buf.len()`. Used to accumulate
    /// `written` across wraps.
    last_write_pos: usize,
    /// Monotonic count of bytes the DMA has written, reconstructed from `last_write_pos` deltas.
    written: u64,
    /// Monotonic count of bytes handed to the caller.
    read: u64,
}

impl<'d> RingBufferedUartRx<'d> {
    /// Create a ring-buffered DMA UART RX driver.
    ///
    /// `rx_buffer` is the DMA pool. Its length must be a non-zero multiple of 4 (the segment
    /// count); it is split into 4 equal segments internally. Wake latency is one segment, so
    /// e.g. a 256-byte pool wakes a blocked reader every 64 bytes.
    ///
    /// Two DMA channels are required. Only `rx_dma` needs an interrupt binding — the control
    /// channel runs silently.
    pub fn new<T: Instance, RxDma: ChannelInstance, CtrlDma: ChannelInstance>(
        _uart: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        rx_dma: Peri<'d, RxDma>,
        ctrl_dma: Peri<'d, CtrlDma>,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>> + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>> + 'd,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        super::Uart::<'d, Async>::init(T::info(), None, Some(rx.into()), None, None, config);

        let data = Channel::new(rx_dma, irq);
        // The control channel never raises an interrupt, so it does not need a binding.
        let control = Channel::new_no_interrupt(ctrl_dma);

        Self::new_inner(
            T::info(),
            T::dma_state(),
            T::ring_buffered_state(),
            data,
            control,
            rx_buffer,
        )
    }

    fn new_inner(
        info: &'static Info,
        dma_state: &'static DmaState,
        state: &'static State,
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

        // Clear any stale error flags, then arm the error interrupts. `dmaonerr` stays off:
        // a framing/parity error must not halt a continuously running ring.
        info.regs.uarticr().write(|w| w.0 = 0x780);
        dma_state.rx_errs.store(0, Ordering::Relaxed);
        info.regs.uartimsc().write_set(|w| {
            w.set_oeim(true);
            w.set_beim(true);
            w.set_peim(true);
            w.set_feim(true);
        });
        info.regs.uartdmacr().write_set(|w| {
            w.set_rxdmae(true);
            w.set_dmaonerr(false);
        });

        info.interrupt.unpend();
        unsafe { info.interrupt.enable() };

        // Control channel: one word from the pointer table into the data channel's WRITE_ADDR,
        // then chain back to re-trigger the data channel. The read address wraps the table.
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

        // Data channel: UARTDR into segment 0, then chain to the control channel.
        dch.read_addr().write_value(info.regs.uartdr().as_ptr() as u32);
        dch.write_addr().write_value(base as u32);
        set_trans_count(dch, seg_len as u32);
        dch.al1_ctrl().write_value({
            let mut w = pac::dma::regs::Ctrl(0);
            w.set_data_size(pac::dma::vals::DataSize::SizeByte);
            w.set_incr_read(false); // UARTDR is a fixed address
            w.set_incr_write(true);
            w.set_treq_sel(info.rx_dreq);
            w.set_chain_to(control.number());
            w.set_irq_quiet(false); // completion IRQ is the reader's wakeup
            w.set_en(true);
            w
        });

        compiler_fence(Ordering::SeqCst);

        // Start the loop. CTRL_TRIG is a trigger register, so this write starts the data
        // channel; the control channel is already enabled and will be pulled in by the chain.
        dch.ctrl_trig().write_value(dch.al1_ctrl().read());

        Self {
            info,
            dma_state,
            data,
            _control: control,
            buf,
            read_pos: 0,
            last_write_pos: 0,
            written: 0,
            read: 0,
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

    /// Fold the current DMA position into the monotonic `written` counter.
    fn sync_written(&mut self) {
        let pos = self.write_pos();
        // The position is read from a register, but the bytes it accounts for were written to
        // plain memory by the DMA. Keep the compiler from hoisting a read of that memory above
        // the position sample that authorised it.
        compiler_fence(Ordering::Acquire);
        let delta = pos.wrapping_sub(self.last_write_pos) % self.buf.len();
        self.written += delta as u64;
        self.last_write_pos = pos;
    }

    /// Take and classify any UART error the interrupt handler has latched.
    ///
    /// The handler masks the error interrupts off rather than clearing the flags, so they are
    /// re-armed here once the error has been reported.
    fn take_error(&mut self) -> Option<Error> {
        // AtomicU16 has no RMW on thumbv6m, so take-and-clear under a critical section,
        // the same way `UartRx::<Async>::read` does.
        let errs = critical_section::with(|_| {
            let v = self.dma_state.rx_errs.load(Ordering::Relaxed);
            self.dma_state.rx_errs.store(0, Ordering::Relaxed);
            v
        });
        if errs == 0 {
            return None;
        }

        let ris = Uartris(errs as u32);
        self.info.regs.uarticr().write(|w| w.0 = errs as u32);
        self.info.regs.uartimsc().write_set(|w| {
            w.set_oeim(true);
            w.set_beim(true);
            w.set_peim(true);
            w.set_feim(true);
        });

        if ris.oeris() {
            Some(Error::Overrun)
        } else if ris.beris() {
            Some(Error::Break)
        } else if ris.peris() {
            Some(Error::Parity)
        } else if ris.feris() {
            Some(Error::Framing)
        } else {
            None
        }
    }

    /// Resynchronise the reader onto the current DMA position, discarding buffered data.
    fn resync(&mut self) {
        self.sync_written();
        self.read_pos = self.last_write_pos;
        self.read = self.written;
    }

    /// Copy out whatever is currently available, if anything.
    ///
    /// Returns `Err(Overrun)` if the DMA has caught up with the reader, in which case the
    /// buffered data is discarded and reception continues from the current position.
    fn try_read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.sync_written();

        let len = self.buf.len();
        let available = (self.written - self.read) as usize;

        // The writer is within one segment of lapping the reader; anything still unread is
        // about to be, or already has been, overwritten.
        if available > len - len / NBUFS {
            self.resync();
            return Err(Error::Overrun);
        }

        let n = available.min(buf.len());
        if n == 0 {
            return Ok(0);
        }

        // The available run may wrap the end of the pool.
        let head = (len - self.read_pos).min(n);
        buf[..head].copy_from_slice(&self.buf[self.read_pos..self.read_pos + head]);
        if n > head {
            buf[head..n].copy_from_slice(&self.buf[..n - head]);
        }

        self.read_pos = (self.read_pos + n) % len;
        self.read += n as u64;
        Ok(n)
    }

    /// Read bytes into `buf`, waiting until at least one is available.
    ///
    /// Returns the number of bytes read, which is never zero unless `buf` is empty. This does
    /// not wait for `buf` to be filled — use [`embedded_io_async::Read::read_exact`] for that.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        poll_fn(|cx| {
            // Register before sampling, so a completion between the sample and the return of
            // Pending still wakes us.
            dma::channel_waker(self.data.number()).register(cx.waker());
            self.dma_state.rx_err_waker.register(cx.waker());

            if let Some(e) = self.take_error() {
                return Poll::Ready(Err(e));
            }

            match self.try_read(buf) {
                Ok(0) => Poll::Pending,
                r => Poll::Ready(r),
            }
        })
        .await
    }

    /// Read whatever is already buffered, without waiting.
    ///
    /// Returns `Ok(0)` if nothing has arrived yet.
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        if let Some(e) = self.take_error() {
            return Err(e);
        }
        self.try_read(buf)
    }

    /// Number of bytes currently available to read.
    pub fn len(&mut self) -> usize {
        self.sync_written();
        (self.written - self.read) as usize
    }

    /// Whether any data is currently available to read.
    pub fn is_empty(&mut self) -> bool {
        self.len() == 0
    }

    /// Whether a call to [`read`](Self::read) would return without waiting.
    pub fn read_ready(&mut self) -> Result<bool, Error> {
        Ok(!self.is_empty())
    }
}

impl<'d> Drop for RingBufferedUartRx<'d> {
    fn drop(&mut self) {
        let dch = pac::DMA.ch(self.data.number() as _);
        let cch = pac::DMA.ch(self._control.number() as _);

        // Break the chain first: aborting a channel that another channel chains to would just
        // let it be re-triggered. A channel chained to itself does not chain.
        dch.al1_ctrl().modify(|w| w.set_chain_to(self.data.number()));
        cch.al1_ctrl().modify(|w| w.set_chain_to(self._control.number()));

        // RP2350 errata RP2350-E5: clear EN before the abort so the channel cannot re-trigger.
        #[cfg(feature = "_rp235x")]
        {
            dch.al1_ctrl().modify(|w| w.set_en(false));
            cch.al1_ctrl().modify(|w| w.set_en(false));
        }

        pac::DMA
            .chan_abort()
            .modify(|m| m.set_chan_abort((1 << self.data.number()) | (1 << self._control.number())));
        while dch.ctrl_trig().read().busy() || cch.ctrl_trig().read().busy() {}

        self.info.regs.uartdmacr().write_clear(|w| w.set_rxdmae(true));
        self.info.regs.uartimsc().write_clear(|w| {
            w.set_oeim(true);
            w.set_beim(true);
            w.set_peim(true);
            w.set_feim(true);
        });
    }
}

/// Write a channel's transfer count, papering over the RP2040/RP2350 register shape difference.
fn set_trans_count(ch: pac::dma::Channel, count: u32) {
    #[cfg(feature = "rp2040")]
    ch.trans_count().write_value(count);
    #[cfg(feature = "_rp235x")]
    ch.trans_count().write(|w| {
        w.set_mode(pac::dma::vals::TransCountMode::Normal);
        w.set_count(count);
    });
}

impl<'d> embedded_io_async::ErrorType for RingBufferedUartRx<'d> {
    type Error = Error;
}

impl<'d> embedded_io_async::Read for RingBufferedUartRx<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Self::read(self, buf).await
    }
}

impl<'d> embedded_io_async::ReadReady for RingBufferedUartRx<'d> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        Self::read_ready(self)
    }
}
