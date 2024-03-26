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
// Re-export SVD variants to allow user to directly set values
pub use pac::uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

use crate::gpio::sealed::Pin;
use crate::gpio::{AnyPin, Pin as GpioPin, PselBits};
use crate::interrupt::typelevel::Interrupt;
use crate::ppi::{
    self, AnyConfigurableChannel, AnyGroup, Channel, ConfigurableChannel, Event, Group, Ppi, PpiGroup, Task,
};
use crate::timer::{Instance as TimerInstance, Timer};
use crate::uarte::{configure, drop_tx_rx, Config, Instance as UarteInstance};
use crate::{interrupt, pac, Peripheral};

mod sealed {
    use super::*;

    pub struct State {
        pub tx_buf: RingBuffer,
        pub tx_count: AtomicUsize,

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
                ss.rx_waker.wake();
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
        }

        // =============================

        if let Some(mut tx) = unsafe { s.tx_buf.try_reader() } {
            // TX end
            if r.events_endtx.read().bits() != 0 {
                r.events_endtx.reset();

                let n = s.tx_count.load(Ordering::Relaxed);
                //trace!("  irq_tx: endtx {:?}", n);
                tx.pop_done(n);
                ss.tx_waker.wake();
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
        into_ref!(uarte, timer, rxd, txd, ppi_ch1, ppi_ch2, ppi_group);
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
        into_ref!(uarte, timer, rxd, txd, cts, rts, ppi_ch1, ppi_ch2, ppi_group);
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
        peri: PeripheralRef<'d, U>,
        timer: PeripheralRef<'d, T>,
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
        configure(U::regs(), config, cts.is_some());

        let tx = BufferedUarteTx::new_innerer(unsafe { peri.clone_unchecked() }, txd, cts, tx_buffer);
        let rx = BufferedUarteRx::new_innerer(peri, timer, ppi_ch1, ppi_ch2, ppi_group, rxd, rts, rx_buffer);

        U::Interrupt::pend();
        unsafe { U::Interrupt::enable() };

        U::state().tx_rx_refcount.store(2, Ordering::Relaxed);

        Self { tx, rx }
    }

    /// Adjust the baud rate to the provided value.
    pub fn set_baudrate(&mut self, baudrate: Baudrate) {
        let r = U::regs();
        r.baudrate.write(|w| w.baudrate().variant(baudrate));
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

    /// Flush this output stream, ensuring that all intermediately buffered contents reach their destination.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.tx.flush().await
    }
}

/// Reader part of the buffered UARTE driver.
pub struct BufferedUarteTx<'d, U: UarteInstance> {
    _peri: PeripheralRef<'d, U>,
}

impl<'d, U: UarteInstance> BufferedUarteTx<'d, U> {
    /// Create a new BufferedUarteTx without hardware flow control.
    pub fn new(
        uarte: impl Peripheral<P = U> + 'd,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        txd: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
        tx_buffer: &'d mut [u8],
    ) -> Self {
        into_ref!(uarte, txd);
        Self::new_inner(uarte, txd.map_into(), None, config, tx_buffer)
    }

    /// Create a new BufferedUarte with hardware flow control (RTS/CTS)
    ///
    /// # Panics
    ///
    /// Panics if `rx_buffer.len()` is odd.
    pub fn new_with_cts(
        uarte: impl Peripheral<P = U> + 'd,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        txd: impl Peripheral<P = impl GpioPin> + 'd,
        cts: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
        tx_buffer: &'d mut [u8],
    ) -> Self {
        into_ref!(uarte, txd, cts);
        Self::new_inner(uarte, txd.map_into(), Some(cts.map_into()), config, tx_buffer)
    }

    fn new_inner(
        peri: PeripheralRef<'d, U>,
        txd: PeripheralRef<'d, AnyPin>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
        tx_buffer: &'d mut [u8],
    ) -> Self {
        configure(U::regs(), config, cts.is_some());

        let this = Self::new_innerer(peri, txd, cts, tx_buffer);

        U::Interrupt::pend();
        unsafe { U::Interrupt::enable() };

        U::state().tx_rx_refcount.store(1, Ordering::Relaxed);

        this
    }

