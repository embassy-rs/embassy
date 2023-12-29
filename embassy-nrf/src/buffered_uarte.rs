//! Async buffered UART driver.
//!
//! Note that discarding a future from a read or write operation may lead to losing
//! data. For example, when using `futures_util::future::select` and completion occurs
//! on the "other" future, you should capture the incomplete future and continue to use
//! it for the next read or write. This pattern is a consideration for all IO, and not
//! just serial communications.
//!
//! Please also see [crate::uarte] to understand when [BufferedUarte] should be used.

use core::cmp::min;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::slice;
use core::sync::atomic::{compiler_fence, AtomicBool, AtomicU8, AtomicUsize, Ordering};
use core::task::Poll;

use embassy_hal_internal::atomic_ring_buffer::RingBuffer;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
// Re-export SVD variants to allow user to directly set values
pub use pac::uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

use crate::gpio::sealed::Pin;
use crate::gpio::{self, AnyPin, Pin as GpioPin, PselBits};
use crate::interrupt::typelevel::Interrupt;
use crate::ppi::{
    self, AnyConfigurableChannel, AnyGroup, Channel, ConfigurableChannel, Event, Group, Ppi, PpiGroup, Task,
};
use crate::timer::{Instance as TimerInstance, Timer};
use crate::uarte::{apply_workaround_for_enable_anomaly, Config, Instance as UarteInstance};
use crate::{interrupt, pac, Peripheral};

mod sealed {
    use super::*;

    pub struct State {
        pub tx_waker: AtomicWaker,
        pub tx_buf: RingBuffer,
        pub tx_count: AtomicUsize,

        pub rx_waker: AtomicWaker,
        pub rx_buf: RingBuffer,
        pub rx_started: AtomicBool,
        pub rx_started_count: AtomicU8,
        pub rx_ended_count: AtomicU8,
        pub rx_ppi_ch: AtomicU8,
    }
}

/// UART error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    // No errors for now
}

pub(crate) use sealed::State;

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            tx_waker: AtomicWaker::new(),
            tx_buf: RingBuffer::new(),
            tx_count: AtomicUsize::new(0),

            rx_waker: AtomicWaker::new(),
            rx_buf: RingBuffer::new(),
            rx_started: AtomicBool::new(false),
            rx_started_count: AtomicU8::new(0),
            rx_ended_count: AtomicU8::new(0),
            rx_ppi_ch: AtomicU8::new(0),
        }
    }
}

/// Interrupt handler.
pub struct InterruptHandler<U: UarteInstance> {
    _phantom: PhantomData<U>,
}

