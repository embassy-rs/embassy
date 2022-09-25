//! Async buffered UART
//!
//! WARNING!!! The functionality provided here is intended to be used only
//! in situations where hardware flow control are available i.e. CTS and RTS.
//! This is a problem that should be addressed at a later stage and can be
//! fully explained at <https://github.com/embassy-rs/embassy/issues/536>.
//!
//! Note that discarding a future from a read or write operation may lead to losing
//! data. For example, when using `futures_util::future::select` and completion occurs
//! on the "other" future, you should capture the incomplete future and continue to use
//! it for the next read or write. This pattern is a consideration for all IO, and not
//! just serial communications.
//!
//! Please also see [crate::uarte] to understand when [BufferedUarte] should be used.

use core::cell::RefCell;
use core::cmp::min;
use core::future::{poll_fn, Future};
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_cortex_m::peripheral::{PeripheralMutex, PeripheralState, StateStorage};
use embassy_hal_common::ring_buffer::RingBuffer;
use embassy_hal_common::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::WakerRegistration;
// Re-export SVD variants to allow user to directly set values
pub use pac::uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

use crate::gpio::{self, Pin as GpioPin};
use crate::interrupt::InterruptExt;
use crate::ppi::{AnyConfigurableChannel, ConfigurableChannel, Event, Ppi, Task};
use crate::timer::{Frequency, Instance as TimerInstance, Timer};
use crate::uarte::{apply_workaround_for_enable_anomaly, Config, Instance as UarteInstance};
use crate::{pac, Peripheral};

#[derive(Copy, Clone, Debug, PartialEq)]
enum RxState {
    Idle,
    Receiving,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum TxState {
    Idle,
    Transmitting(usize),
}

/// A type for storing the state of the UARTE peripheral that can be stored in a static.
pub struct State<'d, U: UarteInstance, T: TimerInstance>(StateStorage<StateInner<'d, U, T>>);
impl<'d, U: UarteInstance, T: TimerInstance> State<'d, U, T> {
    /// Create an instance for storing UARTE peripheral state.
    pub fn new() -> Self {
        Self(StateStorage::new())
    }
}

struct StateInner<'d, U: UarteInstance, T: TimerInstance> {
    _peri: PeripheralRef<'d, U>,
    timer: Timer<'d, T>,
    _ppi_ch1: Ppi<'d, AnyConfigurableChannel, 1, 2>,
    _ppi_ch2: Ppi<'d, AnyConfigurableChannel, 1, 1>,

    rx: RingBuffer<'d>,
    rx_state: RxState,
    rx_waker: WakerRegistration,

    tx: RingBuffer<'d>,
    tx_state: TxState,
    tx_waker: WakerRegistration,
}

/// Interface to a UARTE instance
pub struct BufferedUarte<'d, U: UarteInstance, T: TimerInstance> {
    inner: RefCell<PeripheralMutex<'d, StateInner<'d, U, T>>>,
}

impl<'d, U: UarteInstance, T: TimerInstance> Unpin for BufferedUarte<'d, U, T> {}

