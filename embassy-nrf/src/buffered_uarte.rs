use core::cmp::min;
use core::mem;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};
use embassy::interrupt::InterruptExt;
use embassy::io::{AsyncBufRead, AsyncWrite, Result};
use embassy::util::{Unborrow, WakerRegistration};
use embassy_extras::peripheral::{PeripheralMutex, PeripheralState};
use embassy_extras::ring_buffer::RingBuffer;
use embassy_extras::{low_power_wait_until, unborrow};

use crate::fmt::{panic, *};
use crate::gpio::sealed::Pin as _;
use crate::gpio::{OptionalPin as GpioOptionalPin, Pin as GpioPin};
use crate::pac;
use crate::ppi::{AnyConfigurableChannel, ConfigurableChannel, Event, Ppi, Task};
use crate::timer::Instance as TimerInstance;
use crate::uarte::{Config, Instance as UarteInstance};

// Re-export SVD variants to allow user to directly set values
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

struct State<'d, U: UarteInstance, T: TimerInstance> {
    uarte: U,
    timer: T,
    _ppi_ch1: Ppi<'d, AnyConfigurableChannel>,
    _ppi_ch2: Ppi<'d, AnyConfigurableChannel>,

    rx: RingBuffer<'d>,
    rx_state: RxState,
    rx_waker: WakerRegistration,

    tx: RingBuffer<'d>,
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
pub struct BufferedUarte<'d, U: UarteInstance, T: TimerInstance> {
    inner: PeripheralMutex<State<'d, U, T>>,
}

