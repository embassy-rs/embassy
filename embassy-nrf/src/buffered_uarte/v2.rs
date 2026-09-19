//! Async buffered UART driver.
//!
//! Note that discarding a future from a read or write operation may lead to losing
//! data. For example, when using `futures_util::future::select` and completion occurs
//! on the "other" future, you should capture the incomplete future and continue to use
//! it for the next read or write. This pattern is a consideration for all IO, and not
//! just serial communications.
//!
//! Please also see [crate::uarte] to understand when [BufferedUarte] should be used.
//!
//! The code is based on the generic buffered_uarte implementation but uses the nrf54l
//! frame timeout event to correctly determine the size of transferred data.
//! Counting of rxrdy events, used in the generic implementation, cannot be applied
//! to nrf54l chips, as they buffer up to 4 bytes in a single DMA transaction.
//! The only reliable way to find the number of bytes received is to stop the transfer,
//! wait for the DMA stopped event, and read the value in the rx.dma.amount register.
//! This also flushes all in-flight data to RAM.

use core::cmp::min;
use core::future::{Future, poll_fn};
use core::marker::PhantomData;
use core::slice;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::Peri;
use embassy_hal_internal::atomic_ring_buffer::RingBuffer;
use pac::uarte::vals;
// Re-export SVD variants to allow user to directly set values
pub use pac::uarte::vals::{Baudrate, ConfigParity as Parity};

use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::interrupt::InterruptExt;
use crate::interrupt::typelevel::Interrupt;
use crate::uarte::{
    Config, DMA_SIZE, Instance as UarteInstance, configure, configure_rx_pins, configure_tx_pins, drop_tx_rx,
};
use crate::{interrupt, pac};

pub(crate) struct State {
    tx_buf: RingBuffer,
    tx_count: AtomicUsize,

    rx_buf: RingBuffer,
    /// Whether a RX DMA transfer is currently in progress.
    rx_started: AtomicBool,
}

/// UART error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    // No errors for now
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            tx_buf: RingBuffer::new(),
            tx_count: AtomicUsize::new(0),

            rx_buf: RingBuffer::new(),
            rx_started: AtomicBool::new(false),
        }
    }
}

/// Interrupt handler.
pub struct InterruptHandler<U: UarteInstance> {
    _phantom: PhantomData<U>,
}

impl<U: UarteInstance> interrupt::typelevel::Handler<U::Interrupt> for InterruptHandler<U> {
    unsafe fn on_interrupt() {
        let r = U::regs();
        let ss = U::state();
        let s = U::buffered_state();

        if let Some(mut rx) = unsafe { s.rx_buf.try_writer() } {
            let buf_len = s.rx_buf.len();
            let half_len = buf_len / 2;

            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                let errs = r.errorsrc().read();
                r.errorsrc().write_value(errs);

                if errs.overrun() {
                    panic!("BufferedUarte UART overrun");
                }
            }

            if r.events_dma().rx().end().read() != 0 {
                //trace!("  irq_rx: endrx");
                r.events_dma().rx().end().write_value(0);

                if s.rx_started.swap(false, Ordering::Relaxed) {
                    // Received some bytes, wake task.
                    let rxed = r.dma().rx().amount().read().amount() as usize;
                    rx.push_done(rxed);
                    ss.rx_waker.wake();
                }
            }

            if !s.rx_started.load(Ordering::Relaxed) {
                let (ptr, len) = rx.push_buf();

                // If the buffer is full, leave the DMA stopped. `consume()` pends the interrupt
                // again once the user has freed up some space.
                if len != 0 {
                    let len = min(len, half_len);

                    // Set up the DMA read
                    r.dma().rx().ptr().write_value(ptr as u32);
                    r.dma().rx().maxcnt().write(|w| w.set_maxcnt(len as _));

                    s.rx_started.store(true, Ordering::Relaxed);

                    // manually start
                    r.tasks_dma().rx().start().write_value(1);
                }
            }
        }

        // =============================

