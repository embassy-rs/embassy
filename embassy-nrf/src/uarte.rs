//! HAL interface to the UARTE peripheral
//!
//! See product specification:
//!
//! - nrf52832: Section 35
//! - nrf52840: Section 6.34
use core::cell::UnsafeCell;
use core::cmp::min;
use core::marker::PhantomPinned;
use core::ops::Deref;
use core::pin::Pin;
use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};

use crate::interrupt;
use crate::interrupt::CriticalSection;
use crate::pac::{uarte0, Interrupt, UARTE0, UARTE1};
use embedded_hal::digital::v2::OutputPin;
use nrf52840_hal::gpio::{Floating, Input, Output, Pin as GpioPin, Port as GpioPort, PushPull};

// Re-export SVD variants to allow user to directly set values
pub use uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

use embassy::io::{AsyncBufRead, AsyncWrite, Result};
use embassy::util::WakerStore;

//use crate::trace;

const RINGBUF_SIZE: usize = 512;
struct RingBuf {
    buf: [u8; RINGBUF_SIZE],
    start: usize,
    end: usize,
    empty: bool,
}

impl RingBuf {
    fn new() -> Self {
        RingBuf {
            buf: [0; RINGBUF_SIZE],
            start: 0,
            end: 0,
            empty: true,
        }
    }

    fn push_buf(&mut self) -> &mut [u8] {
        if self.start == self.end && !self.empty {
            #[cfg(feature = "defmt")]
            defmt::trace!("  ringbuf: push_buf empty");
            return &mut self.buf[..0];
        }

        let n = if self.start <= self.end {
            RINGBUF_SIZE - self.end
        } else {
            self.start - self.end
        };

        #[cfg(feature = "defmt")]
        defmt::trace!("  ringbuf: push_buf {:?}..{:?}", self.end, self.end + n);
        &mut self.buf[self.end..self.end + n]
    }

    fn push(&mut self, n: usize) {
        #[cfg(feature = "defmt")]
        defmt::trace!("  ringbuf: push {:?}", n);
        if n == 0 {
            return;
        }

        self.end = Self::wrap(self.end + n);
        self.empty = false;
    }

    fn pop_buf(&mut self) -> &mut [u8] {
        if self.empty {
            #[cfg(feature = "defmt")]
            defmt::trace!("  ringbuf: pop_buf empty");
            return &mut self.buf[..0];
        }

        let n = if self.end <= self.start {
            RINGBUF_SIZE - self.start
        } else {
            self.end - self.start
        };

        #[cfg(feature = "defmt")]
        defmt::trace!("  ringbuf: pop_buf {:?}..{:?}", self.start, self.start + n);
        &mut self.buf[self.start..self.start + n]
    }

    fn pop(&mut self, n: usize) {
        #[cfg(feature = "defmt")]
        defmt::trace!("  ringbuf: pop {:?}", n);
        if n == 0 {
            return;
        }

        self.start = Self::wrap(self.start + n);
        self.empty = self.start == self.end;
    }

