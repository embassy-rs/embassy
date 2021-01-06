//! HAL interface to the UARTE peripheral
//!
//! See product specification:
//!
//! - nrf52832: Section 35
//! - nrf52840: Section 6.34
use core::cmp::min;
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
use crate::hal::ppi::ConfigurablePpi;
use crate::interrupt::{self, OwnedInterrupt};
use crate::pac;
use crate::util::peripheral::{PeripheralMutex, PeripheralState};
use crate::util::ring_buffer::RingBuffer;

// Re-export SVD variants to allow user to directly set values
pub use crate::hal::uarte::Pins;
pub use pac::uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

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

struct State<'a, U: Instance, T: TimerInstance, P1: ConfigurablePpi, P2: ConfigurablePpi> {
    uarte: U,
    timer: T,
    ppi_channel_1: P1,
    ppi_channel_2: P2,

    rx: RingBuffer<'a>,
    rx_state: RxState,
    rx_waker: WakerRegistration,

    tx: RingBuffer<'a>,
    tx_state: TxState,
    tx_waker: WakerRegistration,
}

/// Interface to a UARTE instance
///
/// This is a very basic interface that comes with the following limitations:
/// - The UARTE instances share the same address space with instances of UART.
///   You need to make sure that conflicting instances
///   are disabled before using `Uarte`. See product specification:
///     - nrf52832: Section 15.2
///     - nrf52840: Section 6.1.2
pub struct BufferedUarte<
    'a,
    U: Instance,
    T: TimerInstance,
    P1: ConfigurablePpi,
    P2: ConfigurablePpi,
> {
    inner: PeripheralMutex<State<'a, U, T, P1, P2>>,
}

#[cfg(any(feature = "52833", feature = "52840"))]
fn port_bit(port: GpioPort) -> bool {
    match port {
        GpioPort::Port0 => false,
        GpioPort::Port1 => true,
    }
}

impl<'a, U: Instance, T: TimerInstance, P1: ConfigurablePpi, P2: ConfigurablePpi>
    BufferedUarte<'a, U, T, P1, P2>
{
    pub fn new(
        uarte: U,
        timer: T,
        mut ppi_channel_1: P1,
        mut ppi_channel_2: P2,
        irq: U::Interrupt,
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

        // Disable the irq, let the Registration enable it when everything is set up.
        irq.disable();
        irq.pend();

        // BAUDRATE register values are `baudrate * 2^32 / 16000000`
        // source: https://devzone.nordicsemi.com/f/nordic-q-a/391/uart-baudrate-register-values
        //
        // We want to stop RX if line is idle for 2 bytes worth of time
        // That is 20 bits (each byte is 1 start bit + 8 data bits + 1 stop bit)
        // This gives us the amount of 16M ticks for 20 bits.
        let timeout = 0x8000_0000 / (baudrate as u32 / 40);

        timer.tasks_stop.write(|w| unsafe { w.bits(1) });
        timer.bitmode.write(|w| w.bitmode()._32bit());
        timer.prescaler.write(|w| unsafe { w.prescaler().bits(0) });
        timer.cc[0].write(|w| unsafe { w.bits(timeout) });
        timer.mode.write(|w| w.mode().timer());
        timer.shorts.write(|w| {
            w.compare0_clear().set_bit();
            w.compare0_stop().set_bit();
            w
        });

        ppi_channel_1.set_event_endpoint(&uarte.events_rxdrdy);
        ppi_channel_1.set_task_endpoint(&timer.tasks_clear);
        ppi_channel_1.set_fork_task_endpoint(&timer.tasks_start);
        ppi_channel_1.enable();

        ppi_channel_2.set_event_endpoint(&timer.events_compare[0]);
        ppi_channel_2.set_task_endpoint(&uarte.tasks_stoprx);
        ppi_channel_2.enable();

        BufferedUarte {
            inner: PeripheralMutex::new(
                State {
                    uarte,
                    timer,
                    ppi_channel_1,
                    ppi_channel_2,

                    rx: RingBuffer::new(rx_buffer),
                    rx_state: RxState::Idle,
                    rx_waker: WakerRegistration::new(),

                    tx: RingBuffer::new(tx_buffer),
                    tx_state: TxState::Idle,
                    tx_waker: WakerRegistration::new(),
                },
                irq,
            ),
        }
    }

    fn inner(self: Pin<&mut Self>) -> Pin<&mut PeripheralMutex<State<'a, U, T, P1, P2>>> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner) }
    }
}

impl<'a, U: Instance, T: TimerInstance, P1: ConfigurablePpi, P2: ConfigurablePpi> Drop
    for BufferedUarte<'a, U, T, P1, P2>
{
    fn drop(&mut self) {
        // stop DMA before dropping, because DMA is using the buffer in `self`.
        todo!()
    }
}

impl<'a, U: Instance, T: TimerInstance, P1: ConfigurablePpi, P2: ConfigurablePpi> AsyncBufRead
    for BufferedUarte<'a, U, T, P1, P2>
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        self.inner().with(|state, _irq| {
            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started
            compiler_fence(Ordering::SeqCst);
            trace!("poll_read");

            // We have data ready in buffer? Return it.
            let buf = state.rx.pop_buf();
            if buf.len() != 0 {
                trace!("  got {:?} {:?}", buf.as_ptr() as u32, buf.len());
                let buf: &[u8] = buf;
                let buf: &[u8] = unsafe { mem::transmute(buf) };
                return Poll::Ready(Ok(buf));
            }

            trace!("  empty");
            state.rx_waker.register(cx.waker());
            Poll::<Result<&[u8]>>::Pending
        })
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.inner().with(|state, irq| {
            trace!("consume {:?}", amt);
            state.rx.pop(amt);
            irq.pend();
        })
    }
}