impl<U: UarteInstance> interrupt::typelevel::Handler<U::Interrupt> for InterruptHandler<U> {
    unsafe fn on_interrupt() {
        //trace!("irq: start");
        let r = U::regs();
        let s = U::buffered_state();

        let buf_len = s.rx_buf.len();
        let half_len = buf_len / 2;
        let mut tx = unsafe { s.tx_buf.reader() };
        let mut rx = unsafe { s.rx_buf.writer() };

        if r.events_error.read().bits() != 0 {
            r.events_error.reset();
            let errs = r.errorsrc.read();
            r.errorsrc.write(|w| unsafe { w.bits(errs.bits()) });

            if errs.overrun().bit() {
                panic!("BufferedUarte overrun");
            }
        }

        // Received some bytes, wake task.
        if r.inten.read().rxdrdy().bit_is_set() && r.events_rxdrdy.read().bits() != 0 {
            r.intenclr.write(|w| w.rxdrdy().clear());
            r.events_rxdrdy.reset();
            s.rx_waker.wake();
        }

        if r.events_endrx.read().bits() != 0 {
            //trace!("  irq_rx: endrx");
            r.events_endrx.reset();

            let val = s.rx_ended_count.load(Ordering::Relaxed);
            s.rx_ended_count.store(val.wrapping_add(1), Ordering::Relaxed);
        }

        if r.events_rxstarted.read().bits() != 0 || !s.rx_started.load(Ordering::Relaxed) {
            //trace!("  irq_rx: rxstarted");
            let (ptr, len) = rx.push_buf();
            if len >= half_len {
                r.events_rxstarted.reset();

                //trace!("  irq_rx: starting second {:?}", half_len);

                // Set up the DMA read
                r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
                r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(half_len as _) });

                let chn = s.rx_ppi_ch.load(Ordering::Relaxed);

                // Enable endrx -> startrx PPI channel.
                // From this point on, if endrx happens, startrx is automatically fired.
                ppi::regs().chenset.write(|w| unsafe { w.bits(1 << chn) });

                // It is possible that endrx happened BEFORE enabling the PPI. In this case
                // the PPI channel doesn't trigger, and we'd hang. We have to detect this
                // and manually start.

                // check again in case endrx has happened between the last check and now.
                if r.events_endrx.read().bits() != 0 {
                    //trace!("  irq_rx: endrx");
                    r.events_endrx.reset();

                    let val = s.rx_ended_count.load(Ordering::Relaxed);
                    s.rx_ended_count.store(val.wrapping_add(1), Ordering::Relaxed);
                }

                let rx_ended = s.rx_ended_count.load(Ordering::Relaxed);
                let rx_started = s.rx_started_count.load(Ordering::Relaxed);

                // If we started the same amount of transfers as ended, the last rxend has
                // already occured.
                let rxend_happened = rx_started == rx_ended;

                // Check if the PPI channel is still enabled. The PPI channel disables itself
                // when it fires, so if it's still enabled it hasn't fired.
                let ppi_ch_enabled = ppi::regs().chen.read().bits() & (1 << chn) != 0;

                // if rxend happened, and the ppi channel hasn't fired yet, the rxend got missed.
                // this condition also naturally matches if `!started`, needed to kickstart the DMA.
                if rxend_happened && ppi_ch_enabled {
                    //trace!("manually starting.");

                    // disable the ppi ch, it's of no use anymore.
                    ppi::regs().chenclr.write(|w| unsafe { w.bits(1 << chn) });

                    // manually start
                    r.tasks_startrx.write(|w| unsafe { w.bits(1) });
                }

                rx.push_done(half_len);

                s.rx_started_count.store(rx_started.wrapping_add(1), Ordering::Relaxed);
                s.rx_started.store(true, Ordering::Relaxed);
            } else {
                //trace!("  irq_rx: rxstarted no buf");
                r.intenclr.write(|w| w.rxstarted().clear());
            }
        }

        // =============================

        // TX end
        if r.events_endtx.read().bits() != 0 {
            r.events_endtx.reset();

            let n = s.tx_count.load(Ordering::Relaxed);
            //trace!("  irq_tx: endtx {:?}", n);
            tx.pop_done(n);
            s.tx_waker.wake();
            s.tx_count.store(0, Ordering::Relaxed);
        }

        // If not TXing, start.
        if s.tx_count.load(Ordering::Relaxed) == 0 {
            let (ptr, len) = tx.pop_buf();
            if len != 0 {
                //trace!("  irq_tx: starting {:?}", len);
                s.tx_count.store(len, Ordering::Relaxed);

                // Set up the DMA write
                r.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
                r.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

                // Start UARTE Transmit transaction
                r.tasks_starttx.write(|w| unsafe { w.bits(1) });
            }
        }

        //trace!("irq: end");
    }
}

/// Buffered UARTE driver.
pub struct BufferedUarte<'d, U: UarteInstance, T: TimerInstance> {
    _peri: PeripheralRef<'d, U>,
    timer: Timer<'d, T>,
    _ppi_ch1: Ppi<'d, AnyConfigurableChannel, 1, 1>,
    _ppi_ch2: Ppi<'d, AnyConfigurableChannel, 1, 2>,
    _ppi_group: PpiGroup<'d, AnyGroup>,
}

impl<'d, U: UarteInstance, T: TimerInstance> Unpin for BufferedUarte<'d, U, T> {}

