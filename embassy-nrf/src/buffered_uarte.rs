//! HAL interface to the UARTE peripheral
//!
//! See product specification:
//!
//! - nrf52832: Section 35
//! - nrf52840: Section 6.34
use core::cmp::min;
use core::marker::PhantomData;
use core::mem;
use core::ops::Deref;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};
use embassy::io::{AsyncBufRead, AsyncWrite, Result};
use embassy::util::WakerRegistration;
use embedded_hal::digital::v2::OutputPin;

use crate::fmt::{panic, todo, *};
use crate::hal::gpio::Port as GpioPort;
use crate::interrupt::{self, OwnedInterrupt};
use crate::pac;
use crate::pac::uarte0;
use crate::util::peripheral;
use crate::util::ring_buffer::RingBuffer;

// Re-export SVD variants to allow user to directly set values
pub use crate::hal::uarte::Pins;
pub use uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

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
pub struct BufferedUarte<'a, T: Instance> {
    reg: peripheral::Registration<State<'a, T>>,
    wtf: PhantomData<&'a ()>,
}

impl<'a, T: Instance> Unpin for BufferedUarte<'a, T> {}

#[cfg(any(feature = "52833", feature = "52840"))]
fn port_bit(port: GpioPort) -> bool {
    match port {
        GpioPort::Port0 => false,
        GpioPort::Port1 => true,
    }
}

impl<'a, T: Instance> BufferedUarte<'a, T> {
    pub fn new(
        uarte: T,
        irq: T::Interrupt,
        rx_buffer: &'a mut [u8],
        tx_buffer: &'a mut [u8],
        mut pins: Pins,
        parity: Parity,
        baudrate: Baudrate,
    ) -> Self {
        // Select pins
        uarte.psel.rxd.write(|w| {
            let w = unsafe { w.pin().bits(pins.rxd.pin()) };
            #[cfg(any(feature = "52833", feature = "52840"))]
            let w = w.port().bit(port_bit(pins.rxd.port()));
            w.connect().connected()
        });
        pins.txd.set_high().unwrap();
        uarte.psel.txd.write(|w| {
            let w = unsafe { w.pin().bits(pins.txd.pin()) };
            #[cfg(any(feature = "52833", feature = "52840"))]
            let w = w.port().bit(port_bit(pins.txd.port()));
            w.connect().connected()
        });

        // Optional pins
        uarte.psel.cts.write(|w| {
            if let Some(ref pin) = pins.cts {
                let w = unsafe { w.pin().bits(pin.pin()) };
                #[cfg(any(feature = "52833", feature = "52840"))]
                let w = w.port().bit(port_bit(pin.port()));
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });

        uarte.psel.rts.write(|w| {
            if let Some(ref pin) = pins.rts {
                let w = unsafe { w.pin().bits(pin.pin()) };
                #[cfg(any(feature = "52833", feature = "52840"))]
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

        irq.pend();

        BufferedUarte {
            reg: peripheral::Registration::new(
                irq,
                State {
                    inner: uarte,

                    rx: RingBuffer::new(rx_buffer),
                    rx_state: RxState::Idle,
                    rx_waker: WakerRegistration::new(),

                    tx: RingBuffer::new(tx_buffer),
                    tx_state: TxState::Idle,
                    tx_waker: WakerRegistration::new(),
                },
            ),
            wtf: PhantomData,
        }
    }
}

impl<'a, T: Instance> Drop for BufferedUarte<'a, T> {
    fn drop(&mut self) {
        // stop DMA before dropping, because DMA is using the buffer in `self`.
        todo!()
    }
}

impl<'a, T: Instance> AsyncBufRead for BufferedUarte<'a, T> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        let this = unsafe { self.get_unchecked_mut() };
        this.reg.with(|state, _| {
            let z: Poll<Result<&[u8]>> = state.poll_fill_buf(cx);
            let z: Poll<Result<&[u8]>> = unsafe { mem::transmute(z) };
            z
        })
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let this = unsafe { self.get_unchecked_mut() };
        this.reg.with(|state, irq| state.consume(irq, amt))
    }
}

impl<'a, T: Instance> AsyncWrite for BufferedUarte<'a, T> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let this = unsafe { self.get_unchecked_mut() };
        this.reg.with(|state, irq| state.poll_write(irq, cx, buf))
    }
}

// ====================================
// ====================================
// ====================================

// public because it needs to be used in Instance trait, but
// should not be used outside the module
#[doc(hidden)]
pub struct State<'a, T: Instance> {
    inner: T,

    rx: RingBuffer<'a>,
    rx_state: RxState,
    rx_waker: WakerRegistration,

    tx: RingBuffer<'a>,
    tx_state: TxState,
    tx_waker: WakerRegistration,
}