impl<'d, U: UarteInstance, T: TimerInstance> BufferedUarte<'d, U, T> {
    /// unsafe: may not leak self or futures
    pub unsafe fn new(
        uarte: impl Unborrow<Target = U> + 'd,
        timer: impl Unborrow<Target = T> + 'd,
        ppi_ch1: impl Unborrow<Target = impl ConfigurableChannel> + 'd,
        ppi_ch2: impl Unborrow<Target = impl ConfigurableChannel> + 'd,
        irq: impl Unborrow<Target = U::Interrupt> + 'd,
        rxd: impl Unborrow<Target = impl GpioPin> + 'd,
        txd: impl Unborrow<Target = impl GpioPin> + 'd,
        cts: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        rts: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        config: Config,
        rx_buffer: &'d mut [u8],
        tx_buffer: &'d mut [u8],
    ) -> Self {
        unborrow!(uarte, timer, ppi_ch1, ppi_ch2, irq, rxd, txd, cts, rts);

        let r = U::regs();
        let rt = timer.regs();

        rxd.conf().write(|w| w.input().connect().drive().h0h1());
        r.psel.rxd.write(|w| unsafe { w.bits(rxd.psel_bits()) });

        txd.set_high();
        txd.conf().write(|w| w.dir().output().drive().h0h1());
        r.psel.txd.write(|w| unsafe { w.bits(txd.psel_bits()) });

        if let Some(pin) = rts.pin_mut() {
            pin.set_high();
            pin.conf().write(|w| w.dir().output().drive().h0h1());
        }
        r.psel.cts.write(|w| unsafe { w.bits(cts.psel_bits()) });

        if let Some(pin) = cts.pin_mut() {
            pin.conf().write(|w| w.input().connect().drive().h0h1());
        }
        r.psel.rts.write(|w| unsafe { w.bits(rts.psel_bits()) });

        r.baudrate.write(|w| w.baudrate().variant(config.baudrate));
        r.config.write(|w| w.parity().variant(config.parity));

        // Configure
        let hardware_flow_control = match (rts.pin().is_some(), cts.pin().is_some()) {
            (false, false) => false,
            (true, true) => true,
            _ => panic!("RTS and CTS pins must be either both set or none set."),
        };
        r.config.write(|w| {
            w.hwfc().bit(hardware_flow_control);
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
        r.enable.write(|w| w.enable().enabled());

        // BAUDRATE register values are `baudrate * 2^32 / 16000000`
        // source: https://devzone.nordicsemi.com/f/nordic-q-a/391/uart-baudrate-register-values
        //
        // We want to stop RX if line is idle for 2 bytes worth of time
        // That is 20 bits (each byte is 1 start bit + 8 data bits + 1 stop bit)
        // This gives us the amount of 16M ticks for 20 bits.
        let timeout = 0x8000_0000 / (config.baudrate as u32 / 40);

        rt.tasks_stop.write(|w| unsafe { w.bits(1) });
        rt.bitmode.write(|w| w.bitmode()._32bit());
        rt.prescaler.write(|w| unsafe { w.prescaler().bits(0) });
        rt.cc[0].write(|w| unsafe { w.bits(timeout) });
        rt.mode.write(|w| w.mode().timer());
        rt.shorts.write(|w| {
            w.compare0_clear().set_bit();
            w.compare0_stop().set_bit();
            w
        });

        let mut ppi_ch1 = Ppi::new(ppi_ch1.degrade_configurable());
        ppi_ch1.set_event(Event::from_reg(&r.events_rxdrdy));
        ppi_ch1.set_task(Task::from_reg(&rt.tasks_clear));
        ppi_ch1.set_fork_task(Task::from_reg(&rt.tasks_start));
        ppi_ch1.enable();

        let mut ppi_ch2 = Ppi::new(ppi_ch2.degrade_configurable());
        ppi_ch2.set_event(Event::from_reg(&rt.events_compare[0]));
        ppi_ch2.set_task(Task::from_reg(&r.tasks_stoprx));
        ppi_ch2.enable();

        BufferedUarte {
            inner: PeripheralMutex::new(
                State {
                    uarte,
                    timer,
                    _ppi_ch1: ppi_ch1,
                    _ppi_ch2: ppi_ch2,

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

    pub fn set_baudrate(self: Pin<&mut Self>, baudrate: Baudrate) {
        self.inner().with(|state, _irq| {
            let r = U::regs();
            let rt = state.timer.regs();

            let timeout = 0x8000_0000 / (baudrate as u32 / 40);
            rt.cc[0].write(|w| unsafe { w.bits(timeout) });
            rt.tasks_clear.write(|w| unsafe { w.bits(1) });

            r.baudrate.write(|w| w.baudrate().variant(baudrate));
        });
    }

    fn inner(self: Pin<&mut Self>) -> Pin<&mut PeripheralMutex<State<'d, U, T>>> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner) }
    }
}

impl<'d, U: UarteInstance, T: TimerInstance> AsyncBufRead for BufferedUarte<'d, U, T> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        let mut inner = self.inner();
        inner.as_mut().register_interrupt();
        inner.with(|state, _irq| {
            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started
            compiler_fence(Ordering::SeqCst);
            trace!("poll_read");

            // We have data ready in buffer? Return it.
            let buf = state.rx.pop_buf();
            if !buf.is_empty() {
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
        let mut inner = self.inner();
        inner.as_mut().register_interrupt();
        inner.with(|state, irq| {
            trace!("consume {:?}", amt);
            state.rx.pop(amt);
            irq.pend();
        })
    }
}

impl<'d, U: UarteInstance, T: TimerInstance> AsyncWrite for BufferedUarte<'d, U, T> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let mut inner = self.inner();
        inner.as_mut().register_interrupt();
        inner.with(|state, irq| {
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

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started
            compiler_fence(Ordering::SeqCst);

            irq.pend();

            Poll::Ready(Ok(n))
        })
    }
}

impl<'a, U: UarteInstance, T: TimerInstance> Drop for State<'a, U, T> {
    fn drop(&mut self) {
        let r = U::regs();
        let rt = self.timer.regs();

        // TODO this probably deadlocks. do like Uarte instead.

        rt.tasks_stop.write(|w| unsafe { w.bits(1) });
        if let RxState::Receiving = self.rx_state {
            r.tasks_stoprx.write(|w| unsafe { w.bits(1) });
        }
        if let TxState::Transmitting(_) = self.tx_state {
            r.tasks_stoptx.write(|w| unsafe { w.bits(1) });
        }
        if let RxState::Receiving = self.rx_state {
            low_power_wait_until(|| r.events_endrx.read().bits() == 1);
        }
        if let TxState::Transmitting(_) = self.tx_state {
            low_power_wait_until(|| r.events_endtx.read().bits() == 1);
        }
    }
}

impl<'a, U: UarteInstance, T: TimerInstance> PeripheralState for State<'a, U, T> {
    type Interrupt = U::Interrupt;
    fn on_interrupt(&mut self) {
        trace!("irq: start");
        let r = U::regs();
        let rt = self.timer.regs();

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
                        r.tasks_startrx.write(|w|
                            // `1` is a valid value to write to task registers.
                            unsafe { w.bits(1) });
                    }
                    break;
                }
                RxState::Receiving => {
                    trace!("  irq_rx: in state receiving");
                    if r.events_endrx.read().bits() != 0 {
                        rt.tasks_stop.write(|w| unsafe { w.bits(1) });

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
                        r.tasks_starttx.write(|w|
                            // `1` is a valid value to write to task registers.
                            unsafe { w.bits(1) });
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