impl<'d, U: UarteInstance, T: TimerInstance> BufferedUarte<'d, U, T> {
    /// Create a new BufferedUarte without hardware flow control.
    ///
    /// # Panics
    ///
    /// Panics if `rx_buffer.len()` is odd.
    pub fn new(
        uarte: impl Peripheral<P = U> + 'd,
        timer: impl Peripheral<P = T> + 'd,
        ppi_ch1: impl Peripheral<P = impl ConfigurableChannel> + 'd,
        ppi_ch2: impl Peripheral<P = impl ConfigurableChannel> + 'd,
        ppi_group: impl Peripheral<P = impl Group> + 'd,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        rxd: impl Peripheral<P = impl GpioPin> + 'd,
        txd: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
        rx_buffer: &'d mut [u8],
        tx_buffer: &'d mut [u8],
    ) -> Self {
        into_ref!(rxd, txd, ppi_ch1, ppi_ch2, ppi_group);
        Self::new_inner(
            uarte,
            timer,
            ppi_ch1.map_into(),
            ppi_ch2.map_into(),
            ppi_group.map_into(),
            rxd.map_into(),
            txd.map_into(),
            None,
            None,
            config,
            rx_buffer,
            tx_buffer,
        )
    }

    /// Create a new BufferedUarte with hardware flow control (RTS/CTS)
    ///
    /// # Panics
    ///
    /// Panics if `rx_buffer.len()` is odd.
    pub fn new_with_rtscts(
        uarte: impl Peripheral<P = U> + 'd,
        timer: impl Peripheral<P = T> + 'd,
        ppi_ch1: impl Peripheral<P = impl ConfigurableChannel> + 'd,
        ppi_ch2: impl Peripheral<P = impl ConfigurableChannel> + 'd,
        ppi_group: impl Peripheral<P = impl Group> + 'd,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        rxd: impl Peripheral<P = impl GpioPin> + 'd,
        txd: impl Peripheral<P = impl GpioPin> + 'd,
        cts: impl Peripheral<P = impl GpioPin> + 'd,
        rts: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
        rx_buffer: &'d mut [u8],
        tx_buffer: &'d mut [u8],
    ) -> Self {
        into_ref!(rxd, txd, cts, rts, ppi_ch1, ppi_ch2, ppi_group);
        Self::new_inner(
            uarte,
            timer,
            ppi_ch1.map_into(),
            ppi_ch2.map_into(),
            ppi_group.map_into(),
            rxd.map_into(),
            txd.map_into(),
            Some(cts.map_into()),
            Some(rts.map_into()),
            config,
            rx_buffer,
            tx_buffer,
        )
    }