impl<'a, U: Instance, T: TimerInstance, P1: ConfigurablePpi, P2: ConfigurablePpi> AsyncWrite
    for BufferedUarte<'a, U, T, P1, P2>
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        self.inner().with(|state, irq| {
            trace!("poll_write: {:?}", buf.len());

            let tx_buf = state.tx.push_buf();
            if tx_buf.len() == 0 {
                trace!("poll_write: pending");
                state.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            let n = min(tx_buf.len(), buf.len());
            tx_buf[..n].copy_from_slice(&buf[..n]);
            state.tx.push(n);

            trace!("poll_write: queued {:?}", n);

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started
            compiler_fence(Ordering::SeqCst);

            irq.pend();

            Poll::Ready(Ok(n))
        })
    }
}

impl<'a, U: Instance, T: TimerInstance, P1: ConfigurablePpi, P2: ConfigurablePpi> PeripheralState
    for State<'a, U, T, P1, P2>
{
    type Interrupt = U::Interrupt;
    fn on_interrupt(&mut self) {
        trace!("irq: start");
        loop {
            match self.rx_state {
                RxState::Idle => {
                    trace!("  irq_rx: in state idle");

                    let buf = self.rx.push_buf();
                    if buf.len() != 0 {
                        trace!("  irq_rx: starting {:?}", buf.len());
                        self.rx_state = RxState::Receiving;

                        // Set up the DMA read
                        self.uarte.rxd.ptr.write(|w|
                            // The PTR field is a full 32 bits wide and accepts the full range
                            // of values.
                            unsafe { w.ptr().bits(buf.as_ptr() as u32) });
                        self.uarte.rxd.maxcnt.write(|w|
                            // We're giving it the length of the buffer, so no danger of
                            // accessing invalid memory. We have verified that the length of the
                            // buffer fits in an `u8`, so the cast to `u8` is also fine.
                            //
                            // The MAXCNT field is at least 8 bits wide and accepts the full
                            // range of values.
                            unsafe { w.maxcnt().bits(buf.len() as _) });
                        trace!("  irq_rx: buf {:?} {:?}", buf.as_ptr() as u32, buf.len());

                        // Start UARTE Receive transaction
                        self.uarte.tasks_startrx.write(|w|
                            // `1` is a valid value to write to task registers.
                            unsafe { w.bits(1) });
                    }
                    break;
                }
                RxState::Receiving => {
                    trace!("  irq_rx: in state receiving");
                    if self.uarte.events_endrx.read().bits() != 0 {
                        self.timer.tasks_stop.write(|w| unsafe { w.bits(1) });

                        let n: usize = self.uarte.rxd.amount.read().amount().bits() as usize;
                        trace!("  irq_rx: endrx {:?}", n);
                        self.rx.push(n);

                        self.uarte.events_endrx.reset();

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
                    if buf.len() != 0 {
                        trace!("  irq_tx: starting {:?}", buf.len());
                        self.tx_state = TxState::Transmitting(buf.len());

                        // Set up the DMA write
                        self.uarte.txd.ptr.write(|w|
                            // The PTR field is a full 32 bits wide and accepts the full range
                            // of values.
                            unsafe { w.ptr().bits(buf.as_ptr() as u32) });
                        self.uarte.txd.maxcnt.write(|w|
                            // We're giving it the length of the buffer, so no danger of
                            // accessing invalid memory. We have verified that the length of the
                            // buffer fits in an `u8`, so the cast to `u8` is also fine.
                            //
                            // The MAXCNT field is 8 bits wide and accepts the full range of
                            // values.
                            unsafe { w.maxcnt().bits(buf.len() as _) });

                        // Start UARTE Transmit transaction
                        self.uarte.tasks_starttx.write(|w|
                            // `1` is a valid value to write to task registers.
                            unsafe { w.bits(1) });
                    }
                    break;
                }
                TxState::Transmitting(n) => {
                    trace!("  irq_tx: in state Transmitting");
                    if self.uarte.events_endtx.read().bits() != 0 {
                        self.uarte.events_endtx.reset();

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

mod sealed {
    pub trait Instance {}

    impl Instance for crate::pac::UARTE0 {}
    #[cfg(any(feature = "52833", feature = "52840", feature = "9160"))]
    impl Instance for crate::pac::UARTE1 {}

    pub trait TimerInstance {}
    impl TimerInstance for crate::pac::TIMER0 {}
    impl TimerInstance for crate::pac::TIMER1 {}
    impl TimerInstance for crate::pac::TIMER2 {}
}

pub trait Instance: Deref<Target = pac::uarte0::RegisterBlock> + sealed::Instance {
    type Interrupt: OwnedInterrupt;
}

impl Instance for pac::UARTE0 {
    type Interrupt = interrupt::UARTE0_UART0Interrupt;
}

#[cfg(any(feature = "52833", feature = "52840", feature = "9160"))]
impl Instance for pac::UARTE1 {
    type Interrupt = interrupt::UARTE1Interrupt;
}

pub trait TimerInstance:
    Deref<Target = pac::timer0::RegisterBlock> + sealed::TimerInstance
{
}
impl TimerInstance for crate::pac::TIMER0 {}
impl TimerInstance for crate::pac::TIMER1 {}
impl TimerInstance for crate::pac::TIMER2 {}
