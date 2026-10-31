//! Ring-buffered DMA UART RX driver.
//!
//! This driver receives continuously into a circular pool of DMA buffers, with no
//! per-byte interrupt and no CPU involvement between buffers. Use it when the
//! interrupt-driven [`BufferedUart`](super::BufferedUart) can no longer keep up —
//! roughly above 460 kBaud, or when the RX interrupt load is crowding out other work.
//!
//! The DMA machinery lives in [`crate::dma_ring`]; see its docs for how the two chained
//! channels are wired and how the stream position is reconstructed. This module adds the
//! PL011-specific parts: pin and baud setup, and error reporting.
//!
//! # Latency
//!
//! Data becomes readable as soon as the DMA write pointer passes it, but a *blocked* reader is
//! only woken on segment boundaries. Wake latency is therefore one segment
//! (`rx_buffer.len() / 4` bytes). Size the buffer accordingly: a smaller pool wakes sooner,
//! a larger pool tolerates a slower reader.
//!
//! There is **no idle-line detection on this path**, and there cannot be. The PL011
//! receive-timeout interrupt is useless behind DMA: with `RXDMAE` set the RX DREQ asserts
//! whenever the FIFO is non-empty, so the DMA drains it long before the 32-bit-period idle
//! condition can hold, and the interrupt never fires. A reader cannot be woken at a frame
//! boundary — only at a segment boundary.
//!
//! # Limitations
//!
//! Overrun detection is not exact; see [`crate::dma_ring`] for the precise bound. In short, a
//! reader that falls behind by more than the pool minus one segment gets [`Error::Overrun`], but
//! one that falls a whole pool or more behind may get stale bytes with no error. Size the pool so
//! the worst-case gap between reads stays well under one pool, and treat `Overrun` as "I was
//! late", not as the only way lateness can show up.
//!
//! Bytes become readable as soon as the DMA writes them, not only at segment boundaries. At high
//! baud rates, a read loop that takes longer per call than one byte time always finds a new byte
//! waiting: each read then returns a few bytes and the task never sleeps, starving the rest of
//! its executor. Read into a buffer of at least one segment (a quarter of the pool) and keep the
//! work per read small.

use core::future::poll_fn;
use core::sync::atomic::Ordering;
use core::task::Poll;

use super::*;
use crate::dma_ring::{RxRing, State};
use crate::dma_ring_tx::{TxInterruptHandler, TxRing};

/// Ring-buffered DMA UART RX driver.
///
/// A blocked reader is woken once per DMA segment, a quarter of the pool, not at the end of a
/// frame: the PL011 receive-timeout interrupt never fires while DMA drains the FIFO. Overrun
/// detection is approximate, so size the pool to keep the worst-case gap between reads well
/// under one pool.
pub struct RingBufferedUartRx<'d> {
    info: &'static Info,
    dma_state: &'static DmaState,
    ring: RxRing<'d>,
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
        // Clear any stale error flags, then arm the error interrupts.
        info.regs.uarticr().write(|w| w.0 = 0x780);
        dma_state.rx_errs.store(0, Ordering::Relaxed);
        info.regs.uartimsc().write_set(|w| {
            w.set_oeim(true);
            w.set_beim(true);
            w.set_peim(true);
            w.set_feim(true);
        });

        // `dmaonerr` must stay off: a framing or parity error must not halt a ring that is
        // meant to run continuously.
        info.regs.uartdmacr().write_set(|w| {
            w.set_rxdmae(true);
            w.set_dmaonerr(false);
        });

        info.interrupt.unpend();
        unsafe { info.interrupt.enable() };

        let ring = RxRing::new(
            state,
            info.regs.uartdr().as_ptr() as *const u8,
            info.rx_dreq,
            data,
            control,
            buf,
        );

        Self { info, dma_state, ring }
    }

    /// Take and classify any UART error the interrupt handler has latched.
    ///
    /// The handler masks the error interrupts off rather than clearing the flags, so they are
    /// re-armed here once the error has been reported.
    fn take_error(&mut self) -> Option<Error> {
        // Fast path: `read` calls this every time, and errors are rare.
        if self.dma_state.rx_errs.load(Ordering::Relaxed) == 0 {
            return None;
        }
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
            self.ring.register_waker(cx.waker());
            self.dma_state.rx_err_waker.register(cx.waker());

            if let Some(e) = self.take_error() {
                return Poll::Ready(Err(e));
            }

            match self.ring.try_read(buf) {
                Ok(0) => Poll::Pending,
                Ok(n) => Poll::Ready(Ok(n)),
                Err(()) => Poll::Ready(Err(Error::Overrun)),
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
        self.ring.try_read(buf).map_err(|()| Error::Overrun)
    }

    /// Number of bytes currently available to read.
    pub fn len(&mut self) -> usize {
        self.ring.len()
    }

    /// Whether any data is currently available to read.
    pub fn is_empty(&mut self) -> bool {
        self.ring.is_empty()
    }

    /// Whether a call to [`read`](Self::read) would return without waiting.
    pub fn read_ready(&mut self) -> Result<bool, Error> {
        Ok(!self.is_empty())
    }
}