        if let Some(mut tx) = unsafe { s.tx_buf.try_reader() } {
            // TX end
            if r.events_dma().tx().end().read() != 0 {
                r.events_dma().tx().end().write_value(0);

                let n = s.tx_count.load(Ordering::Relaxed);
                //trace!("  irq_tx: endtx {:?}", n);
                tx.pop_done(n);
                ss.tx_waker.wake();
                s.tx_count.store(0, Ordering::Relaxed);
            }

            // If not TXing, start.
            if s.tx_count.load(Ordering::Relaxed) == 0 {
                let (ptr, len) = tx.pop_buf();
                let len = len.min(DMA_SIZE);
                if len != 0 {
                    //trace!("  irq_tx: starting {:?}", len);
                    s.tx_count.store(len, Ordering::Relaxed);

                    // Set up the DMA write
                    r.dma().tx().ptr().write_value(ptr as u32);
                    r.dma().tx().maxcnt().write(|w| w.set_maxcnt(len as _));

                    // Start UARTE Transmit transaction
                    r.tasks_dma().tx().start().write_value(1);
                }
            }
        }

        //trace!("irq: end");
    }
}

/// Buffered UARTE driver.
pub struct BufferedUarte<'d> {
    tx: BufferedUarteTx<'d>,
    rx: BufferedUarteRx<'d>,
}

impl<'d> Unpin for BufferedUarte<'d> {}

impl<'d> BufferedUarte<'d> {
    /// Create a new BufferedUarte without hardware flow control.
    #[allow(clippy::too_many_arguments)]
    pub fn new<U: UarteInstance>(
        uarte: Peri<'d, U>,
        txd: Peri<'d, impl GpioPin>,
        rxd: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        Self::new_inner(uarte, txd.into(), rxd.into(), None, None, tx_buffer, rx_buffer, config)
    }

    /// Create a new BufferedUarte with hardware flow control (RTS/CTS)
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_rtscts<U: UarteInstance>(
        uarte: Peri<'d, U>,
        txd: Peri<'d, impl GpioPin>,
        rxd: Peri<'d, impl GpioPin>,
        cts: Peri<'d, impl GpioPin>,
        rts: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        Self::new_inner(
            uarte,
            txd.into(),
            rxd.into(),
            Some(cts.into()),
            Some(rts.into()),
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_inner<U: UarteInstance>(
        peri: Peri<'d, U>,
        txd: Peri<'d, AnyPin>,
        rxd: Peri<'d, AnyPin>,
        cts: Option<Peri<'d, AnyPin>>,
        rts: Option<Peri<'d, AnyPin>>,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        let r = U::regs();
        let irq = U::Interrupt::IRQ;

        configure(r, config, cts.is_some());

        let tx = BufferedUarteTx::new_innerer(unsafe { peri.clone_unchecked() }, txd, cts, tx_buffer);
        let rx = BufferedUarteRx::new_innerer(peri, rxd, rts, rx_buffer);

        r.enable().write(|w| w.set_enable(vals::Enable::Enabled));
        irq.pend();
        unsafe { irq.enable() };

        U::state().tx_rx_refcount.store(2, Ordering::Relaxed);

        Self { tx, rx }
    }

    /// Adjust the baud rate to the provided value.
    pub fn set_baudrate(&mut self, baudrate: Baudrate) {
        self.tx.set_baudrate(baudrate);
    }

    /// Split the UART in writer and reader parts.
    ///
    /// This allows reading and writing concurrently from independent tasks.
    pub fn split(self) -> (BufferedUarteTx<'d>, BufferedUarteRx<'d>) {
        (self.tx, self.rx)
    }

    /// Split the UART in writer and reader parts, by reference.
    ///
    /// The returned halves borrow from `self`, so you can drop them and go back to using
    /// the "un-split" `self`. This allows temporarily splitting the UART.
    pub fn split_ref(&mut self) -> (BufferedUarteTx<'_>, BufferedUarteRx<'_>) {
        (
            BufferedUarteTx {
                r: self.tx.r,
                irq: self.tx.irq,
                state: self.tx.state,
                buffered_state: self.tx.buffered_state,
                is_borrowed: true,
                _p: PhantomData,
            },
            BufferedUarteRx {
                r: self.rx.r,
                irq: self.rx.irq,
                state: self.rx.state,
                buffered_state: self.rx.buffered_state,
                is_borrowed: true,
                _p: PhantomData,
            },
        )
    }

    /// Pull some bytes from this source into the specified buffer, returning how many bytes were read.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.rx.read(buf).await
    }

    /// Return the contents of the internal buffer, filling it with more data from the inner reader if it is empty.
    pub async fn fill_buf(&mut self) -> Result<&[u8], Error> {
        self.rx.fill_buf().await
    }

    /// Tell this buffer that `amt` bytes have been consumed from the buffer, so they should no longer be returned in calls to `fill_buf`.
    pub fn consume(&mut self, amt: usize) {
        self.rx.consume(amt)
    }

    /// Write a buffer into this writer, returning how many bytes were written.
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.tx.write(buf).await
    }