impl<'d, U: UarteInstance, T: TimerInstance> BufferedUarte<'d, U, T> {
    /// Create a new instance of a BufferedUarte.
    ///
    /// See the [module documentation](crate::buffered_uarte) for more details about the intended use.
    ///
    /// The BufferedUarte uses the provided state to store the buffers and peripheral state. The timer and ppi channels are used to 'emulate' idle line detection so that read operations
    /// can return early if there is no data to receive.
    pub fn new(
        state: &'d mut State<'d, U, T>,
        peri: impl Peripheral<P = U> + 'd,
        timer: impl Peripheral<P = T> + 'd,
        ppi_ch1: impl Peripheral<P = impl ConfigurableChannel + 'd> + 'd,
        ppi_ch2: impl Peripheral<P = impl ConfigurableChannel + 'd> + 'd,
        irq: impl Peripheral<P = U::Interrupt> + 'd,
        rxd: impl Peripheral<P = impl GpioPin> + 'd,
        txd: impl Peripheral<P = impl GpioPin> + 'd,
        cts: impl Peripheral<P = impl GpioPin> + 'd,
        rts: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
        rx_buffer: &'d mut [u8],
        tx_buffer: &'d mut [u8],
    ) -> Self {
        into_ref!(peri, ppi_ch1, ppi_ch2, irq, rxd, txd, cts, rts);

        let r = U::regs();

        let mut timer = Timer::new(timer);

        rxd.conf().write(|w| w.input().connect().drive().h0h1());
        r.psel.rxd.write(|w| unsafe { w.bits(rxd.psel_bits()) });

        txd.set_high();
        txd.conf().write(|w| w.dir().output().drive().h0h1());
        r.psel.txd.write(|w| unsafe { w.bits(txd.psel_bits()) });

        cts.conf().write(|w| w.input().connect().drive().h0h1());
        r.psel.cts.write(|w| unsafe { w.bits(cts.psel_bits()) });

        rts.set_high();
        rts.conf().write(|w| w.dir().output().drive().h0h1());
        r.psel.rts.write(|w| unsafe { w.bits(rts.psel_bits()) });

        r.baudrate.write(|w| w.baudrate().variant(config.baudrate));
        r.config.write(|w| w.parity().variant(config.parity));

        // Configure
        r.config.write(|w| {
            w.hwfc().bit(true);
            w.parity().variant(config.parity);
            w
        });
        r.baudrate.write(|w| w.baudrate().variant(config.baudrate));

        // Enable interrupts
        r.intenset.write(|w| w.endrx().set().endtx().set());

        // Disable the irq, let the Registration enable it when everything is set up.
        irq.disable();
        irq.pend();

        // Enable UARTE instance
        apply_workaround_for_enable_anomaly(&r);
        r.enable.write(|w| w.enable().enabled());

        // BAUDRATE register values are `baudrate * 2^32 / 16000000`
        // source: https://devzone.nordicsemi.com/f/nordic-q-a/391/uart-baudrate-register-values
        //
        // We want to stop RX if line is idle for 2 bytes worth of time
        // That is 20 bits (each byte is 1 start bit + 8 data bits + 1 stop bit)
        // This gives us the amount of 16M ticks for 20 bits.
        let timeout = 0x8000_0000 / (config.baudrate as u32 / 40);

        timer.set_frequency(Frequency::F16MHz);
        timer.cc(0).write(timeout);
        timer.cc(0).short_compare_clear();
        timer.cc(0).short_compare_stop();

        let mut ppi_ch1 = Ppi::new_one_to_two(
            ppi_ch1.map_into(),
            Event::from_reg(&r.events_rxdrdy),
            timer.task_clear(),
            timer.task_start(),
        );
        ppi_ch1.enable();

        let mut ppi_ch2 = Ppi::new_one_to_one(
            ppi_ch2.map_into(),
            timer.cc(0).event_compare(),
            Task::from_reg(&r.tasks_stoprx),
        );
        ppi_ch2.enable();

        Self {
            inner: RefCell::new(PeripheralMutex::new(irq, &mut state.0, move || StateInner {
                _peri: peri,
                timer,
                _ppi_ch1: ppi_ch1,
                _ppi_ch2: ppi_ch2,

                rx: RingBuffer::new(rx_buffer),
                rx_state: RxState::Idle,
                rx_waker: WakerRegistration::new(),

                tx: RingBuffer::new(tx_buffer),
                tx_state: TxState::Idle,
                tx_waker: WakerRegistration::new(),
            })),
        }
    }

    /// Adjust the baud rate to the provided value.
    pub fn set_baudrate(&mut self, baudrate: Baudrate) {
        self.inner.borrow_mut().with(|state| {
            let r = U::regs();

            let timeout = 0x8000_0000 / (baudrate as u32 / 40);
            state.timer.cc(0).write(timeout);
            state.timer.clear();

            r.baudrate.write(|w| w.baudrate().variant(baudrate));
        });
    }