impl<'d> Drop for RingBufferedUartRx<'d> {
    fn drop(&mut self) {
        // Stop the DREQ source first; `RxRing`'s own Drop then breaks the chain and aborts
        // both channels, and it runs after this body.
        self.info.regs.uartdmacr().write_clear(|w| w.set_rxdmae(true));
        self.info.regs.uartimsc().write_clear(|w| {
            w.set_oeim(true);
            w.set_beim(true);
            w.set_peim(true);
            w.set_feim(true);
        });
    }
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

/// Ring-buffered DMA UART TX driver.
///
/// Writes are copied into a circular pool and sent by one DMA channel, with no per-byte
/// interrupt: the CPU is involved once per contiguous run of queued bytes, not once per FIFO
/// refill. Use it when the interrupt-driven [`BufferedUartTx`] spends too much time in its
/// interrupt at high baud rates.
///
/// Unlike the receive side, transmit cannot run the DMA continuously, because a looping DMA
/// would resend old bytes whenever it caught up with the writer. See [`crate::dma_ring_tx`] for
/// how runs are started and why the pool can be any length.
///
/// The DMA channel must be bound to [`TxInterruptHandler`](crate::dma_ring_tx::TxInterruptHandler),
/// not to [`dma::InterruptHandler`].
pub struct RingBufferedUartTx<'d> {
    info: &'static Info,
    ring: TxRing<'d>,
}

impl<'d> RingBufferedUartTx<'d> {
    /// Create a ring-buffered DMA UART TX driver.
    ///
    /// `tx_buffer` is the pool; any non-empty length works. Larger pools let more be queued
    /// before [`write`](Self::write) has to wait.
    pub fn new<T: Instance, TxDma: ChannelInstance>(
        _uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        tx_dma: Peri<'d, TxDma>,
        irq: impl Binding<TxDma::Interrupt, TxInterruptHandler<TxDma>> + 'd,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        let info = T::info();
        super::Uart::<'d, Async>::init(info, Some(tx.into()), None, None, None, config);

        Self::new_inner(info, tx_dma, irq, tx_buffer)
    }

    /// Everything but the UART setup, which a combined driver does once for both halves.
    fn new_inner<TxDma: ChannelInstance>(
        info: &'static Info,
        tx_dma: Peri<'d, TxDma>,
        irq: impl Binding<TxDma::Interrupt, TxInterruptHandler<TxDma>> + 'd,
        tx_buffer: &'d mut [u8],
    ) -> Self {
        let ring = TxRing::new(
            tx_dma,
            irq,
            info.regs.uartdr().as_ptr() as *mut u8,
            info.tx_dreq,
            tx_buffer,
        );
        info.regs.uartdmacr().write_set(|w| w.set_txdmae(true));

        Self { info, ring }
    }

    /// Queue as much of `buf` as fits and return how much that was, without waiting.
    pub fn try_write(&mut self, buf: &[u8]) -> usize {
        self.ring.try_write(buf)
    }

    /// Queue bytes from `buf`, waiting until at least one fits.
    ///
    /// Returns the number of bytes queued, which is never zero unless `buf` is empty. This does
    /// not wait for `buf` to be queued in full — use [`embedded_io_async::Write::write_all`]
    /// for that. Queued bytes are sent in the background; use [`flush`](Self::flush) to wait for
    /// them to leave the wire.
    pub async fn write(&mut self, buf: &[u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }
        poll_fn(|cx| {
            // Register before checking, so a run completing in between still wakes us.
            self.ring.register_waker(cx.waker());
            match self.ring.try_write(buf) {
                0 => Poll::Pending,
                n => Poll::Ready(n),
            }
        })
        .await
    }