    /// Try writing a buffer without waiting, returning how many bytes were written.
    pub fn try_write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.tx.try_write(buf)
    }

    /// Flush this output stream, ensuring that all intermediately buffered contents reach their destination.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.tx.flush().await
    }
}

/// Writer part of the buffered UARTE driver.
pub struct BufferedUarteTx<'d> {
    r: pac::uarte::Uarte,
    irq: interrupt::Interrupt,
    state: &'static crate::uarte::State,
    buffered_state: &'static State,
    /// Whether this half was borrowed via `split_ref`, in which case dropping it must not tear down the peripheral.
    is_borrowed: bool,
    _p: PhantomData<&'d ()>,
}

impl<'d> BufferedUarteTx<'d> {
    /// Create a new BufferedUarteTx without hardware flow control.
    pub fn new<U: UarteInstance>(
        uarte: Peri<'d, U>,
        txd: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        Self::new_inner(uarte, txd.into(), None, tx_buffer, config)
    }

    /// Create a new BufferedUarteTx with hardware flow control (CTS)
    pub fn new_with_cts<U: UarteInstance>(
        uarte: Peri<'d, U>,
        txd: Peri<'d, impl GpioPin>,
        cts: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        Self::new_inner(uarte, txd.into(), Some(cts.into()), tx_buffer, config)
    }

    fn new_inner<U: UarteInstance>(
        peri: Peri<'d, U>,
        txd: Peri<'d, AnyPin>,
        cts: Option<Peri<'d, AnyPin>>,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        let r = U::regs();
        let irq = U::Interrupt::IRQ;

        configure(r, config, cts.is_some());

        let this = Self::new_innerer(peri, txd, cts, tx_buffer);

        r.enable().write(|w| w.set_enable(vals::Enable::Enabled));
        irq.pend();
        unsafe { irq.enable() };

        U::state().tx_rx_refcount.store(1, Ordering::Relaxed);

        this
    }

    fn new_innerer<U: UarteInstance>(
        _peri: Peri<'d, U>,
        txd: Peri<'d, AnyPin>,
        cts: Option<Peri<'d, AnyPin>>,
        tx_buffer: &'d mut [u8],
    ) -> Self {
        let r = U::regs();

        configure_tx_pins(r, txd, cts);

        // Initialize state
        let s = U::buffered_state();
        s.tx_count.store(0, Ordering::Relaxed);
        let len = tx_buffer.len();
        unsafe { s.tx_buf.init(tx_buffer.as_mut_ptr(), len) };

        r.events_dma().tx().ready().write_value(0);

        // Enable interrupts
        r.intenset().write(|w| {
            w.set_dmatxend(true);
        });

        Self {
            r,
            irq: U::Interrupt::IRQ,
            state: U::state(),
            buffered_state: s,
            is_borrowed: false,
            _p: PhantomData,
        }
    }

    /// Write a buffer into this writer, returning how many bytes were written.
    pub fn write<'a>(&'a mut self, buf: &'a [u8]) -> impl Future<Output = Result<usize, Error>> + 'a + use<'a, 'd> {
        poll_fn(move |cx| {
            //trace!("poll_write: {:?}", buf.len());
            let ss = self.state;
            let s = self.buffered_state;
            let mut tx = unsafe { s.tx_buf.writer() };

            let tx_buf = tx.push_slice();
            if tx_buf.is_empty() {
                //trace!("poll_write: pending");
                ss.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            let n = min(tx_buf.len(), buf.len());
            tx_buf[..n].copy_from_slice(&buf[..n]);
            tx.push_done(n);

            //trace!("poll_write: queued {:?}", n);

            compiler_fence(Ordering::SeqCst);
            self.irq.pend();

            Poll::Ready(Ok(n))
        })
    }

    /// Try writing a buffer without waiting, returning how many bytes were written.
    pub fn try_write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        //trace!("poll_write: {:?}", buf.len());
        let s = self.buffered_state;
        let mut tx = unsafe { s.tx_buf.writer() };

        let tx_buf = tx.push_slice();
        if tx_buf.is_empty() {
            return Ok(0);
        }

        let n = min(tx_buf.len(), buf.len());
        tx_buf[..n].copy_from_slice(&buf[..n]);
        tx.push_done(n);

        //trace!("poll_write: queued {:?}", n);

        compiler_fence(Ordering::SeqCst);
        self.irq.pend();

        Ok(n)
    }