    fn new_innerer(
        peri: PeripheralRef<'d, U>,
        txd: PeripheralRef<'d, AnyPin>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
        tx_buffer: &'d mut [u8],
    ) -> Self {
        let r = U::regs();

        txd.set_high();
        txd.conf().write(|w| w.dir().output().drive().h0h1());
        r.psel.txd.write(|w| unsafe { w.bits(txd.psel_bits()) });

        if let Some(pin) = &cts {
            pin.conf().write(|w| w.input().connect().drive().h0h1());
        }
        r.psel.cts.write(|w| unsafe { w.bits(cts.psel_bits()) });

        // Initialize state
        let s = U::buffered_state();
        s.tx_count.store(0, Ordering::Relaxed);
        let len = tx_buffer.len();
        unsafe { s.tx_buf.init(tx_buffer.as_mut_ptr(), len) };

        r.events_txstarted.reset();

        // Enable interrupts
        r.intenset.write(|w| {
            w.endtx().set();
            w
        });

        Self { _peri: peri }
    }

    /// Write a buffer into this writer, returning how many bytes were written.
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
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
        .await
    }

    /// Flush this output stream, ensuring that all intermediately buffered contents reach their destination.
    pub async fn flush(&mut self) -> Result<(), Error> {
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
        .await
    }
}

impl<'a, U: UarteInstance> Drop for BufferedUarteTx<'a, U> {
    fn drop(&mut self) {
        let r = U::regs();

        r.intenclr.write(|w| {
            w.txdrdy().set_bit();
            w.txstarted().set_bit();
            w.txstopped().set_bit();
            w
        });
        r.events_txstopped.reset();
        r.tasks_stoptx.write(|w| unsafe { w.bits(1) });
        while r.events_txstopped.read().bits() == 0 {}

        let s = U::buffered_state();
        unsafe { s.tx_buf.deinit() }

        let s = U::state();
        drop_tx_rx(r, s);
    }
}