    pub fn split<'u>(&'u mut self) -> (BufferedUarteRx<'u, 'd, U, T>, BufferedUarteTx<'u, 'd, U, T>) {
        (BufferedUarteRx { inner: self }, BufferedUarteTx { inner: self })
    }

    async fn inner_read<'a>(&'a self, buf: &'a mut [u8]) -> Result<usize, core::convert::Infallible> {
        poll_fn(move |cx| {
            let mut do_pend = false;
            let mut inner = self.inner.borrow_mut();
            let res = inner.with(|state| {
                compiler_fence(Ordering::SeqCst);
                trace!("poll_read");

                // We have data ready in buffer? Return it.
                let data = state.rx.pop_buf();
                if !data.is_empty() {
                    trace!("  got {:?} {:?}", data.as_ptr() as u32, data.len());
                    let len = data.len().min(buf.len());
                    buf[..len].copy_from_slice(&data[..len]);
                    state.rx.pop(len);
                    do_pend = true;
                    return Poll::Ready(Ok(len));
                }

                trace!("  empty");
                state.rx_waker.register(cx.waker());
                Poll::Pending
            });
            if do_pend {
                inner.pend();
            }

            res
        })
        .await
    }

    async fn inner_write<'a>(&'a self, buf: &'a [u8]) -> Result<usize, core::convert::Infallible> {
        poll_fn(move |cx| {
            let mut inner = self.inner.borrow_mut();
            let res = inner.with(|state| {
                trace!("poll_write: {:?}", buf.len());

                let tx_buf = state.tx.push_buf();
                if tx_buf.is_empty() {
                    trace!("poll_write: pending");
                    state.tx_waker.register(cx.waker());
                    return Poll::Pending;
                }

                let n = min(tx_buf.len(), buf.len());
                tx_buf[..n].copy_from_slice(&buf[..n]);
                state.tx.push(n);

                trace!("poll_write: queued {:?}", n);

                compiler_fence(Ordering::SeqCst);

                Poll::Ready(Ok(n))
            });

            inner.pend();

            res
        })
        .await
    }

    async fn inner_flush<'a>(&'a self) -> Result<(), core::convert::Infallible> {
        poll_fn(move |cx| {
            self.inner.borrow_mut().with(|state| {
                trace!("poll_flush");

                if !state.tx.is_empty() {
                    trace!("poll_flush: pending");
                    state.tx_waker.register(cx.waker());
                    return Poll::Pending;
                }

                Poll::Ready(Ok(()))
            })
        })
        .await
    }

    async fn inner_fill_buf<'a>(&'a self) -> Result<&'a [u8], core::convert::Infallible> {
        poll_fn(move |cx| {
            self.inner.borrow_mut().with(|state| {
                compiler_fence(Ordering::SeqCst);
                trace!("fill_buf");

                // We have data ready in buffer? Return it.
                let buf = state.rx.pop_buf();
                if !buf.is_empty() {
                    trace!("  got {:?} {:?}", buf.as_ptr() as u32, buf.len());
                    let buf: &[u8] = buf;
                    // Safety: buffer lives as long as uart
                    let buf: &[u8] = unsafe { core::mem::transmute(buf) };
                    return Poll::Ready(Ok(buf));
                }

                trace!("  empty");
                state.rx_waker.register(cx.waker());
                Poll::<Result<&[u8], core::convert::Infallible>>::Pending
            })
        })
        .await
    }

    fn inner_consume(&self, amt: usize) {
        let mut inner = self.inner.borrow_mut();
        let signal = inner.with(|state| {
            let full = state.rx.is_full();
            state.rx.pop(amt);
            full
        });
        if signal {
            inner.pend();
        }
    }
}

pub struct BufferedUarteTx<'u, 'd, U: UarteInstance, T: TimerInstance> {
    inner: &'u BufferedUarte<'d, U, T>,
}

pub struct BufferedUarteRx<'u, 'd, U: UarteInstance, T: TimerInstance> {
    inner: &'u BufferedUarte<'d, U, T>,
}

impl<'d, U: UarteInstance, T: TimerInstance> embedded_io::Io for BufferedUarte<'d, U, T> {
    type Error = core::convert::Infallible;
}

impl<'u, 'd, U: UarteInstance, T: TimerInstance> embedded_io::Io for BufferedUarteRx<'u, 'd, U, T> {
    type Error = core::convert::Infallible;
}

impl<'u, 'd, U: UarteInstance, T: TimerInstance> embedded_io::Io for BufferedUarteTx<'u, 'd, U, T> {
    type Error = core::convert::Infallible;
}

impl<'d, U: UarteInstance, T: TimerInstance> embedded_io::asynch::Read for BufferedUarte<'d, U, T> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        self.inner_read(buf)
    }
}

impl<'u, 'd: 'u, U: UarteInstance, T: TimerInstance> embedded_io::asynch::Read for BufferedUarteRx<'u, 'd, U, T> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        self.inner.inner_read(buf)
    }
}

impl<'d, U: UarteInstance, T: TimerInstance> embedded_io::asynch::BufRead for BufferedUarte<'d, U, T> {
    type FillBufFuture<'a> = impl Future<Output = Result<&'a [u8], Self::Error>>
    where
        Self: 'a;

    fn fill_buf<'a>(&'a mut self) -> Self::FillBufFuture<'a> {
        self.inner_fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner_consume(amt)
    }
}