    /// Flush this output stream, ensuring that all intermediately buffered contents reach their destination.
    pub fn flush(&mut self) -> impl Future<Output = Result<(), Error>> + '_ {
        let ss = self.state;
        let s = self.buffered_state;
        poll_fn(move |cx| {
            //trace!("poll_flush");
            if !s.tx_buf.is_empty() {
                //trace!("poll_flush: pending");
                ss.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            Poll::Ready(Ok(()))
        })
    }

    /// Adjust the baud rate to the provided value.
    pub fn set_baudrate(&mut self, baudrate: Baudrate) {
        self.r.baudrate().write(|w| w.set_baudrate(baudrate));
    }
}

impl<'a> Drop for BufferedUarteTx<'a> {
    fn drop(&mut self) {
        if self.is_borrowed {
            return;
        }

        let r = self.r;

        r.intenclr().write(|w| {
            w.set_txdrdy(true);
            w.set_dmatxready(true);
            w.set_txstopped(true);
        });
        r.events_txstopped().write_value(0);
        r.tasks_dma().tx().stop().write_value(1);
        while r.events_txstopped().read() == 0 {}

        let s = self.buffered_state;
        unsafe { s.tx_buf.deinit() }

        let s = self.state;
        drop_tx_rx(r, s);
    }
}

/// Reader part of the buffered UARTE driver.
pub struct BufferedUarteRx<'d> {
    r: pac::uarte::Uarte,
    irq: interrupt::Interrupt,
    state: &'static crate::uarte::State,
    buffered_state: &'static State,
    /// Whether this half was borrowed via `split_ref`, in which case dropping it must not tear down the peripheral.
    is_borrowed: bool,
    _p: PhantomData<&'d ()>,
}

impl<'d> BufferedUarteRx<'d> {
    /// Create a new BufferedUarteRx without hardware flow control.
    pub fn new<U: UarteInstance>(
        uarte: Peri<'d, U>,
        rxd: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        Self::new_inner(uarte, rxd.into(), None, rx_buffer, config)
    }

    /// Create a new BufferedUarteRx with hardware flow control (RTS)
    pub fn new_with_rts<U: UarteInstance>(
        uarte: Peri<'d, U>,
        rxd: Peri<'d, impl GpioPin>,
        rts: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        Self::new_inner(uarte, rxd.into(), Some(rts.into()), rx_buffer, config)
    }

    fn new_inner<U: UarteInstance>(
        peri: Peri<'d, U>,
        rxd: Peri<'d, AnyPin>,
        rts: Option<Peri<'d, AnyPin>>,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        let r = U::regs();
        let irq = U::Interrupt::IRQ;

        configure(r, config, rts.is_some());

        let this = Self::new_innerer(peri, rxd, rts, rx_buffer);

        r.enable().write(|w| w.set_enable(vals::Enable::Enabled));
        irq.pend();
        unsafe { irq.enable() };

        U::state().tx_rx_refcount.store(1, Ordering::Relaxed);

        this
    }

    fn new_innerer<U: UarteInstance>(
        _peri: Peri<'d, U>,
        rxd: Peri<'d, AnyPin>,
        rts: Option<Peri<'d, AnyPin>>,
        rx_buffer: &'d mut [u8],
    ) -> Self {
        let r = U::regs();

        configure_rx_pins(r, rxd, rts);

        // Initialize state
        let s = U::buffered_state();
        let rx_len = rx_buffer.len().min(DMA_SIZE * 2);
        let rx_ptr = rx_buffer.as_mut_ptr();
        unsafe { s.rx_buf.init(rx_ptr, rx_len) };
        s.rx_started.store(false, Ordering::Relaxed);

        // clear errors
        let errors = r.errorsrc().read();
        r.errorsrc().write_value(errors);

        r.events_error().write_value(0);
        r.events_dma().rx().end().write_value(0);

        // set timeout-to-stop short
        r.shorts().write(|w| {
            w.set_frametimeout_dma_rx_stop(true);
        });

        // set default timeout
        r.frametimeout().write_value(pac::uarte::regs::Frametimeout(0x10));

        // Enable interrupts
        r.intenset().write(|w| {
            w.set_dmatxend(true);
            w.set_error(true);
            w.set_dmarxend(true);
        });

        Self {
            r,
            irq: U::Interrupt::IRQ,
            state: U::state(),
            buffered_state: s,
            is_borrowed: false,
            _p: PhantomData,
        }
    }

