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
use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::slice;
use core::sync::atomic::{compiler_fence, AtomicBool, AtomicU8, AtomicUsize, Ordering};
use core::task::Poll;

use embassy_hal_internal::atomic_ring_buffer::RingBuffer;
use embassy_hal_internal::Peri;
use pac::uarte::vals;
// Re-export SVD variants to allow user to directly set values
pub use pac::uarte::vals::{Baudrate, ConfigParity as Parity};

use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::interrupt::typelevel::Interrupt;
use crate::ppi::{
    self, AnyConfigurableChannel, AnyGroup, Channel, ConfigurableChannel, Event, Group, Ppi, PpiGroup, Task,
};
use crate::timer::{Instance as TimerInstance, Timer};
use crate::uarte::{configure, configure_rx_pins, configure_tx_pins, drop_tx_rx, Config, Instance as UarteInstance};
use crate::{interrupt, pac, EASY_DMA_SIZE};

pub(crate) struct State {
    tx_buf: RingBuffer,
    tx_count: AtomicUsize,

    rx_buf: RingBuffer,
    rx_started: AtomicBool,
    rx_started_count: AtomicU8,
    rx_ended_count: AtomicU8,
    rx_ppi_ch: AtomicU8,
}

/// UART error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
                    panic!("BufferedUarte overrun");
                }
            }

            // Received some bytes, wake task.
            if r.inten().read().rxdrdy() && r.events_rxdrdy().read() != 0 {
                r.intenclr().write(|w| w.set_rxdrdy(true));
                r.events_rxdrdy().write_value(0);
                ss.rx_waker.wake();
            }

            if r.events_endrx().read() != 0 {
                //trace!("  irq_rx: endrx");
                r.events_endrx().write_value(0);

                let val = s.rx_ended_count.load(Ordering::Relaxed);
                s.rx_ended_count.store(val.wrapping_add(1), Ordering::Relaxed);
            }

            if r.events_rxstarted().read() != 0 || !s.rx_started.load(Ordering::Relaxed) {
                //trace!("  irq_rx: rxstarted");
                let (ptr, len) = rx.push_buf();
                if len >= half_len {
                    r.events_rxstarted().write_value(0);

                    //trace!("  irq_rx: starting second {:?}", half_len);

                    // Set up the DMA read
                    r.rxd().ptr().write_value(ptr as u32);
                    r.rxd().maxcnt().write(|w| w.set_maxcnt(half_len as _));

                    let chn = s.rx_ppi_ch.load(Ordering::Relaxed);

                    // Enable endrx -> startrx PPI channel.
                    // From this point on, if endrx happens, startrx is automatically fired.
                    ppi::regs().chenset().write(|w| w.0 = 1 << chn);

                    // It is possible that endrx happened BEFORE enabling the PPI. In this case
                    // the PPI channel doesn't trigger, and we'd hang. We have to detect this
                    // and manually start.

                    // check again in case endrx has happened between the last check and now.
                    if r.events_endrx().read() != 0 {
                        //trace!("  irq_rx: endrx");
                        r.events_endrx().write_value(0);

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
                    let ppi_ch_enabled = ppi::regs().chen().read().ch(chn as _);

                    // if rxend happened, and the ppi channel hasn't fired yet, the rxend got missed.
                    // this condition also naturally matches if `!started`, needed to kickstart the DMA.
                    if rxend_happened && ppi_ch_enabled {
                        //trace!("manually starting.");

                        // disable the ppi ch, it's of no use anymore.
                        ppi::regs().chenclr().write(|w| w.set_ch(chn as _, true));

                        // manually start
                        r.tasks_startrx().write_value(1);
                    }

                    rx.push_done(half_len);

                    s.rx_started_count.store(rx_started.wrapping_add(1), Ordering::Relaxed);
                    s.rx_started.store(true, Ordering::Relaxed);
                } else {
                    //trace!("  irq_rx: rxstarted no buf");
                    r.intenclr().write(|w| w.set_rxstarted(true));
                }
            }
        }

        // =============================

        if let Some(mut tx) = unsafe { s.tx_buf.try_reader() } {
            // TX end
            if r.events_endtx().read() != 0 {
                r.events_endtx().write_value(0);

                let n = s.tx_count.load(Ordering::Relaxed);
                //trace!("  irq_tx: endtx {:?}", n);
                tx.pop_done(n);
                ss.tx_waker.wake();
                s.tx_count.store(0, Ordering::Relaxed);
            }

            // If not TXing, start.
            if s.tx_count.load(Ordering::Relaxed) == 0 {
                let (ptr, len) = tx.pop_buf();
                let len = len.min(EASY_DMA_SIZE);
                if len != 0 {
                    //trace!("  irq_tx: starting {:?}", len);
                    s.tx_count.store(len, Ordering::Relaxed);

                    // Set up the DMA write
                    r.txd().ptr().write_value(ptr as u32);
                    r.txd().maxcnt().write(|w| w.set_maxcnt(len as _));

                    // Start UARTE Transmit transaction
                    r.tasks_starttx().write_value(1);
                }
            }
        }

        //trace!("irq: end");
    }
}