    fn wrap(n: usize) -> usize {
        assert!(n <= RINGBUF_SIZE);
        if n == RINGBUF_SIZE {
            0
        } else {
            n
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum RxState {
    Idle,
    Receiving,
    ReceivingReady,
    Stopping,
}
#[derive(Copy, Clone, Debug, PartialEq)]
enum TxState {
    Idle,
    Transmitting(usize),
}

/// Interface to a UARTE instance
///
/// This is a very basic interface that comes with the following limitations:
/// - The UARTE instances share the same address space with instances of UART.
///   You need to make sure that conflicting instances
///   are disabled before using `Uarte`. See product specification:
///     - nrf52832: Section 15.2
///     - nrf52840: Section 6.1.2
pub struct Uarte<T: Instance> {
    started: bool,
    state: UnsafeCell<UarteState<T>>,
}

// public because it needs to be used in Instance::{get_state, set_state}, but
// should not be used outside the module
#[doc(hidden)]
pub struct UarteState<T> {
    inner: T,

    rx: RingBuf,
    rx_state: RxState,
    rx_waker: WakerStore,

    tx: RingBuf,
    tx_state: TxState,
    tx_waker: WakerStore,

    _pin: PhantomPinned,
}

fn port_bit(port: GpioPort) -> bool {
    match port {
        GpioPort::Port0 => false,
        GpioPort::Port1 => true,
    }
}

impl<T: Instance> Uarte<T> {
    pub fn new(uarte: T, mut pins: Pins, parity: Parity, baudrate: Baudrate) -> Self {
        // Select pins
        uarte.psel.rxd.write(|w| {
            let w = unsafe { w.pin().bits(pins.rxd.pin()) };
            let w = w.port().bit(port_bit(pins.rxd.port()));
            w.connect().connected()
        });
        pins.txd.set_high().unwrap();
        uarte.psel.txd.write(|w| {
            let w = unsafe { w.pin().bits(pins.txd.pin()) };
            let w = w.port().bit(port_bit(pins.txd.port()));
            w.connect().connected()
        });

        // Optional pins
        uarte.psel.cts.write(|w| {
            if let Some(ref pin) = pins.cts {
                let w = unsafe { w.pin().bits(pin.pin()) };
                let w = w.port().bit(port_bit(pin.port()));
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });

        uarte.psel.rts.write(|w| {
            if let Some(ref pin) = pins.rts {
                let w = unsafe { w.pin().bits(pin.pin()) };
                let w = w.port().bit(port_bit(pin.port()));
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });

        // Enable UARTE instance
        uarte.enable.write(|w| w.enable().enabled());

        // Enable interrupts
        uarte.intenset.write(|w| w.endrx().set().endtx().set());

        // Configure
        let hardware_flow_control = pins.rts.is_some() && pins.cts.is_some();
        uarte
            .config
            .write(|w| w.hwfc().bit(hardware_flow_control).parity().variant(parity));

        // Configure frequency
        uarte.baudrate.write(|w| w.baudrate().variant(baudrate));

        Uarte {
            started: false,
            state: UnsafeCell::new(UarteState {
                inner: uarte,

                rx: RingBuf::new(),
                rx_state: RxState::Idle,
                rx_waker: WakerStore::new(),

                tx: RingBuf::new(),
                tx_state: TxState::Idle,
                tx_waker: WakerStore::new(),

                _pin: PhantomPinned,
            }),
        }
    }

    fn with_state<'a, R>(
        self: Pin<&'a mut Self>,
        f: impl FnOnce(Pin<&'a mut UarteState<T>>) -> R,
    ) -> R {
        let Self { state, started } = unsafe { self.get_unchecked_mut() };

        interrupt::free(|cs| {
            let ptr = state.get();

            if !*started {
                T::set_state(cs, ptr);

                *started = true;

                // safety: safe because critical section ensures only one *mut UartState
                // exists at the same time.
                unsafe { Pin::new_unchecked(&mut *ptr) }.start();
            }

            // safety: safe because critical section ensures only one *mut UartState
            // exists at the same time.
            f(unsafe { Pin::new_unchecked(&mut *ptr) })
        })
    }
}

impl<T: Instance> Drop for Uarte<T> {
    fn drop(&mut self) {
        // stop DMA before dropping, because DMA is using the buffer in `self`.
        todo!()
    }
}

impl<T: Instance> AsyncBufRead for Uarte<T> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        self.with_state(|s| s.poll_fill_buf(cx))
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.with_state(|s| s.consume(amt))
    }
}

impl<T: Instance> AsyncWrite for Uarte<T> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        self.with_state(|s| s.poll_write(cx, buf))
    }
}

impl<T: Instance> UarteState<T> {
    pub fn start(self: Pin<&mut Self>) {
        interrupt::set_priority(T::interrupt(), interrupt::Priority::Level7);
        interrupt::enable(T::interrupt());
        interrupt::pend(T::interrupt());
    }

    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        let this = unsafe { self.get_unchecked_mut() };

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(Ordering::SeqCst);
        #[cfg(feature = "defmt")]
        defmt::trace!("poll_read");

        // We have data ready in buffer? Return it.
        let buf = this.rx.pop_buf();
        if buf.len() != 0 {
            #[cfg(feature = "defmt")]
            defmt::trace!("  got {:?} {:?}", buf.as_ptr() as u32, buf.len());
            return Poll::Ready(Ok(buf));
        }

        #[cfg(feature = "defmt")]
        defmt::trace!("  empty");

        if this.rx_state == RxState::ReceivingReady {
            #[cfg(feature = "defmt")]
            defmt::trace!("  stopping");
            this.rx_state = RxState::Stopping;
            this.inner.tasks_stoprx.write(|w| unsafe { w.bits(1) });
        }

        this.rx_waker.store(cx.waker());
        Poll::Pending
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let this = unsafe { self.get_unchecked_mut() };
        #[cfg(feature = "defmt")]
        defmt::trace!("consume {:?}", amt);
        this.rx.pop(amt);
        interrupt::pend(T::interrupt());
    }

    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let this = unsafe { self.get_unchecked_mut() };