    fn new_inner(
        peri: impl Peripheral<P = U> + 'd,
        timer: impl Peripheral<P = T> + 'd,
        ppi_ch1: PeripheralRef<'d, AnyConfigurableChannel>,
        ppi_ch2: PeripheralRef<'d, AnyConfigurableChannel>,
        ppi_group: PeripheralRef<'d, AnyGroup>,
        rxd: PeripheralRef<'d, AnyPin>,
        txd: PeripheralRef<'d, AnyPin>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
        rx_buffer: &'d mut [u8],
        tx_buffer: &'d mut [u8],
    ) -> Self {
        into_ref!(peri, timer);

        assert!(rx_buffer.len() % 2 == 0);

        let r = U::regs();

        let hwfc = cts.is_some();

        rxd.conf().write(|w| w.input().connect().drive().h0h1());
        r.psel.rxd.write(|w| unsafe { w.bits(rxd.psel_bits()) });

        txd.set_high();
        txd.conf().write(|w| w.dir().output().drive().h0h1());
        r.psel.txd.write(|w| unsafe { w.bits(txd.psel_bits()) });

        if let Some(pin) = &cts {
            pin.conf().write(|w| w.input().connect().drive().h0h1());
        }
        r.psel.cts.write(|w| unsafe { w.bits(cts.psel_bits()) });

        if let Some(pin) = &rts {
            pin.set_high();
            pin.conf().write(|w| w.dir().output().drive().h0h1());
        }
        r.psel.rts.write(|w| unsafe { w.bits(rts.psel_bits()) });

        // Initialize state
        let s = U::buffered_state();
        s.tx_count.store(0, Ordering::Relaxed);
        s.rx_started_count.store(0, Ordering::Relaxed);
        s.rx_ended_count.store(0, Ordering::Relaxed);
        s.rx_started.store(false, Ordering::Relaxed);
        let len = tx_buffer.len();
        unsafe { s.tx_buf.init(tx_buffer.as_mut_ptr(), len) };
        let len = rx_buffer.len();
        unsafe { s.rx_buf.init(rx_buffer.as_mut_ptr(), len) };

        // Configure
        r.config.write(|w| {
            w.hwfc().bit(hwfc);
            w.parity().variant(config.parity);
            w
        });
        r.baudrate.write(|w| w.baudrate().variant(config.baudrate));

        // clear errors
        let errors = r.errorsrc.read().bits();
        r.errorsrc.write(|w| unsafe { w.bits(errors) });

        r.events_rxstarted.reset();
        r.events_txstarted.reset();
        r.events_error.reset();
        r.events_endrx.reset();
        r.events_endtx.reset();

        // Enable interrupts
        r.intenclr.write(|w| unsafe { w.bits(!0) });
        r.intenset.write(|w| {
            w.endtx().set();
            w.rxstarted().set();
            w.error().set();
            w.endrx().set();
            w
        });

        // Enable UARTE instance
        apply_workaround_for_enable_anomaly(&r);
        r.enable.write(|w| w.enable().enabled());

        // Configure byte counter.
        let timer = Timer::new_counter(timer);
        timer.cc(1).write(rx_buffer.len() as u32 * 2);
        timer.cc(1).short_compare_clear();
        timer.clear();
        timer.start();

        let mut ppi_ch1 = Ppi::new_one_to_one(ppi_ch1, Event::from_reg(&r.events_rxdrdy), timer.task_count());
        ppi_ch1.enable();

        s.rx_ppi_ch.store(ppi_ch2.number() as u8, Ordering::Relaxed);
        let mut ppi_group = PpiGroup::new(ppi_group);
        let mut ppi_ch2 = Ppi::new_one_to_two(
            ppi_ch2,
            Event::from_reg(&r.events_endrx),
            Task::from_reg(&r.tasks_startrx),
            ppi_group.task_disable_all(),
        );
        ppi_ch2.disable();
        ppi_group.add_channel(&ppi_ch2);

        U::Interrupt::pend();
        unsafe { U::Interrupt::enable() };

        Self {
            _peri: peri,
            timer,
            _ppi_ch1: ppi_ch1,
            _ppi_ch2: ppi_ch2,
            _ppi_group: ppi_group,
        }
    }

    fn pend_irq() {
        U::Interrupt::pend()
    }

    /// Adjust the baud rate to the provided value.
    pub fn set_baudrate(&mut self, baudrate: Baudrate) {
        let r = U::regs();
        r.baudrate.write(|w| w.baudrate().variant(baudrate));
    }