impl<'a, T: Instance> State<'a, T> {
    fn poll_fill_buf(&mut self, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(Ordering::SeqCst);
        trace!("poll_read");

        // We have data ready in buffer? Return it.
        let buf = self.rx.pop_buf();
        if buf.len() != 0 {
            trace!("  got {:?} {:?}", buf.as_ptr() as u32, buf.len());
            return Poll::Ready(Ok(buf));
        }

        trace!("  empty");

        if self.rx_state == RxState::ReceivingReady {
            trace!("  stopping");
            self.rx_state = RxState::Stopping;
            self.inner.tasks_stoprx.write(|w| unsafe { w.bits(1) });
        }

        self.rx_waker.register(cx.waker());
        Poll::Pending
    }

    fn consume(&mut self, irq: &mut T::Interrupt, amt: usize) {
        trace!("consume {:?}", amt);
        self.rx.pop(amt);
        irq.pend();
    }

    fn poll_write(
        &mut self,
        irq: &mut T::Interrupt,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        trace!("poll_write: {:?}", buf.len());

        let tx_buf = self.tx.push_buf();
        if tx_buf.len() == 0 {
            trace!("poll_write: pending");
            self.tx_waker.register(cx.waker());
            return Poll::Pending;
        }

        let n = min(tx_buf.len(), buf.len());
        tx_buf[..n].copy_from_slice(&buf[..n]);
        self.tx.push(n);

        trace!("poll_write: queued {:?}", n);

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(Ordering::SeqCst);

        irq.pend();

        Poll::Ready(Ok(n))
    }
}

impl<'a, T: Instance> peripheral::State for State<'a, T> {
    type Interrupt = T::Interrupt;
    fn store<'b>() -> &'b peripheral::Store<Self> {
        unsafe { mem::transmute(T::storage()) }
    }

    fn on_interrupt(&mut self) {
        trace!("irq: start");
        let mut more_work = true;
        while more_work {
            more_work = false;
            match self.rx_state {
                RxState::Idle => {
                    trace!("  irq_rx: in state idle");

                    if self.inner.events_rxdrdy.read().bits() != 0 {
                        trace!("  irq_rx: rxdrdy?????");
                        self.inner.events_rxdrdy.reset();
                    }

                    if self.inner.events_endrx.read().bits() != 0 {
                        panic!("unexpected endrx");
                    }

                    let buf = self.rx.push_buf();
                    if buf.len() != 0 {
                        trace!("  irq_rx: starting {:?}", buf.len());
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
                        trace!("  irq_rx: buf {:?} {:?}", buf.as_ptr() as u32, buf.len());

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
                    trace!("  irq_rx: in state receiving");
                    if self.inner.events_rxdrdy.read().bits() != 0 {
                        trace!("  irq_rx: rxdrdy");

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
                    trace!("  irq_rx: in state ReceivingReady");

                    if self.inner.events_rxdrdy.read().bits() != 0 {
                        trace!("  irq_rx: rxdrdy");
                        self.inner.events_rxdrdy.reset();
                    }

                    if self.inner.events_endrx.read().bits() != 0 {
                        let n: usize = self.inner.rxd.amount.read().amount().bits() as usize;
                        trace!("  irq_rx: endrx {:?}", n);
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
                    trace!("  irq_tx: in state Idle");
                    let buf = self.tx.pop_buf();
                    if buf.len() != 0 {
                        trace!("  irq_tx: starting {:?}", buf.len());
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
                    trace!("  irq_tx: in state Transmitting");
                    if self.inner.events_endtx.read().bits() != 0 {
                        self.inner.events_endtx.reset();

                        trace!("  irq_tx: endtx {:?}", n);
                        self.tx.pop(n);
                        self.tx_waker.wake();
                        self.tx_state = TxState::Idle;
                        more_work = true; // start another tx if possible
                    }
                }
            }
        }
        trace!("irq: end");
    }
}

mod private {
    pub trait Sealed {}

    impl Sealed for crate::pac::UARTE0 {}
    #[cfg(any(feature = "52833", feature = "52840", feature = "9160"))]
    impl Sealed for crate::pac::UARTE1 {}
}

pub trait Instance:
    Deref<Target = uarte0::RegisterBlock> + Sized + private::Sealed + 'static
{
    type Interrupt: OwnedInterrupt;
    fn storage() -> &'static peripheral::Store<State<'static, Self>>;
}

impl Instance for pac::UARTE0 {
    type Interrupt = interrupt::UARTE0_UART0Interrupt;
    fn storage() -> &'static peripheral::Store<State<'static, Self>> {
        static STORAGE: peripheral::Store<State<'static, crate::pac::UARTE0>> =
            peripheral::Store::uninit();
        &STORAGE
    }
}

impl Instance for pac::UARTE1 {
    type Interrupt = interrupt::UARTE1Interrupt;
    fn storage() -> &'static peripheral::Store<State<'static, Self>> {
        static STORAGE: peripheral::Store<State<'static, crate::pac::UARTE1>> =
            peripheral::Store::uninit();
        &STORAGE
    }
}