        #[cfg(feature = "defmt")]
        defmt::trace!("poll_write: {:?}", buf.len());

        let tx_buf = this.tx.push_buf();
        if tx_buf.len() == 0 {
            #[cfg(feature = "defmt")]
            defmt::trace!("poll_write: pending");
            this.tx_waker.store(cx.waker());
            return Poll::Pending;
        }

        let n = min(tx_buf.len(), buf.len());
        tx_buf[..n].copy_from_slice(&buf[..n]);
        this.tx.push(n);

        #[cfg(feature = "defmt")]
        defmt::trace!("poll_write: queued {:?}", n);

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(Ordering::SeqCst);

        interrupt::pend(T::interrupt());

        Poll::Ready(Ok(n))
    }

    fn on_interrupt(&mut self) {
        #[cfg(feature = "defmt")]
        defmt::trace!("irq: start");
        let mut more_work = true;
        while more_work {
            more_work = false;
            match self.rx_state {
                RxState::Idle => {
                    #[cfg(feature = "defmt")]
                    defmt::trace!("  irq_rx: in state idle");

                    if self.inner.events_rxdrdy.read().bits() != 0 {
                        #[cfg(feature = "defmt")]
                        defmt::trace!("  irq_rx: rxdrdy?????");
                        self.inner.events_rxdrdy.reset();
                    }

                    if self.inner.events_endrx.read().bits() != 0 {
                        panic!("unexpected endrx");
                    }

                    let buf = self.rx.push_buf();
                    if buf.len() != 0 {
                        #[cfg(feature = "defmt")]
                        defmt::trace!("  irq_rx: starting {:?}", buf.len());
                        self.rx_state = RxState::Receiving;

                        // Set up the DMA read
                        self.inner.rxd.ptr.write(|w|
                            // The PTR field is a full 32 bits wide and accepts the full range
                            // of values.
                            unsafe { w.ptr().bits(buf.as_ptr() as u32) });
                        self.inner.rxd.maxcnt.write(|w|
                            // We're giving it the length of the buffer, so no danger of
                            // accessing invalid memory. We have verified that the length of the
                            // buffer fits in an `u8`, so the cast to `u8` is also fine.
                            //
                            // The MAXCNT field is at least 8 bits wide and accepts the full
                            // range of values.
                            unsafe { w.maxcnt().bits(buf.len() as _) });
                        #[cfg(feature = "defmt")]
                        defmt::trace!("  irq_rx: buf {:?} {:?}", buf.as_ptr() as u32, buf.len());

                        // Enable RXRDY interrupt.
                        self.inner.events_rxdrdy.reset();
                        self.inner.intenset.write(|w| w.rxdrdy().set());

                        // Start UARTE Receive transaction
                        self.inner.tasks_startrx.write(|w|
                            // `1` is a valid value to write to task registers.
                            unsafe { w.bits(1) });
                    }
                }
                RxState::Receiving => {
                    #[cfg(feature = "defmt")]
                    defmt::trace!("  irq_rx: in state receiving");
                    if self.inner.events_rxdrdy.read().bits() != 0 {
                        #[cfg(feature = "defmt")]
                        defmt::trace!("  irq_rx: rxdrdy");

                        // Disable the RXRDY event interrupt
                        // RXRDY is triggered for every byte, but we only care about whether we have
                        // some bytes or not. So as soon as we have at least one, disable it, to avoid
                        // wasting CPU cycles in interrupts.
                        self.inner.intenclr.write(|w| w.rxdrdy().clear());

                        self.inner.events_rxdrdy.reset();

                        self.rx_waker.wake();
                        self.rx_state = RxState::ReceivingReady;
                        more_work = true; // in case we also have endrx pending
                    }
                }
                RxState::ReceivingReady | RxState::Stopping => {
                    #[cfg(feature = "defmt")]
                    defmt::trace!("  irq_rx: in state ReceivingReady");

                    if self.inner.events_rxdrdy.read().bits() != 0 {
                        #[cfg(feature = "defmt")]
                        defmt::trace!("  irq_rx: rxdrdy");
                        self.inner.events_rxdrdy.reset();
                    }

                    if self.inner.events_endrx.read().bits() != 0 {
                        let n: usize = self.inner.rxd.amount.read().amount().bits() as usize;
                        #[cfg(feature = "defmt")]
                        defmt::trace!("  irq_rx: endrx {:?}", n);
                        self.rx.push(n);

                        self.inner.events_endrx.reset();

                        self.rx_waker.wake();
                        self.rx_state = RxState::Idle;
                        more_work = true; // start another rx if possible
                    }
                }
            }
        }

        more_work = true;
        while more_work {
            more_work = false;
            match self.tx_state {
                TxState::Idle => {
                    #[cfg(feature = "defmt")]
                    defmt::trace!("  irq_tx: in state Idle");
                    let buf = self.tx.pop_buf();
                    if buf.len() != 0 {
                        #[cfg(feature = "defmt")]
                        defmt::trace!("  irq_tx: starting {:?}", buf.len());
                        self.tx_state = TxState::Transmitting(buf.len());

                        // Set up the DMA write
                        self.inner.txd.ptr.write(|w|
                            // The PTR field is a full 32 bits wide and accepts the full range
                            // of values.
                            unsafe { w.ptr().bits(buf.as_ptr() as u32) });
                        self.inner.txd.maxcnt.write(|w|
                            // We're giving it the length of the buffer, so no danger of
                            // accessing invalid memory. We have verified that the length of the
                            // buffer fits in an `u8`, so the cast to `u8` is also fine.
                            //
                            // The MAXCNT field is 8 bits wide and accepts the full range of
                            // values.
                            unsafe { w.maxcnt().bits(buf.len() as _) });

                        // Start UARTE Transmit transaction
                        self.inner.tasks_starttx.write(|w|
                            // `1` is a valid value to write to task registers.
                            unsafe { w.bits(1) });
                    }
                }
                TxState::Transmitting(n) => {
                    #[cfg(feature = "defmt")]
                    defmt::trace!("  irq_tx: in state Transmitting");
                    if self.inner.events_endtx.read().bits() != 0 {
                        self.inner.events_endtx.reset();

                        #[cfg(feature = "defmt")]
                        defmt::trace!("  irq_tx: endtx {:?}", n);
                        self.tx.pop(n);
                        self.tx_waker.wake();
                        self.tx_state = TxState::Idle;
                        more_work = true; // start another tx if possible
                    }
                }
            }
        }
        #[cfg(feature = "defmt")]
        defmt::trace!("irq: end");
    }
}

