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