    /// Pull some bytes from this source into the specified buffer, returning how many bytes were read.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let data = self.fill_buf().await?;
        let n = data.len().min(buf.len());
        buf[..n].copy_from_slice(&data[..n]);
        self.consume(n);
        Ok(n)
    }

    /// Return the contents of the internal buffer, filling it with more data from the inner reader if it is empty.
    pub fn fill_buf(&mut self) -> impl Future<Output = Result<&'_ [u8], Error>> {
        let s = self.buffered_state;
        let ss = self.state;
        poll_fn(move |cx| {
            compiler_fence(Ordering::SeqCst);
            //trace!("poll_read");

            let mut rx = unsafe { s.rx_buf.reader() };

            let (ptr, n) = rx.pop_buf();
            if n == 0 {
                //trace!("  empty");
                ss.rx_waker.register(cx.waker());
                Poll::Pending
            } else {
                Poll::Ready(Ok(unsafe { slice::from_raw_parts(ptr, n) }))
            }
        })
    }

    /// Tell this buffer that `amt` bytes have been consumed from the buffer, so they should no longer be returned in calls to `fill_buf`.
    pub fn consume(&mut self, amt: usize) {
        if amt == 0 {
            return;
        }

        let s = self.buffered_state;
        let mut rx = unsafe { s.rx_buf.reader() };
        rx.pop_done(amt);

        // If the DMA is stopped because the buffer was full, restart it now that there's space.
        if !s.rx_started.load(Ordering::Relaxed) {
            self.irq.pend();
        }
    }

    /// we are ready to read if there is data in the buffer
    fn read_ready(&self) -> Result<bool, Error> {
        Ok(!self.buffered_state.rx_buf.is_empty())
    }
}

impl<'a> Drop for BufferedUarteRx<'a> {
    fn drop(&mut self) {
        if self.is_borrowed {
            return;
        }

        let r = self.r;

        r.intenclr().write(|w| {
            w.set_rxto(true);
        });
        r.events_rxto().write_value(0);

        let s = self.buffered_state;
        unsafe { s.rx_buf.deinit() }

        let s = self.state;
        drop_tx_rx(r, s);
    }
}

mod _embedded_io {
    use super::*;

    impl embedded_io_async::Error for Error {
        fn kind(&self) -> embedded_io_async::ErrorKind {
            match *self {}
        }
    }

    impl<'d> embedded_io_async::ErrorType for BufferedUarte<'d> {
        type Error = Error;
    }

    impl<'d> embedded_io_async::ErrorType for BufferedUarteRx<'d> {
        type Error = Error;
    }

    impl<'d> embedded_io_async::ErrorType for BufferedUarteTx<'d> {
        type Error = Error;
    }

    impl<'d> embedded_io_async::Read for BufferedUarte<'d> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.read(buf).await
        }
    }

    impl<'d> embedded_io_async::Read for BufferedUarteRx<'d> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.read(buf).await
        }
    }

    impl<'d> embedded_io_async::ReadReady for BufferedUarte<'d> {
        fn read_ready(&mut self) -> Result<bool, Self::Error> {
            self.rx.read_ready()
        }
    }

    impl<'d> embedded_io_async::ReadReady for BufferedUarteRx<'d> {
        fn read_ready(&mut self) -> Result<bool, Self::Error> {
            BufferedUarteRx::read_ready(self)
        }
    }

    impl<'d> embedded_io_async::BufRead for BufferedUarte<'d> {
        async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
            self.fill_buf().await
        }

        fn consume(&mut self, amt: usize) {
            self.consume(amt)
        }
    }

    impl<'d> embedded_io_async::BufRead for BufferedUarteRx<'d> {
        async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
            self.fill_buf().await
        }

        fn consume(&mut self, amt: usize) {
            self.consume(amt)
        }
    }

    impl<'d> embedded_io_async::Write for BufferedUarte<'d> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.write(buf).await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.flush().await
        }
    }

    impl<'d> embedded_io_async::Write for BufferedUarteTx<'d> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.write(buf).await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.flush().await
        }
    }
}