pub struct Pins {
    pub rxd: GpioPin<Input<Floating>>,
    pub txd: GpioPin<Output<PushPull>>,
    pub cts: Option<GpioPin<Input<Floating>>>,
    pub rts: Option<GpioPin<Output<PushPull>>>,
}

mod private {
    use nrf52840_pac::{UARTE0, UARTE1};
    pub trait Sealed {}

    impl Sealed for UARTE0 {}
    impl Sealed for UARTE1 {}
}

pub trait Instance: Deref<Target = uarte0::RegisterBlock> + Sized + private::Sealed {
    fn interrupt() -> Interrupt;

    #[doc(hidden)]
    fn get_state(_cs: &CriticalSection) -> *mut UarteState<Self>;

    #[doc(hidden)]
    fn set_state(_cs: &CriticalSection, state: *mut UarteState<Self>);
}

#[interrupt]
unsafe fn UARTE0_UART0() {
    interrupt::free(|cs| UARTE0::get_state(cs).as_mut().unwrap().on_interrupt());
}

#[interrupt]
unsafe fn UARTE1() {
    interrupt::free(|cs| UARTE1::get_state(cs).as_mut().unwrap().on_interrupt());
}

static mut UARTE0_STATE: *mut UarteState<UARTE0> = ptr::null_mut();
static mut UARTE1_STATE: *mut UarteState<UARTE1> = ptr::null_mut();

impl Instance for UARTE0 {
    fn interrupt() -> Interrupt {
        Interrupt::UARTE0_UART0
    }

    fn get_state(_cs: &CriticalSection) -> *mut UarteState<Self> {
        unsafe { UARTE0_STATE } // Safe because of CriticalSection
    }
    fn set_state(_cs: &CriticalSection, state: *mut UarteState<Self>) {
        unsafe { UARTE0_STATE = state } // Safe because of CriticalSection
    }
}

impl Instance for UARTE1 {
    fn interrupt() -> Interrupt {
        Interrupt::UARTE1
    }

    fn get_state(_cs: &CriticalSection) -> *mut UarteState<Self> {
        unsafe { UARTE1_STATE } // Safe because of CriticalSection
    }
    fn set_state(_cs: &CriticalSection, state: *mut UarteState<Self>) {
        unsafe { UARTE1_STATE = state } // Safe because of CriticalSection
    }
}