    /// Split the UART in reader and writer parts.
    ///
    /// This allows reading and writing concurrently from independent tasks.
    pub fn split<'u>(&'u mut self) -> (BufferedUarteRx<'u, 'd, U, T>, BufferedUarteTx<'u, 'd, U, T>) {
        (BufferedUarteRx { inner: self }, BufferedUarteTx { inner: self })
    }

    async fn inner_read(&self, buf: &mut [u8]) -> Result<usize, Error> {
        let data = self.inner_fill_buf().await?;
        let n = data.len().min(buf.len());
        buf[..n].copy_from_slice(&data[..n]);
        self.inner_consume(n);
        Ok(n)
    }

    async fn inner_write<'a>(&'a self, buf: &'a [u8]) -> Result<usize, Error> {
        poll_fn(move |cx| {
            //trace!("poll_write: {:?}", buf.len());
            let s = U::buffered_state();
            let mut tx = unsafe { s.tx_buf.writer() };

            let tx_buf = tx.push_slice();
            if tx_buf.is_empty() {
                //trace!("poll_write: pending");
                s.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            let n = min(tx_buf.len(), buf.len());
            tx_buf[..n].copy_from_slice(&buf[..n]);
            tx.push_done(n);

            //trace!("poll_write: queued {:?}", n);

            compiler_fence(Ordering::SeqCst);
            Self::pend_irq();

            Poll::Ready(Ok(n))
        })
        .await
    }

    async fn inner_flush<'a>(&'a self) -> Result<(), Error> {
        poll_fn(move |cx| {
            //trace!("poll_flush");
            let s = U::buffered_state();
            if !s.tx_buf.is_empty() {
                //trace!("poll_flush: pending");
                s.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            Poll::Ready(Ok(()))
        })
        .await
    }

    async fn inner_fill_buf<'a>(&'a self) -> Result<&'a [u8], Error> {
        poll_fn(move |cx| {
            compiler_fence(Ordering::SeqCst);
            //trace!("poll_read");

            let r = U::regs();
            let s = U::buffered_state();

            // Read the RXDRDY counter.
            T::regs().tasks_capture[0].write(|w| unsafe { w.bits(1) });
            let mut end = T::regs().cc[0].read().bits() as usize;
            //trace!("  rxdrdy count = {:?}", end);

            // We've set a compare channel that resets the counter to 0 when it reaches `len*2`.
            // However, it's unclear if that's instant, or there's a small window where you can
            // still read `len()*2`.
            // This could happen if in one clock cycle the counter is updated, and in the next the
            // clear takes effect. The docs are very sparse, they just say "Task delays: After TIMER
            // is started, the CLEAR, COUNT, and STOP tasks are guaranteed to take effect within one
            // clock cycle of the PCLK16M." :shrug:
            // So, we wrap the counter ourselves, just in case.
            if end > s.rx_buf.len() * 2 {
                end = 0
            }

            // This logic mirrors `atomic_ring_buffer::Reader::pop_buf()`
            let mut start = s.rx_buf.start.load(Ordering::Relaxed);
            let len = s.rx_buf.len();
            if start == end {
                //trace!("  empty");
                s.rx_waker.register(cx.waker());
                r.intenset.write(|w| w.rxdrdy().set_bit());
                return Poll::Pending;
            }

            if start >= len {
                start -= len
            }
            if end >= len {
                end -= len
            }

            let n = if end > start { end - start } else { len - start };
            assert!(n != 0);
            //trace!("  uarte ringbuf: pop_buf {:?}..{:?}", start, start + n);

            let buf = s.rx_buf.buf.load(Ordering::Relaxed);
            Poll::Ready(Ok(unsafe { slice::from_raw_parts(buf.add(start), n) }))
        })
        .await
    }

    fn inner_consume(&self, amt: usize) {
        if amt == 0 {
            return;
        }

        let s = U::buffered_state();
        let mut rx = unsafe { s.rx_buf.reader() };
        rx.pop_done(amt);
        U::regs().intenset.write(|w| w.rxstarted().set());
    }

    /// Pull some bytes from this source into the specified buffer, returning how many bytes were read.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.inner_read(buf).await
    }

    /// Return the contents of the internal buffer, filling it with more data from the inner reader if it is empty.
    pub async fn fill_buf(&mut self) -> Result<&[u8], Error> {
        self.inner_fill_buf().await
    }

    /// Tell this buffer that `amt` bytes have been consumed from the buffer, so they should no longer be returned in calls to `fill_buf`.
    pub fn consume(&mut self, amt: usize) {
        self.inner_consume(amt)
    }

    /// Write a buffer into this writer, returning how many bytes were written.
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.inner_write(buf).await
    }

    /// Flush this output stream, ensuring that all intermediately buffered contents reach their destination.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.inner_flush().await
    }
}

/// Reader part of the buffered UARTE driver.
pub struct BufferedUarteTx<'u, 'd, U: UarteInstance, T: TimerInstance> {
    inner: &'u BufferedUarte<'d, U, T>,
}

impl<'u, 'd, U: UarteInstance, T: TimerInstance> BufferedUarteTx<'u, 'd, U, T> {
    /// Write a buffer into this writer, returning how many bytes were written.
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.inner.inner_write(buf).await
    }

    /// Flush this output stream, ensuring that all intermediately buffered contents reach their destination.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.inner.inner_flush().await
    }
}