impl<'u, 'd: 'u, U: UarteInstance, T: TimerInstance> embedded_io::asynch::BufRead for BufferedUarteRx<'u, 'd, U, T> {
    type FillBufFuture<'a> = impl Future<Output = Result<&'a [u8], Self::Error>>
    where
        Self: 'a;

    fn fill_buf<'a>(&'a mut self) -> Self::FillBufFuture<'a> {
        self.inner.inner_fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.inner_consume(amt)
    }
}

impl<'d, U: UarteInstance, T: TimerInstance> embedded_io::asynch::Write for BufferedUarte<'d, U, T> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        self.inner_write(buf)
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        self.inner_flush()
    }
}

impl<'u, 'd: 'u, U: UarteInstance, T: TimerInstance> embedded_io::asynch::Write for BufferedUarteTx<'u, 'd, U, T> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        self.inner.inner_write(buf)
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        self.inner.inner_flush()
    }
}

impl<'a, U: UarteInstance, T: TimerInstance> Drop for StateInner<'a, U, T> {
    fn drop(&mut self) {
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
    }
}

impl<'a, U: UarteInstance, T: TimerInstance> PeripheralState for StateInner<'a, U, T> {
    type Interrupt = U::Interrupt;
    fn on_interrupt(&mut self) {
        trace!("irq: start");
        let r = U::regs();

        loop {
            match self.rx_state {
                RxState::Idle => {
                    trace!("  irq_rx: in state idle");

                    let buf = self.rx.push_buf();
                    if !buf.is_empty() {
                        trace!("  irq_rx: starting {:?}", buf.len());
                        self.rx_state = RxState::Receiving;

                        // Set up the DMA read
                        r.rxd.ptr.write(|w|
                            // The PTR field is a full 32 bits wide and accepts the full range
                            // of values.
                            unsafe { w.ptr().bits(buf.as_ptr() as u32) });
                        r.rxd.maxcnt.write(|w|
                            // We're giving it the length of the buffer, so no danger of
                            // accessing invalid memory. We have verified that the length of the
                            // buffer fits in an `u8`, so the cast to `u8` is also fine.
                            //
                            // The MAXCNT field is at least 8 bits wide and accepts the full
                            // range of values.
                            unsafe { w.maxcnt().bits(buf.len() as _) });
                        trace!("  irq_rx: buf {:?} {:?}", buf.as_ptr() as u32, buf.len());

                        // Start UARTE Receive transaction
                        r.tasks_startrx.write(|w| unsafe { w.bits(1) });
                    }
                    break;
                }
                RxState::Receiving => {
                    trace!("  irq_rx: in state receiving");
                    if r.events_endrx.read().bits() != 0 {
                        self.timer.stop();

                        let n: usize = r.rxd.amount.read().amount().bits() as usize;
                        trace!("  irq_rx: endrx {:?}", n);
                        self.rx.push(n);

                        r.events_endrx.reset();

                        self.rx_waker.wake();
                        self.rx_state = RxState::Idle;
                    } else {
                        break;
                    }
                }
            }
        }

        loop {
            match self.tx_state {
                TxState::Idle => {
                    trace!("  irq_tx: in state Idle");
                    let buf = self.tx.pop_buf();
                    if !buf.is_empty() {
                        trace!("  irq_tx: starting {:?}", buf.len());
                        self.tx_state = TxState::Transmitting(buf.len());

                        // Set up the DMA write
                        r.txd.ptr.write(|w|
                            // The PTR field is a full 32 bits wide and accepts the full range
                            // of values.
                            unsafe { w.ptr().bits(buf.as_ptr() as u32) });
                        r.txd.maxcnt.write(|w|
                            // We're giving it the length of the buffer, so no danger of
                            // accessing invalid memory. We have verified that the length of the
                            // buffer fits in an `u8`, so the cast to `u8` is also fine.
                            //
                            // The MAXCNT field is 8 bits wide and accepts the full range of
                            // values.
                            unsafe { w.maxcnt().bits(buf.len() as _) });

                        // Start UARTE Transmit transaction
                        r.tasks_starttx.write(|w| unsafe { w.bits(1) });
                    }
                    break;
                }
                TxState::Transmitting(n) => {
                    trace!("  irq_tx: in state Transmitting");
                    if r.events_endtx.read().bits() != 0 {
                        r.events_endtx.reset();

                        trace!("  irq_tx: endtx {:?}", n);
                        self.tx.pop(n);
                        self.tx_waker.wake();
                        self.tx_state = TxState::Idle;
                    } else {
                        break;
                    }
                }
            }
        }
        trace!("irq: end");
    }
}