    /// Wait until every queued byte has been sent, including the one still being shifted out.
    pub async fn flush(&mut self) {
        poll_fn(|cx| {
            self.ring.register_waker(cx.waker());
            if self.ring.pending() != 0 {
                return Poll::Pending;
            }
            // The DMA is done, but the FIFO and the shift register are not. There is no
            // interrupt for that, so poll.
            if self.info.regs.uartfr().read().busy() {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Ready(())
        })
        .await
    }

    /// Bytes that could be queued right now.
    pub fn free_space(&mut self) -> usize {
        self.ring.free()
    }

    /// Bytes queued that the DMA has not finished reading.
    pub fn pending(&mut self) -> usize {
        self.ring.pending()
    }
}

impl<'d> Drop for RingBufferedUartTx<'d> {
    fn drop(&mut self) {
        // Stop the DREQ source; `TxRing`'s own Drop then aborts the channel.
        self.info.regs.uartdmacr().write_clear(|w| w.set_txdmae(true));
    }
}

impl<'d> embedded_io_async::ErrorType for RingBufferedUartTx<'d> {
    type Error = core::convert::Infallible;
}

impl<'d> embedded_io_async::Write for RingBufferedUartTx<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(Self::write(self, buf).await)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Self::flush(self).await;
        Ok(())
    }
}

/// Ring-buffered DMA UART with both directions on one UART.
///
/// This is a [`RingBufferedUartRx`] and a [`RingBufferedUartTx`] set up together, so the UART
/// and both pins are configured once. Use [`split`](Self::split) to hand the halves to separate
/// tasks, or [`split_ref`](Self::split_ref) to borrow them.
///
/// Three DMA channels are needed: a data and a control channel for receive, and one for
/// transmit. The transmit channel must be bound to
/// [`TxInterruptHandler`](crate::dma_ring_tx::TxInterruptHandler).
pub struct RingBufferedUart<'d> {
    tx: RingBufferedUartTx<'d>,
    rx: RingBufferedUartRx<'d>,
}

impl<'d> RingBufferedUart<'d> {
    /// Create a ring-buffered DMA UART.
    ///
    /// See [`RingBufferedUartRx::new`] and [`RingBufferedUartTx::new`] for the buffer
    /// requirements: `rx_buffer` must be a non-zero multiple of 4 bytes, `tx_buffer` any
    /// non-empty length.
    #[allow(clippy::too_many_arguments)]
    pub fn new<T: Instance, RxDma: ChannelInstance, CtrlDma: ChannelInstance, TxDma: ChannelInstance>(
        _uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        rx_dma: Peri<'d, RxDma>,
        ctrl_dma: Peri<'d, CtrlDma>,
        tx_dma: Peri<'d, TxDma>,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>>
        + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
        + Binding<TxDma::Interrupt, TxInterruptHandler<TxDma>>
        + 'd,
        rx_buffer: &'d mut [u8],
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        let info = T::info();
        super::Uart::<'d, Async>::init(info, Some(tx.into()), Some(rx.into()), None, None, config);

        let data = Channel::new(rx_dma, irq);
        let control = Channel::new_no_interrupt(ctrl_dma);

        Self {
            rx: RingBufferedUartRx::new_inner(info, T::dma_state(), T::ring_buffered_state(), data, control, rx_buffer),
            tx: RingBufferedUartTx::new_inner(info, tx_dma, irq, tx_buffer),
        }
    }

    /// Split into independent transmitter and receiver halves.
    pub fn split(self) -> (RingBufferedUartTx<'d>, RingBufferedUartRx<'d>) {
        (self.tx, self.rx)
    }

    /// Borrow the transmitter and receiver halves.
    pub fn split_ref(&mut self) -> (&mut RingBufferedUartTx<'d>, &mut RingBufferedUartRx<'d>) {
        (&mut self.tx, &mut self.rx)
    }

    /// See [`RingBufferedUartRx::read`].
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.rx.read(buf).await
    }

    /// See [`RingBufferedUartRx::blocking_read`].
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.rx.blocking_read(buf)
    }

    /// See [`RingBufferedUartTx::write`].
    pub async fn write(&mut self, buf: &[u8]) -> usize {
        self.tx.write(buf).await
    }

    /// See [`RingBufferedUartTx::try_write`].
    pub fn try_write(&mut self, buf: &[u8]) -> usize {
        self.tx.try_write(buf)
    }

    /// See [`RingBufferedUartTx::flush`].
    pub async fn flush(&mut self) {
        self.tx.flush().await
    }
}

impl<'d> embedded_io_async::ErrorType for RingBufferedUart<'d> {
    type Error = Error;
}

impl<'d> embedded_io_async::Read for RingBufferedUart<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Self::read(self, buf).await
    }
}

impl<'d> embedded_io_async::ReadReady for RingBufferedUart<'d> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        self.rx.read_ready()
    }
}

impl<'d> embedded_io_async::Write for RingBufferedUart<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(Self::write(self, buf).await)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Self::flush(self).await;
        Ok(())
    }
}