/// Writer part of the buffered UARTE driver.
pub struct BufferedUarteRx<'u, 'd, U: UarteInstance, T: TimerInstance> {
    inner: &'u BufferedUarte<'d, U, T>,
}

impl<'u, 'd, U: UarteInstance, T: TimerInstance> BufferedUarteRx<'u, 'd, U, T> {
    /// Pull some bytes from this source into the specified buffer, returning how many bytes were read.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.inner.inner_read(buf).await
    }

    /// Return the contents of the internal buffer, filling it with more data from the inner reader if it is empty.
    pub async fn fill_buf(&mut self) -> Result<&[u8], Error> {
        self.inner.inner_fill_buf().await
    }

    /// Tell this buffer that `amt` bytes have been consumed from the buffer, so they should no longer be returned in calls to `fill_buf`.
    pub fn consume(&mut self, amt: usize) {
        self.inner.inner_consume(amt)
    }
}

mod _embedded_io {
    use super::*;

    impl embedded_io_async::Error for Error {
        fn kind(&self) -> embedded_io_async::ErrorKind {
            match *self {}
        }
    }

    impl<'d, U: UarteInstance, T: TimerInstance> embedded_io_async::ErrorType for BufferedUarte<'d, U, T> {
        type Error = Error;
    }

    impl<'u, 'd, U: UarteInstance, T: TimerInstance> embedded_io_async::ErrorType for BufferedUarteRx<'u, 'd, U, T> {
        type Error = Error;
    }

    impl<'u, 'd, U: UarteInstance, T: TimerInstance> embedded_io_async::ErrorType for BufferedUarteTx<'u, 'd, U, T> {
        type Error = Error;
    }

    impl<'d, U: UarteInstance, T: TimerInstance> embedded_io_async::Read for BufferedUarte<'d, U, T> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.inner_read(buf).await
        }
    }

    impl<'u, 'd: 'u, U: UarteInstance, T: TimerInstance> embedded_io_async::Read for BufferedUarteRx<'u, 'd, U, T> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.inner.inner_read(buf).await
        }
    }

    impl<'d, U: UarteInstance, T: TimerInstance> embedded_io_async::BufRead for BufferedUarte<'d, U, T> {
        async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
            self.inner_fill_buf().await
        }

        fn consume(&mut self, amt: usize) {
            self.inner_consume(amt)
        }
    }

    impl<'u, 'd: 'u, U: UarteInstance, T: TimerInstance> embedded_io_async::BufRead for BufferedUarteRx<'u, 'd, U, T> {
        async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
            self.inner.inner_fill_buf().await
        }

        fn consume(&mut self, amt: usize) {
            self.inner.inner_consume(amt)
        }
    }

    impl<'d, U: UarteInstance, T: TimerInstance> embedded_io_async::Write for BufferedUarte<'d, U, T> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.inner_write(buf).await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.inner_flush().await
        }
    }

    impl<'u, 'd: 'u, U: UarteInstance, T: TimerInstance> embedded_io_async::Write for BufferedUarteTx<'u, 'd, U, T> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.inner.inner_write(buf).await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.inner.inner_flush().await
        }
    }
}

impl<'a, U: UarteInstance, T: TimerInstance> Drop for BufferedUarte<'a, U, T> {
    fn drop(&mut self) {
        self._ppi_group.disable_all();

        let r = U::regs();

        self.timer.stop();

        r.inten.reset();
        r.events_rxto.reset();
        r.tasks_stoprx.write(|w| unsafe { w.bits(1) });
        r.events_txstopped.reset();
        r.tasks_stoptx.write(|w| unsafe { w.bits(1) });

        while r.events_txstopped.read().bits() == 0 {}
        while r.events_rxto.read().bits() == 0 {}

        r.enable.write(|w| w.enable().disabled());

        gpio::deconfigure_pin(r.psel.rxd.read().bits());
        gpio::deconfigure_pin(r.psel.txd.read().bits());
        gpio::deconfigure_pin(r.psel.rts.read().bits());
        gpio::deconfigure_pin(r.psel.cts.read().bits());

        let s = U::buffered_state();
        unsafe {
            s.rx_buf.deinit();
            s.tx_buf.deinit();
        }
    }
}