/// Reader part of the buffered UARTE driver.
pub struct BufferedUarteRx<'d, U: UarteInstance, T: TimerInstance> {
    _peri: PeripheralRef<'d, U>,
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
    pub fn new(
        uarte: impl Peripheral<P = U> + 'd,
        timer: impl Peripheral<P = T> + 'd,
        ppi_ch1: impl Peripheral<P = impl ConfigurableChannel> + 'd,
        ppi_ch2: impl Peripheral<P = impl ConfigurableChannel> + 'd,
        ppi_group: impl Peripheral<P = impl Group> + 'd,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        rxd: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
        rx_buffer: &'d mut [u8],
    ) -> Self {
        into_ref!(uarte, timer, rxd, ppi_ch1, ppi_ch2, ppi_group);
        Self::new_inner(
            uarte,
            timer,
            ppi_ch1.map_into(),
            ppi_ch2.map_into(),
            ppi_group.map_into(),
            rxd.map_into(),
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
    pub fn new_with_rts(
        uarte: impl Peripheral<P = U> + 'd,
        timer: impl Peripheral<P = T> + 'd,
        ppi_ch1: impl Peripheral<P = impl ConfigurableChannel> + 'd,
        ppi_ch2: impl Peripheral<P = impl ConfigurableChannel> + 'd,
        ppi_group: impl Peripheral<P = impl Group> + 'd,
        _irq: impl interrupt::typelevel::Binding<U::Interrupt, InterruptHandler<U>> + 'd,
        rxd: impl Peripheral<P = impl GpioPin> + 'd,
        rts: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
        rx_buffer: &'d mut [u8],
    ) -> Self {
        into_ref!(uarte, timer, rxd, rts, ppi_ch1, ppi_ch2, ppi_group);
        Self::new_inner(
            uarte,
            timer,
            ppi_ch1.map_into(),
            ppi_ch2.map_into(),
            ppi_group.map_into(),
            rxd.map_into(),
            Some(rts.map_into()),
            config,
            rx_buffer,
        )
    }

    fn new_inner(
        peri: PeripheralRef<'d, U>,
        timer: PeripheralRef<'d, T>,
        ppi_ch1: PeripheralRef<'d, AnyConfigurableChannel>,
        ppi_ch2: PeripheralRef<'d, AnyConfigurableChannel>,
        ppi_group: PeripheralRef<'d, AnyGroup>,
        rxd: PeripheralRef<'d, AnyPin>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
        rx_buffer: &'d mut [u8],
    ) -> Self {
        configure(U::regs(), config, rts.is_some());

        let this = Self::new_innerer(peri, timer, ppi_ch1, ppi_ch2, ppi_group, rxd, rts, rx_buffer);

        U::Interrupt::pend();
        unsafe { U::Interrupt::enable() };

        U::state().tx_rx_refcount.store(1, Ordering::Relaxed);

        this
    }

    fn new_innerer(
        peri: PeripheralRef<'d, U>,
        timer: PeripheralRef<'d, T>,
        ppi_ch1: PeripheralRef<'d, AnyConfigurableChannel>,
        ppi_ch2: PeripheralRef<'d, AnyConfigurableChannel>,
        ppi_group: PeripheralRef<'d, AnyGroup>,
        rxd: PeripheralRef<'d, AnyPin>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        rx_buffer: &'d mut [u8],
    ) -> Self {
        assert!(rx_buffer.len() % 2 == 0);

        let r = U::regs();

        rxd.conf().write(|w| w.input().connect().drive().h0h1());
        r.psel.rxd.write(|w| unsafe { w.bits(rxd.psel_bits()) });

        if let Some(pin) = &rts {
            pin.set_high();
            pin.conf().write(|w| w.dir().output().drive().h0h1());
        }
        r.psel.rts.write(|w| unsafe { w.bits(rts.psel_bits()) });

        // Initialize state
        let s = U::buffered_state();
        s.rx_started_count.store(0, Ordering::Relaxed);
        s.rx_ended_count.store(0, Ordering::Relaxed);
        s.rx_started.store(false, Ordering::Relaxed);
        let len = rx_buffer.len();
        unsafe { s.rx_buf.init(rx_buffer.as_mut_ptr(), len) };

        // clear errors
        let errors = r.errorsrc.read().bits();
        r.errorsrc.write(|w| unsafe { w.bits(errors) });

        r.events_rxstarted.reset();
        r.events_error.reset();
        r.events_endrx.reset();

        // Enable interrupts
        r.intenset.write(|w| {
            w.endtx().set();
            w.rxstarted().set();
            w.error().set();
            w.endrx().set();
            w
        });

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
    pub async fn fill_buf(&mut self) -> Result<&[u8], Error> {
        poll_fn(move |cx| {
            compiler_fence(Ordering::SeqCst);
            //trace!("poll_read");

            let r = U::regs();
            let s = U::buffered_state();
            let ss = U::state();

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
                ss.rx_waker.register(cx.waker());
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

    /// Tell this buffer that `amt` bytes have been consumed from the buffer, so they should no longer be returned in calls to `fill_buf`.
    pub fn consume(&mut self, amt: usize) {
        if amt == 0 {
            return;
        }

        let s = U::buffered_state();
        let mut rx = unsafe { s.rx_buf.reader() };
        rx.pop_done(amt);
        U::regs().intenset.write(|w| w.rxstarted().set());
    }
}

impl<'a, U: UarteInstance, T: TimerInstance> Drop for BufferedUarteRx<'a, U, T> {
    fn drop(&mut self) {
        self._ppi_group.disable_all();

        let r = U::regs();

        self.timer.stop();

        r.intenclr.write(|w| {
            w.rxdrdy().set_bit();
            w.rxstarted().set_bit();
            w.rxto().set_bit();
            w
        });
        r.events_rxto.reset();
        r.tasks_stoprx.write(|w| unsafe { w.bits(1) });
        while r.events_rxto.read().bits() == 0 {}

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