/// Buffered UARTE driver.
pub struct BufferedUarte<'d, U: UarteInstance, T: TimerInstance> {
    tx: BufferedUarteTx<'d, U>,
    rx: BufferedUarteRx<'d, U, T>,
}

impl<'d, U: UarteInstance, T: TimerInstance> Unpin for BufferedUarte<'d, U, T> {}

impl<'d, U: UarteInstance, T: TimerInstance> BufferedUarte<'d, U, T> {
    /// Create a new BufferedUarte without hardware flow control.
    ///
    /// # Panics
    ///
    /// Panics if `rx_buffer.len()` is odd.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        uarte: Peri<'d, U>,
        timer: Peri<'d, T>,
        ppi_ch1: Peri<'d, impl ConfigurableChannel>,
        ppi_ch2: Peri<'d, impl ConfigurableChannel>,
        ppi_group: Peri<'d, impl Group>,
        rxd: Peri<'d, impl GpioPin>,
        txd: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        config: Config,
        rx_buffer: &'d mut [u8],
        tx_buffer: &'d mut [u8],
    ) -> Self {
        Self::new_inner(
            uarte,
            timer,
            ppi_ch1.into(),
            ppi_ch2.into(),
            ppi_group.into(),
            rxd.into(),
            txd.into(),
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
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_rtscts(
        uarte: Peri<'d, U>,
        timer: Peri<'d, T>,
        ppi_ch1: Peri<'d, impl ConfigurableChannel>,
        ppi_ch2: Peri<'d, impl ConfigurableChannel>,
        ppi_group: Peri<'d, impl Group>,
        rxd: Peri<'d, impl GpioPin>,
        txd: Peri<'d, impl GpioPin>,
        cts: Peri<'d, impl GpioPin>,
        rts: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        config: Config,
        rx_buffer: &'d mut [u8],
        tx_buffer: &'d mut [u8],
    ) -> Self {
        Self::new_inner(
            uarte,
            timer,
            ppi_ch1.into(),
            ppi_ch2.into(),
            ppi_group.into(),
            rxd.into(),
            txd.into(),
            Some(cts.into()),
            Some(rts.into()),
            config,
            rx_buffer,
            tx_buffer,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_inner(
        peri: Peri<'d, U>,
        timer: Peri<'d, T>,
        ppi_ch1: Peri<'d, AnyConfigurableChannel>,
        ppi_ch2: Peri<'d, AnyConfigurableChannel>,
        ppi_group: Peri<'d, AnyGroup>,
        rxd: Peri<'d, AnyPin>,
        txd: Peri<'d, AnyPin>,
        cts: Option<Peri<'d, AnyPin>>,
        rts: Option<Peri<'d, AnyPin>>,
        config: Config,
        rx_buffer: &'d mut [u8],
        tx_buffer: &'d mut [u8],
    ) -> Self {
        configure(U::regs(), config, cts.is_some());

        let tx = BufferedUarteTx::new_innerer(unsafe { peri.clone_unchecked() }, txd, cts, tx_buffer);
        let rx = BufferedUarteRx::new_innerer(peri, timer, ppi_ch1, ppi_ch2, ppi_group, rxd, rts, rx_buffer);

        U::regs().enable().write(|w| w.set_enable(vals::Enable::ENABLED));
        U::Interrupt::pend();
        unsafe { U::Interrupt::enable() };

        U::state().tx_rx_refcount.store(2, Ordering::Relaxed);

        Self { tx, rx }
    }

    /// Adjust the baud rate to the provided value.
    pub fn set_baudrate(&mut self, baudrate: Baudrate) {
        let r = U::regs();
        r.baudrate().write(|w| w.set_baudrate(baudrate));
    }

    /// Split the UART in reader and writer parts.
    ///
    /// This allows reading and writing concurrently from independent tasks.
    pub fn split(self) -> (BufferedUarteRx<'d, U, T>, BufferedUarteTx<'d, U>) {
        (self.rx, self.tx)
    }

    /// Split the UART in reader and writer parts, by reference.
    ///
    /// The returned halves borrow from `self`, so you can drop them and go back to using
    /// the "un-split" `self`. This allows temporarily splitting the UART.
    pub fn split_by_ref(&mut self) -> (&mut BufferedUarteRx<'d, U, T>, &mut BufferedUarteTx<'d, U>) {
        (&mut self.rx, &mut self.tx)
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

/// Reader part of the buffered UARTE driver.
pub struct BufferedUarteTx<'d, U: UarteInstance> {
    _peri: Peri<'d, U>,
}

impl<'d, U: UarteInstance> BufferedUarteTx<'d, U> {
    /// Create a new BufferedUarteTx without hardware flow control.
    pub fn new(
        uarte: Peri<'d, U>,
        txd: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        config: Config,
        tx_buffer: &'d mut [u8],
    ) -> Self {
        Self::new_inner(uarte, txd.into(), None, config, tx_buffer)
    }

    /// Create a new BufferedUarte with hardware flow control (RTS/CTS)
    ///
    /// # Panics
    ///
    /// Panics if `rx_buffer.len()` is odd.
    pub fn new_with_cts(
        uarte: Peri<'d, U>,
        txd: Peri<'d, impl GpioPin>,
        cts: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        config: Config,
        tx_buffer: &'d mut [u8],
    ) -> Self {
        Self::new_inner(uarte, txd.into(), Some(cts.into()), config, tx_buffer)
    }

    fn new_inner(
        peri: Peri<'d, U>,
        txd: Peri<'d, AnyPin>,
        cts: Option<Peri<'d, AnyPin>>,
        config: Config,
        tx_buffer: &'d mut [u8],
    ) -> Self {
        configure(U::regs(), config, cts.is_some());

        let this = Self::new_innerer(peri, txd, cts, tx_buffer);

        U::regs().enable().write(|w| w.set_enable(vals::Enable::ENABLED));
        U::Interrupt::pend();
        unsafe { U::Interrupt::enable() };

        U::state().tx_rx_refcount.store(1, Ordering::Relaxed);

        this
    }

    fn new_innerer(
        peri: Peri<'d, U>,
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

        r.events_txstarted().write_value(0);

        // Enable interrupts
        r.intenset().write(|w| {
            w.set_endtx(true);
        });

        Self { _peri: peri }
    }

    /// Write a buffer into this writer, returning how many bytes were written.
    pub fn write<'a>(&'a mut self, buf: &'a [u8]) -> impl Future<Output = Result<usize, Error>> + 'a {
        poll_fn(move |cx| {
            //trace!("poll_write: {:?}", buf.len());
            let ss = U::state();
            let s = U::buffered_state();
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
            U::Interrupt::pend();

            Poll::Ready(Ok(n))
        })
    }

    /// Try writing a buffer without waiting, returning how many bytes were written.
    pub fn try_write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        //trace!("poll_write: {:?}", buf.len());
        let s = U::buffered_state();
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
        U::Interrupt::pend();

        Ok(n)
    }

    /// Flush this output stream, ensuring that all intermediately buffered contents reach their destination.
    pub fn flush(&mut self) -> impl Future<Output = Result<(), Error>> + '_ {
        poll_fn(move |cx| {
            //trace!("poll_flush");
            let ss = U::state();
            let s = U::buffered_state();
            if !s.tx_buf.is_empty() {
                //trace!("poll_flush: pending");
                ss.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            Poll::Ready(Ok(()))
        })
    }
}

impl<'a, U: UarteInstance> Drop for BufferedUarteTx<'a, U> {
    fn drop(&mut self) {
        let r = U::regs();

        r.intenclr().write(|w| {
            w.set_txdrdy(true);
            w.set_txstarted(true);
            w.set_txstopped(true);
        });
        r.events_txstopped().write_value(0);
        r.tasks_stoptx().write_value(1);
        while r.events_txstopped().read() == 0 {}

        let s = U::buffered_state();
        unsafe { s.tx_buf.deinit() }

        let s = U::state();
        drop_tx_rx(r, s);
    }
}

/// Reader part of the buffered UARTE driver.
pub struct BufferedUarteRx<'d, U: UarteInstance, T: TimerInstance> {
    _peri: Peri<'d, U>,
    timer: Timer<'d, T>,
    _ppi_ch1: Ppi<'d, AnyConfigurableChannel, 1, 1>,
    _ppi_ch2: Ppi<'d, AnyConfigurableChannel, 1, 2>,
    _ppi_group: PpiGroup<'d, AnyGroup>,
}

impl<'d, U: UarteInstance, T: TimerInstance> BufferedUarteRx<'d, U, T> {
    /// Create a new BufferedUarte without hardware flow control.
    ///
    /// # Panics
    ///
    /// Panics if `rx_buffer.len()` is odd.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        uarte: Peri<'d, U>,
        timer: Peri<'d, T>,
        ppi_ch1: Peri<'d, impl ConfigurableChannel>,
        ppi_ch2: Peri<'d, impl ConfigurableChannel>,
        ppi_group: Peri<'d, impl Group>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        rxd: Peri<'d, impl GpioPin>,
        config: Config,
        rx_buffer: &'d mut [u8],
    ) -> Self {
        Self::new_inner(
            uarte,
            timer,
            ppi_ch1.into(),
            ppi_ch2.into(),
            ppi_group.into(),
            rxd.into(),
            None,
            config,
            rx_buffer,
        )
    }

    /// Create a new BufferedUarte with hardware flow control (RTS/CTS)
    ///
    /// # Panics
    ///
    /// Panics if `rx_buffer.len()` is odd.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_rts(
        uarte: Peri<'d, U>,
        timer: Peri<'d, T>,
        ppi_ch1: Peri<'d, impl ConfigurableChannel>,
        ppi_ch2: Peri<'d, impl ConfigurableChannel>,
        ppi_group: Peri<'d, impl Group>,
        rxd: Peri<'d, impl GpioPin>,
        rts: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        config: Config,
        rx_buffer: &'d mut [u8],
    ) -> Self {
        Self::new_inner(
            uarte,
            timer,
            ppi_ch1.into(),
            ppi_ch2.into(),
            ppi_group.into(),
            rxd.into(),
            Some(rts.into()),
            config,
            rx_buffer,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_inner(
        peri: Peri<'d, U>,
        timer: Peri<'d, T>,
        ppi_ch1: Peri<'d, AnyConfigurableChannel>,
        ppi_ch2: Peri<'d, AnyConfigurableChannel>,
        ppi_group: Peri<'d, AnyGroup>,
        rxd: Peri<'d, AnyPin>,
        rts: Option<Peri<'d, AnyPin>>,
        config: Config,
        rx_buffer: &'d mut [u8],
    ) -> Self {
        configure(U::regs(), config, rts.is_some());

        let this = Self::new_innerer(peri, timer, ppi_ch1, ppi_ch2, ppi_group, rxd, rts, rx_buffer);

        U::regs().enable().write(|w| w.set_enable(vals::Enable::ENABLED));
        U::Interrupt::pend();
        unsafe { U::Interrupt::enable() };

        U::state().tx_rx_refcount.store(1, Ordering::Relaxed);

        this
    }

    #[allow(clippy::too_many_arguments)]
    fn new_innerer(
        peri: Peri<'d, U>,
        timer: Peri<'d, T>,
        ppi_ch1: Peri<'d, AnyConfigurableChannel>,
        ppi_ch2: Peri<'d, AnyConfigurableChannel>,
        ppi_group: Peri<'d, AnyGroup>,
        rxd: Peri<'d, AnyPin>,
        rts: Option<Peri<'d, AnyPin>>,
        rx_buffer: &'d mut [u8],
    ) -> Self {
        assert!(rx_buffer.len() % 2 == 0);

        let r = U::regs();

        configure_rx_pins(r, rxd, rts);

        // Initialize state
        let s = U::buffered_state();
        s.rx_started_count.store(0, Ordering::Relaxed);
        s.rx_ended_count.store(0, Ordering::Relaxed);
        s.rx_started.store(false, Ordering::Relaxed);
        let rx_len = rx_buffer.len().min(EASY_DMA_SIZE * 2);
        unsafe { s.rx_buf.init(rx_buffer.as_mut_ptr(), rx_len) };

        // clear errors
        let errors = r.errorsrc().read();
        r.errorsrc().write_value(errors);

        r.events_rxstarted().write_value(0);
        r.events_error().write_value(0);
        r.events_endrx().write_value(0);

        // Enable interrupts
        r.intenset().write(|w| {
            w.set_endtx(true);
            w.set_rxstarted(true);
            w.set_error(true);
            w.set_endrx(true);
        });

        // Configure byte counter.
        let timer = Timer::new_counter(timer);
        timer.cc(1).write(rx_len as u32 * 2);
        timer.cc(1).short_compare_clear();
        timer.clear();
        timer.start();

        let mut ppi_ch1 = Ppi::new_one_to_one(ppi_ch1, Event::from_reg(r.events_rxdrdy()), timer.task_count());
        ppi_ch1.enable();

        s.rx_ppi_ch.store(ppi_ch2.number() as u8, Ordering::Relaxed);
        let mut ppi_group = PpiGroup::new(ppi_group);
        let mut ppi_ch2 = Ppi::new_one_to_two(
            ppi_ch2,
            Event::from_reg(r.events_endrx()),
            Task::from_reg(r.tasks_startrx()),
            ppi_group.task_disable_all(),
        );
        ppi_ch2.disable();
        ppi_group.add_channel(&ppi_ch2);

        Self {
            _peri: peri,
            timer,
            _ppi_ch1: ppi_ch1,
            _ppi_ch2: ppi_ch2,
            _ppi_group: ppi_group,
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
        poll_fn(move |cx| {
            compiler_fence(Ordering::SeqCst);
            //trace!("poll_read");

            let r = U::regs();
            let s = U::buffered_state();
            let ss = U::state();

            // Read the RXDRDY counter.
            T::regs().tasks_capture(0).write_value(1);
            let mut end = T::regs().cc(0).read() as usize;
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
                ss.rx_waker.register(cx.waker());
                r.intenset().write(|w| w.set_rxdrdy(true));
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
    }

    /// Tell this buffer that `amt` bytes have been consumed from the buffer, so they should no longer be returned in calls to `fill_buf`.
    pub fn consume(&mut self, amt: usize) {
        if amt == 0 {
            return;
        }

        let s = U::buffered_state();
        let mut rx = unsafe { s.rx_buf.reader() };
        rx.pop_done(amt);
        U::regs().intenset().write(|w| w.set_rxstarted(true));
    }

    /// we are ready to read if there is data in the buffer
    fn read_ready() -> Result<bool, Error> {
        let state = U::buffered_state();
        Ok(!state.rx_buf.is_empty())
    }
}

impl<'a, U: UarteInstance, T: TimerInstance> Drop for BufferedUarteRx<'a, U, T> {
    fn drop(&mut self) {
        self._ppi_group.disable_all();

        let r = U::regs();

        self.timer.stop();

        r.intenclr().write(|w| {
            w.set_rxdrdy(true);
            w.set_rxstarted(true);
            w.set_rxto(true);
        });
        r.events_rxto().write_value(0);
        r.tasks_stoprx().write_value(1);
        while r.events_rxto().read() == 0 {}

        let s = U::buffered_state();
        unsafe { s.rx_buf.deinit() }

        let s = U::state();
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

    impl<'d, U: UarteInstance, T: TimerInstance> embedded_io_async::ErrorType for BufferedUarte<'d, U, T> {
        type Error = Error;
    }

    impl<'d, U: UarteInstance, T: TimerInstance> embedded_io_async::ErrorType for BufferedUarteRx<'d, U, T> {
        type Error = Error;
    }

    impl<'d, U: UarteInstance> embedded_io_async::ErrorType for BufferedUarteTx<'d, U> {
        type Error = Error;
    }

    impl<'d, U: UarteInstance, T: TimerInstance> embedded_io_async::Read for BufferedUarte<'d, U, T> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.read(buf).await
        }
    }

    impl<'d: 'd, U: UarteInstance, T: TimerInstance> embedded_io_async::Read for BufferedUarteRx<'d, U, T> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.read(buf).await
        }
    }

    impl<'d, U: UarteInstance, T: TimerInstance + 'd> embedded_io_async::ReadReady for BufferedUarte<'d, U, T> {
        fn read_ready(&mut self) -> Result<bool, Self::Error> {
            BufferedUarteRx::<'d, U, T>::read_ready()
        }
    }

    impl<'d, U: UarteInstance, T: TimerInstance + 'd> embedded_io_async::ReadReady for BufferedUarteRx<'d, U, T> {
        fn read_ready(&mut self) -> Result<bool, Self::Error> {
            Self::read_ready()
        }
    }

    impl<'d, U: UarteInstance, T: TimerInstance> embedded_io_async::BufRead for BufferedUarte<'d, U, T> {
        async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
            self.fill_buf().await
        }

        fn consume(&mut self, amt: usize) {
            self.consume(amt)
        }
    }

    impl<'d: 'd, U: UarteInstance, T: TimerInstance> embedded_io_async::BufRead for BufferedUarteRx<'d, U, T> {
        async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
            self.fill_buf().await
        }

        fn consume(&mut self, amt: usize) {
            self.consume(amt)
        }
    }

    impl<'d, U: UarteInstance, T: TimerInstance> embedded_io_async::Write for BufferedUarte<'d, U, T> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.write(buf).await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.flush().await
        }
    }

    impl<'d: 'd, U: UarteInstance> embedded_io_async::Write for BufferedUarteTx<'d, U> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.write(buf).await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.flush().await
        }
    }
}
